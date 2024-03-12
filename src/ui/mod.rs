use crate::i3wm;
use crate::screenshot;
use crate::state;
use gdk4::gio::prelude::ApplicationExt;
use gdk4::glib::{self, clone};
use gdk4::prelude::ApplicationExtManual;
use gdk4::prelude::DisplayExt;
use gdk4::prelude::MonitorExt;
use gtk4::glib::ControlFlow;
use gtk4::prelude::BoxExt;
use gtk4::prelude::GtkWindowExt;
use gtk4::prelude::NativeExt;
use gtk4::prelude::WidgetExt;
use gtk4::Label;
use gtk4::{Application, Picture};
use gtk4::{ApplicationWindow, EventControllerKey};
use std::ffi::CString;
use std::sync::atomic::Ordering;
use std::time::Duration;

mod style;

pub fn init() {
    let application = Application::builder()
        .application_id("com.iondodon.i3switcherX11")
        .build();

    application.connect_activate(move |app| {
        setup(app);
    });

    application.run();
}

fn setup(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("i3switcherX11")
        .css_classes(vec!["window"])
        .build();

    let controller = EventControllerKey::new();
    let window_clone = window.clone();
    controller.connect_key_released(
        move |_, keyval, _, _| match keyval.name().unwrap().as_str() {
            "Alt_L" => {
                log::debug!("Alt_L released [GTK]");
                window_clone.hide();
                state::IS_VISIBLE.store(false, Ordering::SeqCst);
                state::SELECTED_INDEX.store(-1, Ordering::SeqCst);

                let surface = window_clone.surface().unwrap();
                let display = window_clone.display();
                let monitor = display.monitor_at_surface(&surface).unwrap();
                let monitor_name = monitor.model().unwrap();

                let mut curr_ws_name = state::CURRENT_WS_NAME.write().unwrap();
                if let Some(name) = (*curr_ws_name).clone() {
                    let monitor_name =
                        CString::new(monitor_name.as_bytes()).expect("CString::new failed");
                    let img = screenshot::take(&monitor_name);
                    let mut images = state::SCREENSHOTS.write().unwrap();
                    images.insert(name, img);
                }

                let name = state::FOCUSED_WS_NAME.read().unwrap();
                if let Some(name) = (*name).clone() {
                    i3wm::focus_workspace(name.clone());
                    *curr_ws_name = Some(name);
                }
            }
            _ => {}
        },
    );
    window.add_controller(controller);

    style::init();

    window.present();
    window.hide();

    glib::timeout_add_local(
        Duration::from_millis(10),
        clone!(@weak window => @default-return ControlFlow::Continue, move || {
            log::debug!("Window visible - {}", state::IS_VISIBLE.load(Ordering::SeqCst));
            if state::IS_VISIBLE.load(Ordering::SeqCst) {
                let tabs = gtk4::Box::new(gtk4::Orientation::Horizontal, 3);
                tabs.set_homogeneous(true);
                tabs.add_css_class("tabs");

                let mut i3_conn_lock = state::I3_CONNECTION.write().unwrap();
                let wks = i3_conn_lock.get_workspaces().unwrap().workspaces;
                let mut sindex = state::SELECTED_INDEX.load(Ordering::SeqCst);
                if sindex as usize > wks.len() - 1 {
                    sindex = 0;
                    state::SELECTED_INDEX.store(0, Ordering::SeqCst);
                }
                for (index, ws) in (&wks).iter().enumerate() {
                    let screenshots = state::SCREENSHOTS.read().unwrap();
                    let screenshot = screenshots.get(&ws.name);

                    let tab = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
                    tab.set_width_request(300);
                    let tab_name = Label::new(Some(&ws.name));
                    if let Some(Some(pic)) = screenshot {
                        let pixbuf = screenshot::rgba_image_to_pixbuf(pic);
                        let picture = Picture::for_pixbuf(&pixbuf);
                        tab.append(&picture);
                    }
                    tab.append(&tab_name);
                    tab.add_css_class("tab");

                    if index as i8 == sindex {
                        tab.add_css_class("selected_tab");
                        let mut name = state::FOCUSED_WS_NAME.write().unwrap();
                        *name = Some(ws.name.clone());
                    }
                    tabs.append(&tab);
                }

                window.set_child(Some(&tabs));

                window.show();
            } else {
                if window.is_visible() {
                    window.hide();
                }
            }
            glib::ControlFlow::Continue
        }),
    );
}
