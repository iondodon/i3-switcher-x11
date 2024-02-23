use std::error::Error;
use std::thread;

mod ui;
mod i3wm;
mod x11;
mod state;
mod screenshot;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    thread::spawn(|| { x11::listener::listen_alt_tab() });

    ui::init();

    Ok(())
}