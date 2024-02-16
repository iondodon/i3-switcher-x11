use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::Application;
use i3ipc::event::inner::WorkspaceChange;
use i3ipc::event::{Event, WorkspaceEventInfo};
use i3ipc::{I3Connection, I3EventListener, Subscription};
use std::error::Error;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicI8};
use std::sync::{Arc, Mutex};
use std::thread;

mod x11_listener;
mod ui;
mod i3wm;

fn capture_screenshot(workspace_name: &str) {
    println!("Capturing screenshot of workspace: {}", workspace_name);
    let filename = format!("/tmp/i3-switcher-x11/{}.png", workspace_name);
    Command::new("rm")
        .arg(&filename)
        .output()
        .expect("Failed to remove screenshot");
    Command::new("scrot")
        .arg(&filename)
        .output()
        .expect("Failed to capture screenshot");
    println!("Screenshot saved to {}", filename);
}

fn main() -> Result<(), Box<dyn Error>> {
    let i3_conn = I3Connection::connect().unwrap();
    let i3_conn = Arc::new(Mutex::new(i3_conn)); 
    let is_visible = Arc::new(AtomicBool::new(false));
    let selected_index = Arc::new(AtomicI8::new(-1));

    thread::spawn(|| {
        let mut listener = I3EventListener::connect().unwrap();
        let _ = listener.subscribe(&[Subscription::Workspace]).unwrap();
    
        for event in listener.listen() {
            if let Ok(Event::WorkspaceEvent(WorkspaceEventInfo { change: WorkspaceChange::Focus, current, old, .. })) = event {
                if let Some(old_workspace) = old {
                    if let Some(name) = &old_workspace.name {
                        capture_screenshot(name);
                    }
                }
    
                if let Some(new_workspace) = current {
                    println!("Switched to workspace: {}", new_workspace.name.unwrap());
                }
            }
        }
    });


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