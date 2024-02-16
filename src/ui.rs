use gdk4::glib::{self, clone};
use gtk4::prelude::WidgetExt;
use gtk4::prelude::BoxExt;
use gtk4::Application;
use gtk4::CssProvider;
use gtk4::Frame;
use gtk4::Image;
use gtk4::{ApplicationWindow, EventControllerKey};
use i3ipc::I3Connection;
use x11::xlib;
use std::process::Command;
use std::ptr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use gtk4::prelude::GtkWindowExt;
use gtk4::glib::ControlFlow;
use crate::i3wm;

fn capture_screenshot(workspace_name: String) {
    println!("Capturing screenshot of workspace: {}", workspace_name);
    let filename = format!("/tmp/{}.png", workspace_name);
    Command::new("rm")
        .arg(&filename)
        .output()
        .expect("Failed to remove screenshot");
    Command::new("scrot")
        .arg(&filename)
        .output()
        .expect("Failed to capture screenshot");
    println!("Screenshot saved to {}", filename);
}

pub fn setup(
    app: &Application, i3_conn: Arc<RwLock<I3Connection>>, 
    is_visible: Arc<AtomicBool>, 
    selected_index: Arc<AtomicI8>
) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("First GTK Program")
        .build();

    let focused_ws_name: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
    let current_ws_name: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

    let controller = EventControllerKey::new();
    let window_clone = window.clone();
    let is_visible_clone = is_visible.clone();
    let selected_index_clone = selected_index.clone();
    let focused_ws_name_clone = focused_ws_name.clone();
    controller.connect_key_released(move |_, keyval, _, _| {
        match keyval.name().unwrap().as_str() {
            "Alt_L" => { 
                println!("Alt released gtk");
                window_clone.hide();
                is_visible_clone.store(false, Ordering::SeqCst);
                selected_index_clone.store(-1, Ordering::SeqCst);

                let mut curr_ws_name = current_ws_name.write().unwrap();
                if let Some(name) = (*curr_ws_name).clone() {
                    capture_screenshot(name);
                }                         
                
                let focused_ws_name = focused_ws_name_clone.read().unwrap();
                if let Some(name) = (*focused_ws_name).clone() {
                    i3wm::focus_workspace(name.clone());
                    *curr_ws_name = Some(name);
                }
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
    
    window.present();
    window.hide();

    let is_visible_clone = is_visible.clone();
    let focused_ws_name_clone = focused_ws_name.clone();
    glib::timeout_add_local(Duration::from_millis(100), clone!(@weak window => @default-return ControlFlow::Continue, move || {
        println!("Now is {}", is_visible_clone.load(Ordering::SeqCst));
        if is_visible_clone.load(Ordering::SeqCst) {
            let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 3);
            hbox.set_homogeneous(true);

            let mut i3_conn_lock = i3_conn.write().unwrap();
            let wks = i3_conn_lock.get_workspaces().unwrap().workspaces;
            let mut sindex = selected_index.load(Ordering::SeqCst);
            if sindex as usize > wks.len() - 1 {
                sindex = 0;
                selected_index.store(0, Ordering::SeqCst);
            }
            for (index, ws) in (&wks).iter().enumerate() {
                let screenshot = Image::builder()
                    .file(format!("/tmp/{}.png", ws.name))
                    .build();
                let ws_frame = Frame::builder()
                    .label(ws.name.to_string())
                    .child(&screenshot)
                    .build();
                ws_frame.set_width_request(250);
                if index as i8 == sindex {
                    ws_frame.add_css_class("selected_frame");
                    let mut name = focused_ws_name_clone.write().unwrap();
                    *name = Some(ws.name.clone());
                }
                hbox.append(&ws_frame);
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
                i3_conn_lock.run_command(&command).unwrap();

                xlib::XCloseDisplay(display);
            }

            window.show();
        } else {
            if window.is_visible() {
                window.hide();
            }
        }
        glib::ControlFlow::Continue
    }));
}
