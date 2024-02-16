use gdk4::glib::{self, clone};
use gtk4::prelude::WidgetExt;
use gtk4::prelude::BoxExt;
use gtk4::Application;
use gtk4::CssProvider;
use gtk4::Frame;
use gtk4::{ApplicationWindow, EventControllerKey};
use i3ipc::I3Connection;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use gtk4::prelude::GtkWindowExt;
use gtk4::glib::ControlFlow;

pub fn setup(
    app: &Application, i3_conn: Arc<Mutex<I3Connection>>, 
    is_visible: Arc<AtomicBool>, 
    selected_index: Arc<AtomicI8>
) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("First GTK Program")
        .hexpand_set(true)
        .hexpand(true)
        .build();

    let controller = EventControllerKey::new();
    let window_clone = window.clone();
    let is_visible_clone = is_visible.clone();
    let selected_index_clone = selected_index.clone();
    controller.connect_key_released(move |_, keyval, _, _| {
        match keyval.name().unwrap().as_str() {
            "Alt_L" => { 
                println!("Alt released gtk");
                window_clone.hide(); 
                is_visible_clone.store(false, Ordering::SeqCst);
                selected_index_clone.store(-1, Ordering::SeqCst);
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
    glib::timeout_add_local(Duration::from_millis(100), clone!(@weak window => @default-return ControlFlow::Continue, move || {
        println!("Now is {}", is_visible_clone.load(Ordering::SeqCst));
        if is_visible_clone.load(Ordering::SeqCst) {
            let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 3);
            hbox.set_homogeneous(true);

            let wks = i3_conn.lock().unwrap().get_workspaces().unwrap().workspaces;
            let mut sindex = selected_index.load(Ordering::SeqCst);
            if sindex as usize > wks.len() - 1 {
                sindex = 0;
                selected_index.store(0, Ordering::SeqCst);
            }
            for (index, ws) in (&wks).iter().enumerate() {
                let ws_frame = Frame::builder().label(ws.name.to_string()).build();
                if index as i8 == sindex {
                    ws_frame.add_css_class("selected_frame");
                }
                hbox.append(&ws_frame);
            }
            
            window.set_child(Some(&hbox));

            window.show();
        } else {
            window.hide();
        }
        glib::ControlFlow::Continue
    }));
}
