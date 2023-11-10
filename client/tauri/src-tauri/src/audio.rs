use std::sync::{Arc, Mutex};

use adapter::ffi::ipmb::EndPoint;

const RECORDING: Arc<Mutex<bool>> = Arc::default();

#[tauri::command]
fn start_record() -> Result<(), String> {}
