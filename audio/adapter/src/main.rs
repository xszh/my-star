use std::{fs, time::Duration};

use adapter::record::ShortRecord;
use byteorder::{BigEndian, WriteBytesExt};

fn main() {
  let mut short_record = ShortRecord::new().expect("new short record");
  short_record.start().expect("start success");
  std::thread::sleep(Duration::from_secs(3));
  let output = short_record.stop().expect("stop success");
  println!("output len {}", output.len());

  let mut file = fs::OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open("./output.pcm")
    .expect("create/open output pcm fail");

  for item in output.iter() {
    file.write_i16::<BigEndian>(*item).expect("write file fail");
  }
}
