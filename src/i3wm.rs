use i3ipc::{event::inner::WorkspaceChange, I3EventListener, Subscription};

use crate::state;

pub fn focus_workspace(ws_name: String) {
    let window_id = format!("workspace {}", ws_name);
    let mut i3_conn = state::I3_CONNECTION.write().unwrap();
    i3_conn.run_command(&window_id).unwrap();
}

pub fn listen() {
    let mut listener = I3EventListener::connect().unwrap();

    let subs = [Subscription::Workspace];

    listener.subscribe(&subs).unwrap();

    for event in listener.listen() {
        match event.unwrap() {
            i3ipc::event::Event::WorkspaceEvent(info) => match info.change {
                WorkspaceChange::Init => log::debug!("New workspace {:?}", info),
                WorkspaceChange::Empty => log::debug!("Removed workspace {:?}", info),
                _ => (),
            },
            _ => (),
        }
    }
}
