use adapter::record::short;
use anyhow::{anyhow, Ok, Result};
use tauri::Manager;

fn set_capturing(value: bool, app: tauri::AppHandle) -> anyhow::Result<()> {
  app.emit_all("audio_capture", value)?;
  Ok(())
}

fn set_recording(value: bool, app: tauri::AppHandle) -> anyhow::Result<()> {
  app.emit_all("audio_record", value)?;
  Ok(())
}

pub(crate) fn is_capturing() -> bool {
  short::is_capturing()
}

pub(crate) fn is_recording() -> bool {
  short::is_recording()
}

pub(crate) async fn audio_open(app: tauri::AppHandle) -> anyhow::Result<()> {
  if short::is_capturing() {
    return Ok(());
  }

  short::open()?;
  set_capturing(true, app)?;
  Ok(())
}

pub(crate) async fn audio_close(app: tauri::AppHandle) -> anyhow::Result<()> {
  if !short::is_capturing() {
    return Ok(());
  }

  short::close()?;
  set_capturing(false, app)?;
  Ok(())
}

pub(crate) async fn start_record(app: tauri::AppHandle) -> anyhow::Result<()> {
  if !short::is_capturing() {
    return Err(anyhow!("not capture"));
  }

  if short::is_recording() {
    return Ok(());
  }

  short::start()?;
  set_recording(true, app)?;
  Ok(())
}

pub(crate) fn stop_record(app: tauri::AppHandle) -> Result<Vec<u8>> {
  if !short::is_capturing() {
    return Err(anyhow!("not capture"));
  }
  if !short::is_capturing() {
    return Ok(vec![]);
  }

  let data = short::stop()?;

  println!("data len: {}", data.len());
  set_recording(false, app)?;
  Ok(data.iter().flat_map(|d| d.to_le_bytes()).collect())
}
