use anyhow::Result;
use std::{
  fs::{self, read},
  io::{Cursor, Read},
  time::Duration,
};

use adapter::{record::ShortRecord, player::ShortPlayer, ffi};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

fn quick_record() -> Result<()> {
  let mut short_record = ShortRecord::new().expect("new short record");
  short_record.start().expect("start success");
  std::thread::sleep(Duration::from_secs(3));
  let output = short_record.stop().expect("stop success");
  println!("output len {}", output.len());

  let mut file = fs::OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open("./output.pcm")?;

  for item in output.iter() {
    file.write_i16::<BigEndian>(*item).expect("write file fail");
  }

  Ok(())
}

fn quick_play() -> Result<()> {
  let mut file = fs::OpenOptions::new().read(true).open("./output.pcm")?;
  let mut output: Vec<i16> = vec![];

  loop {
    match file.read_i16::<BigEndian>() {
      Ok(d) => output.push(d),
      Err(e) => {
        match e.kind() {
          std::io::ErrorKind::UnexpectedEof => {},
          _ => println!("read file fail: {}", e),
        }
        break;
      }
    }
  }

  let player = ShortPlayer::new(&output, 16000)?;
  player.play()?;

  Ok(())
}

fn start_endpoint() -> Result<()> {
  let mut ep = ffi::ipmb::EndPoint::new()?;
  ep.launch()?;

  Ok(())
}

fn launch_ipmb() -> Result<()> {
  let mut end_point = crate::ffi::ipmb::EndPoint::new()?;
  end_point.launch()?;

  Ok(())
}

fn main() {
  // quick_play().expect("play success");
  launch_ipmb().expect("launch ipmb");
}
