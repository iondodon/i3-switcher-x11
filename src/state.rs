use std::sync::{
    atomic::{AtomicBool, AtomicI8},
    RwLock,
};

use i3ipc::I3Connection;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref I3_CONNECTION: RwLock<I3Connection> =
        RwLock::new(I3Connection::connect().expect("Failed to connect to i3"));
    pub static ref IS_VISIBLE: AtomicBool = AtomicBool::new(false);
    pub static ref SHOULD_SWITCH: AtomicBool = AtomicBool::new(false);
    pub static ref SELECTED_INDEX: AtomicI8 = AtomicI8::new(-1);
    pub static ref SELECTED_INDEX_CHANGED: AtomicBool = AtomicBool::new(false);
    pub static ref FOCUSED_TAB_NAME: RwLock<Option<String>> = RwLock::new(None);
    pub static ref CURRENT_WS_NAME: RwLock<Option<String>> = RwLock::new(None);
}
