use gtk::prelude::{ApplicationExt, ApplicationExtManual, ButtonExt, ContainerExt, WidgetExt};
use gtk::{Application, ApplicationWindow, Button};
use i3ipc::{reply::NodeType, I3Connection};
use i3ipc::reply::Node;
use core::time;
use std::error::Error;
use std::{self, thread};
use tokio::try_join;

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
        thread::sleep(time::Duration::from_secs(2));
    }

    // Recurse into this node's children and floating nodes
    for child in &node.nodes {
        print_window_names(child);
    }
    for floating in &node.floating_nodes {
        print_window_names(floating);
    }
}

async fn tray() {
    print!("HELLOOOO");
}

async fn ui() {
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
        window.add(&button);

        window.show_all();
    });

    application.run();
}

async fn logic() {
    // Establish a connection to the i3 IPC interface
    let mut connection = I3Connection::connect().unwrap();

    // Query the layout tree
    let tree = connection.get_tree().unwrap();

    // Recursively print the names of all windows
    print_window_names(&tree);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let jh1 = tokio::spawn(tray());
    let jh2 = tokio::spawn(ui());
    let jh3 = tokio::spawn(logic());

    try_join!(jh1, jh2, jh3)?;

    Ok(())
}