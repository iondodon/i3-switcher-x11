use gdk4::gdk_pixbuf::{Colorspace, Pixbuf};
use gdk4::gio::prelude::ApplicationExt;
use gdk4::glib::{self, clone};
use gdk4::prelude::ApplicationExtManual;
use gdk4::prelude::DisplayExt;
use gdk4::prelude::MonitorExt;
use gtk4::prelude::WidgetExt;
use gtk4::prelude::BoxExt;
use gtk4::prelude::NativeExt;
use gtk4::{Application, Picture};
use gtk4::Label;
use gtk4::{ApplicationWindow, EventControllerKey};
use i3ipc::I3Connection;
use image::{ImageBuffer, RgbaImage};
use x11::{xlib, xrandr};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::{ptr, slice};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use gtk4::prelude::GtkWindowExt;
use gtk4::glib::ControlFlow;
use crate::i3wm;
use lazy_static::lazy_static;

mod style;

pub fn init(is_visible: Arc<AtomicBool>, selected_index: Arc<AtomicI8>) {
    let application = Application::builder()
        .application_id("com.iondodon.i3switcherX11")
        .build();

    application.connect_activate(move |app| { 
        setup(app, is_visible.to_owned(), selected_index.to_owned()); 
    });

    application.run();
}

fn screenshot(monitor_name: CString) -> Option<RgbaImage> {
    let mut result: Option<RgbaImage> = None;
    unsafe {
        // Open a connection to the X server
        let display = xlib::XOpenDisplay(ptr::null());
        if display.is_null() {
            eprintln!("Cannot open display");
            std::process::exit(1);
        }

        // Get the default screen
        let screen = xlib::XDefaultScreen(display);

        // Get the XRandR extension version
        let mut major_version: i32 = 0;
        let mut minor_version: i32 = 0;
        xrandr::XRRQueryVersion(display, &mut major_version, &mut minor_version);

        // Get the screen resources
        let root_window = xlib::XRootWindow(display, screen);
        let screen_resources = xrandr::XRRGetScreenResources(display, root_window);

        // Find the output matching your monitor's name (e.g., "HDMI1")
        for i in 0..(*screen_resources).noutput {
            let output_info = xrandr::XRRGetOutputInfo(display, screen_resources, *(*screen_resources).outputs.add(i as usize));
            let output_name_cstr = CStr::from_ptr((*output_info).name);

            if CStr::cmp(&monitor_name, &output_name_cstr) == std::cmp::Ordering::Equal {
                // This is the monitor we're interested in
                let crtc_info = xrandr::XRRGetCrtcInfo(display, screen_resources, (*output_info).crtc);

                // Now, you can use crtc_info's x, y, width, and height to capture the screen
                let x = (*crtc_info).x as i32;
                let y = (*crtc_info).y as i32;
                let width = (*crtc_info).width as u32;
                let height = (*crtc_info).height as u32;

                // Use XGetImage to capture the screen portion
                let image = xlib::XGetImage(display, root_window, x, y, width, height, xlib::XAllPlanes(), xlib::ZPixmap);
                
                if !image.is_null() {
                    let width = (*image).width as u32;
                    let height = (*image).height as u32;

                    let bitmap_data = slice::from_raw_parts((*image).data as *const u8, (width * height * 4) as usize); // Assuming 32-bit color depth

                    // Create a new ImgBuf with width: width and height: height
                    let imgbuf: RgbaImage = ImageBuffer::from_raw(width, height, bitmap_data.to_vec()).unwrap();
                    
                    result = Some(imgbuf);
                }

                xlib::XFree(crtc_info as *mut _);
            }

            xlib::XFree(output_info as *mut _);
        }

        xlib::XFree(screen_resources as *mut _);
        xlib::XCloseDisplay(display);

        result
    }
}

lazy_static! {
    static ref IMAGES: RwLock<HashMap<String, RgbaImage>> = RwLock::new(HashMap::new());
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
    let i3_conn_clone = i3_conn.clone();
    controller.connect_key_released(move |_, keyval, _, _| {
        match keyval.name().unwrap().as_str() {
            "Alt_L" => { 
                log::debug!("Alt_L released [GTK]");
                window_clone.hide();
                is_visible_clone.store(false, Ordering::SeqCst);
                selected_index_clone.store(-1, Ordering::SeqCst);

                let surface = window_clone.surface().unwrap();
                let display = window_clone.display();
                let monitor = display.monitor_at_surface(&surface).unwrap();
                let monitor_name = monitor.model().unwrap();

                let mut curr_ws_name = current_ws_name.write().unwrap();
                if let Some(name) = (*curr_ws_name).clone() {
                    let monitor_name = CString::new(monitor_name.as_bytes()).expect("CString::new failed");
                    let img = screenshot(monitor_name);
                    if let Some(img) = img {
                        let mut images = IMAGES.write().unwrap();
                        images.insert(name, img);
                    }
                }

                let focused_ws_name = focused_ws_name_clone.read().unwrap();
                if let Some(name) = (*focused_ws_name).clone() {
                    i3wm::focus_workspace(name.clone(), i3_conn_clone.clone());
                    *curr_ws_name = Some(name);
                }
            },
            _ => {}
        }
    });
    window.add_controller(controller);

    style::init();
    
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
                let images = IMAGES.read().unwrap();
                let pic = images.get(&ws.name);

                let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
                vbox.set_width_request(300);
                let label = Label::new(Some(&ws.name));
                if let Some(img) = pic {
                    let pixbuf = rgba_image_to_pixbuf(&img);
                    let picture = Picture::for_pixbuf(&pixbuf);
                    vbox.append(&picture);
                }
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


fn rgba_image_to_pixbuf(img: &RgbaImage) -> Pixbuf {
    let width = img.width() as i32;
    let height = img.height() as i32;
    let row_stride = img.sample_layout().height_stride as i32;

    let pixbuf = Pixbuf::from_mut_slice(
        img.clone().into_raw(), // Clones the image data into a new Vec<u8>
        Colorspace::Rgb,
        true, // has_alpha
        8,    // bits_per_sample
        width,
        height,
        row_stride,
    );

    pixbuf
}