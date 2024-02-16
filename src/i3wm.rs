use i3ipc::I3Connection;

pub fn focus_workspace(ws_name: String) {
    let mut connection = I3Connection::connect().unwrap();
    let window_id = format!("workspace {}", ws_name);
    connection.run_command(&window_id).unwrap();
}

