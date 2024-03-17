use crate::i3wm;
use crate::screenshot;
use crate::state;
use gdk4::gio::prelude::ApplicationExt;
use gdk4::glib::{self, clone};
use gdk4::prelude::ApplicationExtManual;
use gdk4::prelude::DisplayExt;
use gdk4::prelude::MonitorExt;
use gtk4::glib::ControlFlow;
use gtk4::prelude::GtkWindowExt;
use gtk4::prelude::NativeExt;
use gtk4::prelude::WidgetExt;
use gtk4::Application;
use gtk4::{ApplicationWindow, EventControllerKey};
use i3ipc::event::inner::WorkspaceChange;
use i3ipc::event::Event;
use std::ffi::CString;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

mod style;
mod tabs;

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

    // add the lock inside TabsList and Tab structs
    // use smaller lock scopes
    let tabs = Arc::new(RwLock::new(tabs::TabsList::new()));

    let controller = EventControllerKey::new();
    let window_clone = window.clone();
    let tabs_clone = tabs.clone();
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
                    let mut tabs = tabs_clone.write().unwrap();
                    tabs.set_image(name, img);
                }

                let name = state::FOCUSED_TAB_NAME.read().unwrap();
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

    {
        let tabs = tabs.read().unwrap();
        window.set_child(Some(&tabs.tabs_box));
    }

    window.present();
    window.hide();

    let (i3_event_sender, i3_event_receiver): (mpsc::Sender<Event>, mpsc::Receiver<Event>) =
        mpsc::channel();

    thread::spawn(|| i3wm::listen(i3_event_sender));

    glib::timeout_add_local(
        Duration::from_millis(50),
        clone!(@weak tabs => @default-return ControlFlow::Continue, move || {

            match i3_event_receiver.try_recv() {
                Ok(event) => {
                    match event {
                        i3ipc::event::Event::WorkspaceEvent(info) => match info.change {
                            WorkspaceChange::Init => {
                                log::debug!("New workspace {:?}", info);
                                let mut tabs = tabs.write().unwrap();
                                tabs.add_new_tab(None, &info.current.unwrap().name.unwrap());
                                tabs.re_render();
                            },
                            WorkspaceChange::Empty => {
                                log::debug!("Removed workspace {:?}", info);
                                let mut tabs = tabs.write().unwrap();
                                tabs.remove_tab(&info.current.unwrap().name.unwrap());
                                tabs.re_render();
                            },
                            _ => (),
                        },
                        _ => ()
                    }
                },
                Err(_) => (),
            }

            glib::ControlFlow::Continue
        }),
    );

    glib::timeout_add_local(
        Duration::from_millis(50),
        clone!(@weak window => @default-return ControlFlow::Continue, move || {
            log::debug!("Window visible - {}", state::IS_VISIBLE.load(Ordering::SeqCst));

            if state::IS_VISIBLE.load(Ordering::SeqCst) {
                if !window.is_visible() {
                    window.show();
                }
                if state::SELECTED_INDEX_CHANGED.load(Ordering::SeqCst) {
                    let tabs = tabs.read().unwrap();
                    if state::SELECTED_INDEX.load(Ordering::SeqCst) as usize >= tabs.tabs_vec.len() {
                        state::SELECTED_INDEX.store(0, Ordering::SeqCst);
                    }
                    let selected_index = state::SELECTED_INDEX.load(Ordering::SeqCst);
                    for (index, tab) in tabs.tabs_vec.iter().enumerate() {
                        if tab.gtk_box.has_css_class("focused_tab") {
                            tab.gtk_box.remove_css_class("focused_tab");
                        }
                        if index as i8 == selected_index {
                            tab.gtk_box.add_css_class("focused_tab");
                            let label = &tab.label;
                            let label_text = label.text().to_string();
                            let mut name = state::FOCUSED_TAB_NAME.write().unwrap();
                            *name = Some(label_text);
                        }
                    }
                    state::SELECTED_INDEX_CHANGED.store(false, Ordering::SeqCst);
                }
            } else {
                if window.is_visible() {
                    window.hide();
                }
            }

            glib::ControlFlow::Continue
        }),
    );
}
