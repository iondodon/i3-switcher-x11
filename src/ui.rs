use gdk4::glib::{self, clone};
use gtk4::prelude::WidgetExt;
use gtk4::prelude::BoxExt;
use gtk4::prelude::FrameExt;
use gtk4::Application;
use gtk4::CssProvider;
use gtk4::Frame;
use gtk4::Image;
use gtk4::{ApplicationWindow, EventControllerKey};
use i3ipc::I3Connection;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use gtk4::prelude::GtkWindowExt;
use gtk4::glib::ControlFlow;


pub fn setup(app: &Application, i3_conn: Arc<Mutex<I3Connection>>, is_visible: Arc<AtomicBool>) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("First GTK Program")
        .hexpand_set(true)
        .hexpand(true)
        .build();

    let controller = EventControllerKey::new();
    let window_clone = window.clone();
    let is_visible_clone = is_visible.clone();
    controller.connect_key_released(move |_, keyval, _, _| {
        match keyval.name().unwrap().as_str() {
            "Alt_L" => { 
                window_clone.hide(); 
                println!("Alt released gtk");
                is_visible_clone.store(false, Ordering::SeqCst);
            },
            _ => {}
        }
    });
    window.add_controller(controller);

    let provider = CssProvider::new();
    provider.load_from_data("
        frame {
            background-color: red;
            border-radius: 0px;
        }

        .selected_frame {
            background-color: blue;
        }
    ");
    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    
    update_window_content(&window, i3_conn.clone());
    
    window.present();
    window.hide();

    let is_visible_clone = is_visible.clone();
    let i3_conn_clone = i3_conn.clone();
    glib::timeout_add_local(Duration::from_millis(100), clone!(@weak window => @default-return ControlFlow::Continue, move || {
        println!("Now is {}", is_visible_clone.load(Ordering::SeqCst));
        if is_visible_clone.load(Ordering::SeqCst) {
            update_window_content(&window, i3_conn_clone.to_owned());
            window.show();
        } else {
            window.hide();
        }
        glib::ControlFlow::Continue
    }));
}


extern crate x11;

use std::ptr;
use x11::xlib;


fn update_window_content(window: &ApplicationWindow, i3_conn: Arc<Mutex<I3Connection>>) {
    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 3);
    hbox.set_hexpand(true);

    let mut i3_conn = i3_conn.lock().unwrap();
    let wks = i3_conn.get_workspaces().unwrap();

    let mut count = 0;
    for ws in &wks.workspaces {
        let ws_frame = Frame::new(Some(&ws.name));

        if count == 0 {
            ws_frame.add_css_class("selected_frame");
        }

        let img = Image::new();

        ws_frame.set_child(Some(&img));
        hbox.append(&ws_frame);

        count = count + 1;
    }

    window.set_child(Some(&hbox));

    // move window to center
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null());
        let screen = xlib::XDefaultScreen(display);
        let screen_width = xlib::XDisplayWidth(display, screen) as i32;
        let screen_height = xlib::XDisplayHeight(display, screen) as i32;

        let window_width = window.width();
        let window_height = window.height();

        let x = (screen_width - window_width) / 2;
        let y = (screen_height - window_height) / 2;

        let command = format!("[title=\"First GTK Program\"] move window to position {} {}", x, y);
        i3_conn.run_command(&command).unwrap();

        xlib::XCloseDisplay(display);
    }
}