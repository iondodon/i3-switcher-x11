use std::sync::{Arc, RwLock};

use i3ipc::I3Connection;

pub fn focus_workspace(ws_name: String, i3_conn: Arc<RwLock<I3Connection>>) {
    let mut i3_conn = i3_conn.write().unwrap();
    let window_id = format!("workspace {}", ws_name);
    i3_conn.run_command(&window_id).unwrap();
}

