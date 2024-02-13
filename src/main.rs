use i3ipc::{reply::NodeType, I3Connection};
use i3ipc::reply::Node;
use core::time;
use std::{self, ptr, thread};
use x11::xlib;

fn focus_window(window_id: i64) {
    let mut connection = I3Connection::connect().unwrap();
    let command = format!("[con_id={}] focus", window_id);
    connection.run_command(&command).unwrap();
}

fn print_x11_window_info(window_id: i64) {
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null_mut());
        if display.is_null() {
            eprintln!("Cannot open display");
            return;
        }

        let mut attributes: xlib::XWindowAttributes = std::mem::zeroed();
        xlib::XGetWindowAttributes(display, window_id as xlib::Window, &mut attributes);

        // Getting window name
        let mut window_name = ptr::null_mut();
        xlib::XFetchName(display, window_id as xlib::Window, &mut window_name);
        let window_name = if !window_name.is_null() {
            let window_name_c_str = std::ffi::CStr::from_ptr(window_name);
            let window_name_str = window_name_c_str.to_str().unwrap_or("");
            xlib::XFree(window_name as *mut _);
            window_name_str.to_string()
        } else {
            "Unknown".to_string()
        };

        println!("Window ID: {:x}", window_id);
        println!("Window Name: {}", window_name);
        println!("Position: ({}, {})", attributes.x, attributes.y);
        println!("Size: {}x{}", attributes.width, attributes.height);

        xlib::XCloseDisplay(display);
    }
}

fn print_window_names(node: &Node) {
    // If this node represents a window, print its name
    if let Some(ref name) = node.name{
        if node.nodetype == NodeType::Con {
            println!("Window id: {}, name: {} \n", node.id, name);
            focus_window(node.id);
            if let Some(window_id) = node.window {
                print_x11_window_info(window_id as i64);
            }
            thread::sleep(time::Duration::from_secs(2));
        }
    }

    // Recurse into this node's children and floating nodes
    for child in &node.nodes {
        print_window_names(child);
    }
    for floating in &node.floating_nodes {
        print_window_names(floating);
    }
}

fn main() {
    // Establish a connection to the i3 IPC interface
    let mut connection = I3Connection::connect().unwrap();

    // Query the layout tree
    let tree = connection.get_tree().unwrap();

    // Recursively print the names of all windows
    print_window_names(&tree);
}