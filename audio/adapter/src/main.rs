use anyhow::Result;
use std::{
  fs::{self, read},
  io::{Cursor, Read},
  time::Duration,
};

use adapter::{ffi, player::ShortPlayer, record};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

fn launch_ipmb() -> Result<()> {
  let mut end_point = crate::ffi::ipmb::EndPoint::new()?;
  end_point.launch()?;

  Ok(())
}

fn main() {
  // quick_play().expect("play success");
  launch_ipmb().expect("launch ipmb");
}
