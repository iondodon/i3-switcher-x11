use std::{ptr, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use x11::xlib;


pub fn listen_alt_tab(is_visible: Arc<AtomicBool>) {
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