use std::error::Error;
use std::thread;
use x11::listener;

mod ui;
mod i3wm;
mod x11;
mod state;
mod screenshot;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    thread::spawn(|| { listener::listen_alt_tab() });

    ui::init();

    Ok(())
}