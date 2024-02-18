use gdk4::gio::prelude::ApplicationExt;
use gdk4::glib::{self, clone};
use gdk4::prelude::ApplicationExtManual;
use gtk4::prelude::WidgetExt;
use gtk4::prelude::BoxExt;
use gtk4::Application;
use gtk4::CssProvider;
use gtk4::Label;
use gtk4::{ApplicationWindow, EventControllerKey};
use i3ipc::I3Connection;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use gtk4::prelude::GtkWindowExt;
use gtk4::glib::ControlFlow;
use crate::cmd;
use crate::i3wm;

pub fn init(is_visible: Arc<AtomicBool>, selected_index: Arc<AtomicI8>) {
    let application = Application::builder()
        .application_id("com.iondodon.i3switcherX11")
        .build();

    application.connect_activate(move |app| { 
        setup(app, is_visible.to_owned(), selected_index.to_owned()); 
    });

    application.run();
}

fn setup(
    app: &Application,
    is_visible: Arc<AtomicBool>, 
    selected_index: Arc<AtomicI8>
) {
    let i3_conn = I3Connection::connect().unwrap();
    let i3_conn = Arc::new(RwLock::new(i3_conn));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("i3switcherX11")
        .css_classes(vec!["window"])
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
                log::debug!("Alt_L released [GTK]");
                window_clone.hide();
                is_visible_clone.store(false, Ordering::SeqCst);
                selected_index_clone.store(-1, Ordering::SeqCst);

                let mut curr_ws_name = current_ws_name.write().unwrap();
                if let Some(name) = (*curr_ws_name).clone() {
                    cmd::capture_screenshot(name);
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
    // reffer to https://thomashunter.name/i3-configurator/
    provider.load_from_data("
        .selected_frame {
            background-color: #4C7899;
        }

        .vbox {
            color: #FFFFFF;
        }

        .window {
            background-color: #333333;
            border-style: solid;
            border-width: 2px;
            border-color: #4C7899;
        }

        .picture {
            margin-top: 4px;
        }

        .hbox {
            background-color: #333333;
            margin: 5px;
            padding: 0.3px;
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
        log::debug!("Window visible - {}", is_visible_clone.load(Ordering::SeqCst));
        if is_visible_clone.load(Ordering::SeqCst) {
            let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 3);
            hbox.set_homogeneous(true);
            hbox.add_css_class("hbox");

            let mut i3_conn_lock = i3_conn.write().unwrap();
            let wks = i3_conn_lock.get_workspaces().unwrap().workspaces;
            let mut sindex = selected_index.load(Ordering::SeqCst);
            if sindex as usize > wks.len() - 1 {
                sindex = 0;
                selected_index.store(0, Ordering::SeqCst);
            }
            for (index, ws) in (&wks).iter().enumerate() {
                let pic = gtk4::Picture::for_filename(format!("/tmp/{}.png", ws.name));

                pic.add_css_class("picture");

                let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
                vbox.set_width_request(300);
                let label = Label::new(Some(&ws.name));
                vbox.append(&pic);
                vbox.append(&label);
                vbox.add_css_class("vbox");

                if index as i8 == sindex {
                    vbox.add_css_class("selected_frame");
                    let mut name = focused_ws_name_clone.write().unwrap();
                    *name = Some(ws.name.clone());
                }
                hbox.append(&vbox);
            }
            
            window.set_child(Some(&hbox));

            window.show();
        } else {
            if window.is_visible() {
                window.hide();
            }
        }
        glib::ControlFlow::Continue
    }));
}
