#![feature(result_option_inspect)]

use anyhow::Result;

pub mod player;
pub mod record;
pub mod resample;
pub mod ffi;

pub trait Adapter {
  fn start_record() -> Result<()>;
  fn stop_record() -> Result<()>;
}