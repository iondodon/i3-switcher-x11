use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::Application;
use i3ipc::I3Connection;
use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicI8};
use std::sync::{Arc, Mutex};
use std::thread;

mod x11_listener;
mod ui;


fn main() -> Result<(), Box<dyn Error>> {
    let i3_conn = I3Connection::connect().unwrap();
    let i3_conn = Arc::new(Mutex::new(i3_conn)); 
    let is_visible = Arc::new(AtomicBool::new(false));
    let selected_index = Arc::new(AtomicI8::new(-1));

    let is_visible_clone = is_visible.clone();
    let selected_index_clone = selected_index.clone();
    thread::spawn(|| { x11_listener::listen_alt_tab(is_visible_clone, selected_index_clone) });
    
    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(move |app| { 
        ui::setup(app, i3_conn.to_owned(), is_visible.to_owned(), selected_index.to_owned()); 
    });

    application.run();

    Ok(())
}