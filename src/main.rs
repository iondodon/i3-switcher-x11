use i3ipc::{reply::NodeType, I3Connection};
use i3ipc::reply::Node;
use core::time;
use std::{self, thread};

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

fn main() {
    // Establish a connection to the i3 IPC interface
    let mut connection = I3Connection::connect().unwrap();

    // Query the layout tree
    let tree = connection.get_tree().unwrap();

    // Recursively print the names of all windows
    print_window_names(&tree);
}