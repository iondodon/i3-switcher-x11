use std::{collections::HashMap, sync::{atomic::{AtomicBool, AtomicI8}, RwLock}};

use i3ipc::I3Connection;
use image::{ImageBuffer, Rgba};
use lazy_static::lazy_static;

lazy_static! {
    static ref i3_conn: I3Connection = I3Connection::connect().unwrap();
    
    
    pub static ref I3_CONNECTION: RwLock<I3Connection> = RwLock::new(i3_conn);

    pub static ref IS_VISIBLE: AtomicBool = AtomicBool::new(false);
    pub static ref SELECTED_INDEX: AtomicI8 = AtomicI8::new(-1);
    pub static ref SCREENSHOTS: RwLock<HashMap<String, Option<ImageBuffer<Rgba<u8>, Vec<u8>>>>> = RwLock::new(HashMap::new());
    pub static ref FOCUSED_WS_NAME: RwLock<Option<String>> = RwLock::new(None);
    pub static ref CURRENT_WS_NAME: RwLock<Option<String>> = RwLock::new(None);
}