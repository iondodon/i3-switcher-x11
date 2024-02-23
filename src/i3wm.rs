use crate::state;

pub fn focus_workspace(ws_name: String) {
    let mut i3_conn = state::I3_CONNECTION.write().unwrap();
    let window_id = format!("workspace {}", ws_name);
    i3_conn.run_command(&window_id).unwrap();
}

