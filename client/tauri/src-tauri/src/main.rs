// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(lazy_cell)]

mod audio;

#[tauri::command]
async fn is_recording() -> Result<bool, ()> {
  Ok(audio::is_recording())
}

#[tauri::command]
async fn start_record(app: tauri::AppHandle) -> Result<(), String> {
  audio::start_record(app).map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_record(app: tauri::AppHandle) -> Result<Vec<u8>, String> {
  audio::stop_record(app).map_err(|e| e.to_string())
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      is_recording,
      start_record,
      stop_record
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
