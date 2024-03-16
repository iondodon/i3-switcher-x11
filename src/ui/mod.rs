use crate::i3wm;
use crate::screenshot;
use crate::state;
use gdk4::gdk_pixbuf;
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
use i3ipc::event::inner::WorkspaceChange;
use i3ipc::event::Event;
use std::ffi::CString;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
use xcap::image::ImageBuffer;
use xcap::image::Rgba;

mod style;

struct Tab {
    picture: Option<Picture>,
    label: Label,
    gtk_box: gtk4::Box,
}

impl Tab {
    fn new(picture: Option<Picture>, name: Option<&str>) -> Self {
        let gtk_box = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
        gtk_box.set_width_request(300);
        gtk_box.add_css_class("tab");

        if let Some(ref pic) = picture {
            gtk_box.append(pic);
        }

        let label = Label::new(name);
        gtk_box.append(&label);

        Tab {
            picture: picture,
            label: Label::new(name),
            gtk_box: gtk_box,
        }
    }
}

struct TabsList {
    tabs_box: gtk4::Box,
    tabs_vec: Vec<Tab>,
}

impl TabsList {
    fn new() -> TabsList {
        let mut tabs = TabsList {
            tabs_box: gtk4::Box::new(gtk4::Orientation::Horizontal, 3),
            tabs_vec: Vec::<Tab>::new(),
        };

        tabs.tabs_box.set_homogeneous(true);
        tabs.tabs_box.add_css_class("tabs");

        let mut i3_conn_lock = state::I3_CONNECTION.write().unwrap();
        let wks = i3_conn_lock.get_workspaces().unwrap().workspaces;
        for (_, ws) in (&wks).iter().enumerate() {
            tabs.add_new_tab(&ws.name);
        }

        tabs.re_render();

        tabs
    }

    fn remove_tab(self: &mut Self, name: &String) {
        for (index, tab) in self.tabs_vec.iter().enumerate() {
            if tab.label.text().eq(name) {
                self.tabs_vec.remove(index);
                return;
            }
        }
    }

    fn add_new_tab(self: &mut Self, name: &String) {
        let screenshots = state::SCREENSHOTS.read().unwrap();
        let screenshot = screenshots.get(name);

        if let Some(Some(pic)) = screenshot {
            let pixbuf = screenshot::rgba_image_to_pixbuf(pic);
            let picture = Picture::for_pixbuf(&pixbuf);
            let tab = Tab::new(Some(picture), Some(name));
            self.tabs_vec.push(tab);
        } else {
            let tab = Tab::new(None, Some(name));
            self.tabs_vec.push(tab);
        }
    }

    fn re_render(self: &mut Self) {
        loop {
            match self.tabs_box.first_child() {
                Some(tab) => self.tabs_box.remove(&tab),
                None => break,
            }
        }

        for tab in &self.tabs_vec {
            self.tabs_box.append(&tab.gtk_box);
        }
    }

    fn set_image(self: &mut Self, name: String, img: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>) {
        let mut i: Option<usize> = None;
        if let Some(pic) = img {
            for (index, tab) in self.tabs_vec.iter().enumerate() {
                if tab.label.text().eq(&name) {
                    i = Some(index);
                    break;
                }
            }

            if let Some(i) = i {
                let pixbuf = screenshot::rgba_image_to_pixbuf(&pic);
                let picture = Picture::for_pixbuf(&pixbuf);
                self.tabs_vec[i] = Tab::new(Some(picture), Some(&name));
            }

            self.re_render();
        }
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

    let tabs = Arc::new(RwLock::new(TabsList::new()));

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

    let (tx, rx): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel();

    thread::spawn(|| i3wm::listen(tx));

    glib::timeout_add_local(
        Duration::from_millis(50),
        clone!(@weak tabs => @default-return ControlFlow::Continue, move || {

            match rx.try_recv() {
                Ok(event) => {
                    match event {
                        i3ipc::event::Event::WorkspaceEvent(info) => match info.change {
                            WorkspaceChange::Init => {
                                log::debug!("New workspace {:?}", info);
                                let mut tabs = tabs.write().unwrap();
                                tabs.add_new_tab(&info.current.unwrap().name.unwrap());
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
                            let label = tab.gtk_box.last_child().unwrap();
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
