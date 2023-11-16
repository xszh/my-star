use std::{
  default,
  sync::{Arc, LazyLock},
  time::{Duration, Instant},
};

use crate::resample;
use anyhow::{anyhow, Result};
use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  FromSample, Sample, SampleFormat, SizedSample, SupportedStreamConfig,
};
use crossbeam::channel::{unbounded, Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use rubato::Resampler;

pub enum ShortRecordChannel {
  Start,
  Stop,
  Close,
}

struct ShortRecord {
  buffer: Arc<RwLock<Option<Vec<f32>>>>,
  ch_cmd: (Sender<ShortRecordChannel>, Receiver<ShortRecordChannel>),
  ch_data: (Sender<Vec<i16>>, Receiver<Vec<i16>>),
  recording: bool,
  capturing: bool,
}

type TShortRecord = Arc<RwLock<ShortRecord>>;

fn get_recorder() -> &'static LazyLock<TShortRecord> {
  static RECORDER: LazyLock<TShortRecord> = LazyLock::new(|| {
    Arc::new(RwLock::new(ShortRecord {
      buffer: Arc::new(RwLock::new(None)),
      ch_cmd: unbounded::<ShortRecordChannel>(),
      ch_data: unbounded::<Vec<i16>>(),
      recording: false,
      capturing: false,
    }))
  });
  &RECORDER
}

pub fn is_recording() -> bool {
  get_recorder().read().recording
}

pub fn is_capturing() -> bool {
  get_recorder().read().capturing
}

pub fn capturing(rx: &Receiver<ShortRecordChannel>) -> Result<()> {
  let host = cpal::default_host();
  let device = host.default_input_device().ok_or(anyhow!(""))?;
  let raw_config = device.default_input_config()?;
  let config = raw_config.clone();

  let stream = match config.sample_format() {
    SampleFormat::I16 => run::<i16>(&device, &config),
    SampleFormat::I32 => run::<i32>(&device, &config),
    SampleFormat::I64 => run::<i64>(&device, &config),
    SampleFormat::U8 => run::<u8>(&device, &config),
    SampleFormat::U16 => run::<u16>(&device, &config),
    SampleFormat::U32 => run::<u32>(&device, &config),
    SampleFormat::U64 => run::<u64>(&device, &config),
    SampleFormat::F32 => run::<f32>(&device, &config),
    SampleFormat::F64 => run::<f64>(&device, &config),
    s => Err(anyhow!("unsupported sample format: {}", s)),
  }?;

  get_recorder().write().capturing = true;
  stream.play()?;

  loop {
    match rx.try_recv() {
      Ok(msg) => match msg {
        ShortRecordChannel::Start => {
          {
            let mut recorder = get_recorder().write();
            recorder.recording = true;
            *recorder.buffer.write() = Some(vec![]);
          }
          continue;
        }
        ShortRecordChannel::Stop => {
          println!("audio ctrl stop");
          get_recorder().write().recording = false;

          let buf = get_recorder()
            .read()
            .buffer
            .write()
            .take()
            .ok_or(anyhow!("stop while no buffer"))?;

          let config = raw_config.clone();
          std::thread::spawn(move || -> Result<()> {
            get_recorder()
              .read()
              .ch_data
              .0
              .send(match finalized_data(&config, &buf) {
                Ok(data) => data,
                Err(e) => {
                  eprintln!("finalize data fail: {}", e);
                  Vec::<i16>::new()
                }
              })?;

            get_recorder().write().recording = false;
            Ok(())
          });
          continue;
        }
        ShortRecordChannel::Close => {
          break;
        }
      },
      Err(e) => {
        if e.is_empty() {
          continue;
        } else {
          eprintln!("try recv error: {e}");
          break;
        }
      }
    }
  }

  drop(stream);
  get_recorder().write().capturing = false;
  Ok(())
}

fn finalized_data(config: &SupportedStreamConfig, buf: &[f32]) -> Result<Vec<i16>> {
  let mut resampler =
    resample::resample::<f32>(config.sample_rate().0 as f64, 16000f64, buf.len())?;
  let mut out = resampler.output_buffer_allocate(true);
  resampler.process_into_buffer(&vec![buf], &mut out, None)?;
  let out = out
    .get(0)
    .ok_or(anyhow!("get first channel fail"))?
    .to_vec();

  println!("finalizing data. len: {}", out.len());
  Ok(
    out
      .into_iter()
      .map(|item| i16::from_sample(item))
      .collect::<Vec<i16>>(),
  )
}

pub fn open() -> Result<()> {
  let ctrl = get_recorder().read().ch_cmd.clone();
  std::thread::spawn(move || {
    capturing(&ctrl.1).expect("??");
  });

  Ok(())
}

pub fn close() -> Result<()> {
  get_recorder()
    .read()
    .ch_cmd
    .0
    .send(ShortRecordChannel::Close)?;
  Ok(())
}

pub fn start() -> Result<()> {
  if get_recorder().read().recording {
    return Ok(());
  }
  println!("short::start");
  get_recorder()
    .read()
    .ch_cmd
    .0
    .send(ShortRecordChannel::Start)?;
  Ok(())
}

pub fn stop() -> Result<Vec<i16>> {
  if !get_recorder().read().recording {
    return Ok(vec![]);
  }
  println!("short::stop");
  get_recorder()
    .read()
    .ch_cmd
    .0
    .send(ShortRecordChannel::Stop)?;

  // println!("short::stop sent");
  let final_data = get_recorder()
    .read()
    .ch_data
    .1
    .recv_timeout(Duration::from_secs(5))?;

  Ok(final_data)

  // loop {
  //   match get_recorder().read().ch_data.1.try_recv() {
  //     Ok(d) => {
  //       return Ok(d);
  //     }
  //     Err(e) => {
  //       if e.is_empty() {
  //         continue;
  //       }
  //       return Err(anyhow!("recv error"));
  //     }
  //   }
  // }
}

fn run<F>(device: &cpal::Device, config: &cpal::SupportedStreamConfig) -> Result<cpal::Stream>
where
  F: SizedSample,
  f32: FromSample<F>,
{
  let channel = config.clone().channels();
  let rec = get_recorder().clone();
  Ok(device.build_input_stream(
    &config.clone().into(),
    move |data: &[F], _: &_| {
      if rec.read().recording {
        let rec = rec.read();
        let mut buf = rec.buffer.write();
        if let Some(mut buffer) = buf.as_mut() {
          write_data(data, &mut buffer, channel);
        }
      }
    },
    |e| println!("input stream fail: {}", e),
    None,
  )?)
}

fn write_data<F>(data: &[F], buffer: &mut Vec<f32>, channel: u16)
where
  F: SizedSample,
  f32: FromSample<F>,
{
  let mut idx: usize = 0;
  let mut c_idx: u16 = 1;
  let mut d: f32 = 0f32;
  while idx < data.len() {
    let x = f32::from_sample(data[idx]) / (channel as f32);
    d += x;
    if c_idx == channel {
      buffer.push(d);
      d = 0f32;
      c_idx = 1;
    } else {
      c_idx += 1;
    }
    idx += 1;
  }
}

#[test]
fn test() {
  open().expect("open");
  std::thread::sleep(Duration::from_secs(1));
  start().expect("start");
  std::thread::sleep(Duration::from_secs(3));
  let d = stop().expect("stop");
  println!("d len: {}", d.len());
  std::thread::sleep(Duration::from_secs(1));
  close().expect("close");
}
