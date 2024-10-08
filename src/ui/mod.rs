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
use gtk4::ApplicationWindow;
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

    let window_clone = window.clone();
    let tabs_clone = tabs.clone();
    glib::timeout_add_local(
        Duration::from_millis(50),
        clone!(@weak tabs => @default-return ControlFlow::Continue, move || {
            if state::SHOULD_SWITCH.load(Ordering::SeqCst) {
                state::SHOULD_SWITCH.store(false, Ordering::SeqCst);
                if state::IS_VISIBLE.load(Ordering::SeqCst) {
                    state::IS_VISIBLE.store(false, Ordering::SeqCst);

                    log::debug!("Switching workspace");

                    window_clone.hide();
                    {
                        let mut tabs = tabs_clone.write().unwrap();
                        tabs.reorder_prev_first();
                    }

                    let surface = window_clone.surface().unwrap();
                    let display = window_clone.display();
                    let monitor = display.monitor_at_surface(&surface).unwrap();
                    let monitor_name = monitor.model().unwrap();

                    let mut curr_ws_name = state::CURRENT_WS_NAME.write().unwrap();
                    if let Some(name) = (*curr_ws_name).clone() {
                        let monitor_name = CString::new(monitor_name.as_bytes()).expect("CString::new failed");
                        let img = screenshot::take(&monitor_name);
                        let mut tabs = tabs_clone.write().unwrap();
                        tabs.set_image(name, img);
                    }

                    let name = state::FOCUSED_TAB_NAME.read().unwrap();
                    if let Some(name) = (*name).clone() {
                        i3wm::focus_workspace(name.clone());
                        *curr_ws_name = Some(name);
                    }

                    state::SELECTED_INDEX.store(-1, Ordering::SeqCst);
                    state::SELECTED_INDEX_CHANGED.store(true, Ordering::SeqCst);
                }
            }

            glib::ControlFlow::Continue
        }),
    );

    glib::timeout_add_local(
        Duration::from_millis(50),
        clone!(@weak tabs, @weak window => @default-return ControlFlow::Continue, move || {

            match i3_event_receiver.try_recv() {
                Ok(event) => {
                    match event {
                        i3ipc::event::Event::WorkspaceEvent(info) => match info.change {
                            WorkspaceChange::Init => {
                                log::debug!("New workspace {:?}", info);
                                let mut tabs = tabs.write().unwrap();
                                tabs.add_new_tab(None, &info.current.unwrap().name.unwrap());
                                tabs.re_render();

                                window.present(); // needed to be able to move the position of the Window

                                // re-center window, 0 and 0 for center
                                let command = format!("[title=\"i3switcherX11\"] move window to position {} {}", 0, 0);
                                let mut i3_conn = state::I3_CONNECTION.write().unwrap();
                                i3_conn.run_command(&command).unwrap();
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

                    let selected_index = state::SELECTED_INDEX.load(Ordering::SeqCst);
                    if selected_index >= tabs.tabs_vec.len() as i8 {
                        state::SELECTED_INDEX.store(0, Ordering::SeqCst);
                    } else if selected_index < 0 {
                        state::SELECTED_INDEX.store(tabs.tabs_vec.len() as i8 - 1, Ordering::SeqCst);
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
