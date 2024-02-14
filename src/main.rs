use gdk4::glib::{self, clone};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, ButtonExt};
use gtk4::prelude::WidgetExt;
use gtk4::{Application, ApplicationWindow, Button, EventControllerKey};
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

        // Define the keysym for Tab and Alt
        const XK_TAB: u64 = 0xFF09;
        const XK_ALT_L: u64 = 0xFFE9; // Left Alt keysym
        let tab_key = xlib::XKeysymToKeycode(display, XK_TAB) as i32;
        let alt_key = xlib::XKeysymToKeycode(display, XK_ALT_L) as i32;
        let alt_mask = xlib::Mod1Mask;

        // Grab Alt+Tab
        xlib::XGrabKey(display, tab_key, alt_mask, root_window, 1, xlib::GrabModeAsync, xlib::GrabModeAsync);

        // Optionally grab the Alt key specifically if needed
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
                    println!("Hmmmm");
                }
            }

            // thread::sleep(Duration::from_millis(200));
        }

        // xlib::XCloseDisplay(display); // Never reached in this loop example
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let is_visible = Arc::new(AtomicBool::new(true));

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

        let button = Button::with_label("Click me!");
        button.connect_clicked(|_| {
            eprintln!("Clicked!");
        });
        window.set_child(Some(&button));

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

        // Attach the event controller to the window
        window.add_controller(controller);

        window.present();

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