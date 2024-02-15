use gdk4::glib::{self, clone};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, ButtonExt};
use gtk4::prelude::WidgetExt;
use gtk4::prelude::BoxExt;
use gtk4::{Application, ApplicationWindow, Button, EventControllerKey};
use i3ipc::I3Connection;
use x11::xlib::{self};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{ptr, thread};
use gtk4::prelude::GtkWindowExt;
use gtk4::glib::ControlFlow;

fn listen_alt_tab(is_visible: Arc<AtomicBool>) {
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null());
        if display.is_null() {
            panic!("Cannot open display");
        }

        let screen = xlib::XDefaultScreen(display);
        let root_window = xlib::XRootWindow(display, screen);

        
        const XK_TAB: u64 = x11::keysym::XK_Tab as u64;
        const XK_ALT_L: u64 = x11::keysym::XK_Alt_L as u64; 
        let tab_key = xlib::XKeysymToKeycode(display, XK_TAB) as i32;
        let alt_key = xlib::XKeysymToKeycode(display, XK_ALT_L) as i32;
        let alt_mask = xlib::Mod1Mask;

        // Grab Alt+Tab
        xlib::XGrabKey(display, tab_key, alt_mask, root_window, 1, xlib::GrabModeAsync, xlib::GrabModeAsync);

        // Event loop
        loop {
            let mut event: xlib::XEvent = std::mem::zeroed();
            xlib::XNextEvent(display, &mut event);

            match event.get_type() {
                xlib::KeyPress => {
                    println!("Alt+Tab Pressed");
                    is_visible.store(true, Ordering::SeqCst);
                },
                xlib::KeyRelease => {
                    let xkey = xlib::XKeyEvent::from(event);
                    if xkey.keycode == alt_key as u32 {
                        is_visible.store(false, Ordering::SeqCst);
                    }
                    if xkey.keycode == tab_key as u32 {
                        //
                    }
                }
                _ => {
                   
                }
            }

        }

        // xlib::XCloseDisplay(display); // Never reached in this loop example, showld be called when the app is closed.
    }
}

fn focus_window(window_id: i64) {
    let mut connection = I3Connection::connect().unwrap();
    let command = format!("[con_id={}] focus", window_id);
    connection.run_command(&command).unwrap();
}


fn main() -> Result<(), Box<dyn Error>> {
    // Establish a connection to the i3 IPC interface
    let mut connection = I3Connection::connect().unwrap();

    let is_visible = Arc::new(AtomicBool::new(false));

    let workspaces = connection.get_workspaces().unwrap();

    let is_visible_clone = is_visible.clone();
    thread::spawn(|| { listen_alt_tab(is_visible_clone) });
    
    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("First GTK Program")
            .default_width(350)
            .default_height(70)
            .build();

         // Create a vertical GtkBox to hold the buttons
        let vbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 1);

        for ws in &workspaces.workspaces {
            let button = Button::with_label(&ws.name);
            button.connect_clicked(|_| {
                eprintln!("Clicked!");
                // You might want to call focus_window(ws.id) here
            });

            // Add each button to the vbox container
            vbox.append(&button); // Use append for GTK 4
        }

        // Set the vbox as the child of the window
        window.set_child(Some(&vbox));

        // Create a new EventControllerKey for detecting key events
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
                _ => println!("Key release ignored")
            }
        });
        window.add_controller(controller);
        window.present();
        window.hide();

        let is_visible_clone = is_visible.clone();
        glib::timeout_add_local(Duration::from_millis(100), clone!(@weak window => @default-return ControlFlow::Continue, move || {
            println!("Now is {}", is_visible_clone.load(Ordering::SeqCst));
            if is_visible_clone.load(Ordering::SeqCst) {
                window.show();
            } else {
                window.hide();
            }
            glib::ControlFlow::Continue
        }));
    });

    application.run();

    Ok(())
}