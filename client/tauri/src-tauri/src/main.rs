// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(lazy_cell)]
#![feature(async_closure)]

mod asr;
mod audio;

#[tauri::command]
async fn is_recording() -> Result<bool, ()> {
  Ok(audio::is_recording())
}

#[tauri::command]
async fn is_capturing() -> Result<bool, ()> {
  Ok(audio::is_capturing())
}

#[tauri::command]
async fn start_record(app: tauri::AppHandle) -> Result<(), String> {
  audio::start_record(app).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn audio_open(app: tauri::AppHandle) -> Result<(), String> {
  audio::audio_open(app).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn audio_close(app: tauri::AppHandle) -> Result<(), String> {
  audio::audio_close(app).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_record(app: tauri::AppHandle) -> Result<Vec<u8>, String> {
  audio::stop_record(app).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_asr(app: tauri::AppHandle) -> Result<(), String> {
  asr::start_asr(app).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_asr(app: tauri::AppHandle, token: String, app_id: String) -> Result<String, String> {
  asr::stop_asr(app, token, app_id)
    .await
    .map_err(|e| e.to_string())
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      is_recording,
      is_capturing,
      start_record,
      stop_record,
      start_asr,
      stop_asr,
      audio_open,
      audio_close
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
