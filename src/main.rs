use gtk4::prelude::{ApplicationExt, ApplicationExtManual, ButtonExt};
use gtk4::prelude::WidgetExt;
use gtk4::{Application, ApplicationWindow, Button, EventControllerKey};
use x11::xlib::{self};
use std::error::Error;
use std::{ptr, thread};
use gtk4::prelude::GtkWindowExt;

fn listen_alt_tab() {
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null());
        if display.is_null() {
            panic!("Cannot open display");
        }

        let screen = xlib::XDefaultScreen(display);
        let root_window = xlib::XRootWindow(display, screen);

        // Define the keysym for Tab and Alt
        const XK_TAB: u64 = 0xFF09;
        let tab_key = xlib::XKeysymToKeycode(display, XK_TAB) as i32;
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
                    println!("Alt+Tab Pressed\n");
                },
                _ => {
                    println!("Hmmmm");
                }
            }
        }

        // xlib::XCloseDisplay(display); // Never reached in this loop example
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    
    thread::spawn(|| { listen_alt_tab() });
    
    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(|app| {
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
        controller.connect_key_released(move |_, keyval, _, _| {
            match keyval.name().unwrap().as_str() {
                "Alt_L" => window_clone.hide(),
                _ => println!("Key release ignored")
            }
        });

        // Attach the event controller to the window
        window.add_controller(controller);

        window.present();
    });

    application.run();

    Ok(())
}