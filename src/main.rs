use std::error::Error;
use std::thread;

mod i3wm;
mod screenshot;
mod state;
mod ui;
mod x11;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    thread::spawn(|| x11::listener::listen_alt_tab());

    ui::init();

    Ok(())
}
