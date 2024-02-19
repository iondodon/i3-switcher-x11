use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicI8};
use std::sync::Arc;
use std::thread;
use x11::listener;

mod ui;
mod i3wm;
mod cmd;
mod x11;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let is_visible = Arc::new(AtomicBool::new(false));
    let selected_index = Arc::new(AtomicI8::new(-1));

    let is_visible_clone = is_visible.clone();
    let selected_index_clone = selected_index.clone();
    thread::spawn(|| { listener::listen_alt_tab(is_visible_clone, selected_index_clone) });

    ui::init(is_visible, selected_index);

    Ok(())
}