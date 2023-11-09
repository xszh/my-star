// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;

use adapter;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn start_record() {
    let end_point = adapter::ffi::ipmb::EndPoint::new();
}
