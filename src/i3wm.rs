use crate::state;

pub fn focus_workspace(ws_name: String) {
    let window_id = format!("workspace {}", ws_name);
    let mut i3_conn = state::I3_CONNECTION.write().unwrap();
    i3_conn.run_command(&window_id).unwrap();
}

