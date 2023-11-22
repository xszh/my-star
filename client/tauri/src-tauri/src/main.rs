// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(lazy_cell)]
#![feature(async_closure)]

use std::time::Duration;

use log::{info, error};
use mystar_core::{
  logger::init_log,
  net::{WSMessage, WSClientCtrl},
};
use tauri::Manager;
use tokio::sync::{broadcast, mpsc::unbounded_channel};

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

#[tauri::command]
async fn get_device_id() -> Result<String, String> {
  use mac_address;
  let m_a = mac_address::get_mac_address().map_err(|e| e.to_string())?;
  let m_a = m_a.ok_or("no mac address".to_string())?.to_string();
  let did = md5::compute(m_a);
  let did = format!("{:x}", did);
  Ok(did)
}

fn main() {
  init_log("./log/tauri-{}.log").unwrap();

  tauri::Builder::default()
    .setup(|app| {
      let (tx_cmd, rx_cmd) = unbounded_channel::<WSClientCtrl>();
      let (tx_msg, mut rx_msg) = broadcast::channel::<Message>(16);

      let handle = app.handle();

      tauri::async_runtime::spawn(async move {
        tokio::spawn(mystar_core::net::run(
          "wss://www.miemie.tech/mystar/ws/",
          rx_cmd,
          tx_msg,
          Duration::from_secs(5),
          None,
        ));

        tokio::spawn(async move {
          let handle_msg = |msg: WSMessage| {
            match msg {
              WSMessage::Text(text) => {
                if let Err(e) = handle.emit_all("ws_text", text) {
                  error!("emit all failed: {}", e);
                }
              },
              WSMessage::Binary(bin) => {
                if let Err(e) = handle.emit_all("ws_text", bin) {
                  error!("emit all failed: {}", e);
                }
              },
              WSMessage::Close() => {
                info!("receive close");
              },
              WSMessage::Open() => {

              }
            }
          };
          loop {
            tokio::select! {
              Ok(msg) = rx_msg.recv() => handle_msg(msg)
            }
          }
        })
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      is_recording,
      is_capturing,
      start_record,
      stop_record,
      start_asr,
      stop_asr,
      audio_open,
      audio_close,
      get_device_id
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
