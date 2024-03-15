use std::{ptr, sync::atomic::Ordering};

use x11::xlib;

use crate::state;

pub fn listen_alt_tab() {
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null());
        if display.is_null() {
            log::error!("Cannot open display");
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
        xlib::XGrabKey(
            display,
            tab_key,
            alt_mask,
            root_window,
            1,
            xlib::GrabModeAsync,
            xlib::GrabModeAsync,
        );

        loop {
            let mut event: xlib::XEvent = std::mem::zeroed();
            xlib::XNextEvent(display, &mut event);

            match event.get_type() {
                xlib::KeyPress => {
                    log::debug!("Alt+Tab Pressed [X11]");
                    state::IS_VISIBLE.store(true, Ordering::SeqCst);
                    let index = state::SELECTED_INDEX.load(Ordering::SeqCst);
                    state::SELECTED_INDEX.store(index + 1, Ordering::SeqCst);
                    state::SELECTED_INDEX_CHANGED.store(true, Ordering::SeqCst);
                }
                xlib::KeyRelease => {
                    let xkey = xlib::XKeyEvent::from(event);
                    if xkey.keycode == alt_key as u32 {
                        state::IS_VISIBLE.store(false, Ordering::SeqCst);
                        state::SELECTED_INDEX.store(-1, Ordering::SeqCst);
                        state::SELECTED_INDEX_CHANGED.store(true, Ordering::SeqCst);
                    }
                    if xkey.keycode == tab_key as u32 {
                        //
                    }
                }
                _ => {}
            }
        }

        // TODO: xlib::XCloseDisplay(display); // Never reached in this loop example, showld be called when the app is closed.
    }
}
