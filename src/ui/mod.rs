use crate::i3wm;
use crate::screenshot;
use crate::state;
use gdk4::gio::prelude::ApplicationExt;
use gdk4::glib::object::Cast;
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

struct Tabs {
    tabs_box: gtk4::Box,
    tabs_vec: Vec<gtk4::Box>,
}

impl Tabs {
    fn new() -> Tabs {
        Tabs {
            tabs_box: gtk4::Box::new(gtk4::Orientation::Horizontal, 3),
            tabs_vec: Vec::<gtk4::Box>::new(),
        }
    }

    fn update(self: &Self) {}

    fn add_new_tab(self: &mut Self, name: &String) {
        let screenshots = state::SCREENSHOTS.read().unwrap();
        let screenshot = screenshots.get(name);

        let tab_box = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
        tab_box.set_width_request(300);
        let tab_name = Label::new(Some(&name));
        if let Some(Some(pic)) = screenshot {
            let pixbuf = screenshot::rgba_image_to_pixbuf(pic);
            let picture = Picture::for_pixbuf(&pixbuf);
            tab_box.append(&picture);
        }
        tab_box.append(&tab_name);
        tab_box.add_css_class("tab");
        self.tabs_vec.push(tab_box);
        self.tabs_box.append(self.tabs_vec.last().unwrap());
    }
}

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

    let mut tabs = Tabs::new();

    tabs.tabs_box.set_homogeneous(true);
    tabs.tabs_box.add_css_class("tabs");

    let mut i3_conn_lock = state::I3_CONNECTION.write().unwrap();
    let wks = i3_conn_lock.get_workspaces().unwrap().workspaces;
    for (_, ws) in (&wks).iter().enumerate() {
        tabs.add_new_tab(&ws.name);
    }

    window.set_child(Some(&tabs.tabs_box));
    window.present();
    window.hide();

    glib::timeout_add_local(
        Duration::from_millis(50),
        clone!(@weak window => @default-return ControlFlow::Continue, move || {
            log::debug!("Window visible - {}", state::IS_VISIBLE.load(Ordering::SeqCst));

            if state::IS_VISIBLE.load(Ordering::SeqCst) {
                if !window.is_visible() {
                    window.show();
                }
                if state::SELECTED_INDEX_CHANGED.load(Ordering::SeqCst) {
                    if state::SELECTED_INDEX.load(Ordering::SeqCst) as usize >= tabs.tabs_vec.len() {
                        state::SELECTED_INDEX.store(0, Ordering::SeqCst);
                    }
                    let selected_index = state::SELECTED_INDEX.load(Ordering::SeqCst);
                    for (index, tab_box) in tabs.tabs_vec.iter().enumerate() {
                        if tab_box.has_css_class("focused_tab") {
                            tab_box.remove_css_class("focused_tab");
                        }
                        if index as i8 == selected_index {
                            tab_box.add_css_class("focused_tab");
                            let label = tab_box.last_child().unwrap();
                            let label = label.downcast_ref::<Label>().unwrap();
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
