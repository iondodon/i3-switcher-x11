use gtk4::prelude::{ApplicationExt, ApplicationExtManual, ButtonExt};
use gtk4::prelude::WidgetExt;
use gtk4::{Application, ApplicationWindow, Button, EventControllerKey};
use i3ipc::{reply::NodeType, I3Connection};
use i3ipc::reply::Node;
use x11::xlib;
use std::error::Error;
use std::ptr;
use gtk4::prelude::GtkWindowExt;

fn focus_window(window_id: i64) {
    let mut connection = I3Connection::connect().unwrap();
    let command = format!("[con_id={}] focus", window_id);
    connection.run_command(&command).unwrap();
}

fn print_window_names(node: &Node) {
    // If this node represents a window, print its name
    if node.nodetype == NodeType::Workspace {
        println!("{:?}\n\n", node.nodes);
        focus_window(node.id);
        // thread::sleep(time::Duration::from_secs(2));
    }

    // Recurse into this node's children and floating nodes
    for child in &node.nodes {
        print_window_names(child);
    }
    for floating in &node.floating_nodes {
        print_window_names(floating);
    }
}

fn tray() {
    print!("HELLOOOO");
}

fn logic() {
    // Establish a connection to the i3 IPC interface
    let mut connection = I3Connection::connect().unwrap();

    // Query the layout tree
    let tree = connection.get_tree().unwrap();

    // Recursively print the names of all windows
    print_window_names(&tree);
}


fn main() -> Result<(), Box<dyn Error>> {
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

            println!("\n Event: {:?}", event);

            match event.get_type() {
                xlib::KeyPress => {
                    println!("Alt+Tab Pressed\n");
                    // Handle window switching logic here
                },
                xlib::KeyRelease => {
                    let xkey = xlib::XKeyEvent::from(event);
                    if xkey.keycode == alt_key as u32 {
                        println!("Alt Released\n");
                        // Handle Alt release logic here
                    }
                    if xkey.keycode == tab_key as u32 {
                        println!("Tab Released\n");
                        // Handle Alt release logic here
                    }
                },
                _ => {
                    println!("Hmmmm");
                }
            }
        }

        // xlib::XCloseDisplay(display); // Never reached in this loop example
    }
    
    let jh1 = std::thread::spawn(|| { tray() });
    let jh2 = std::thread::spawn(|| { logic() });

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

    let _ = jh1.join();
    let _ = jh2.join();

    Ok(())
}