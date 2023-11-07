use std::sync::Arc;

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample};
use crossbeam::channel::{unbounded, Receiver, Sender};
use parking_lot::Mutex;
use rubato::Resampler;

use crate::resample;

pub struct ShortPlayer {
  pub device: cpal::Device,
  pub config: cpal::SupportedStreamConfig,
  pub buffer: Vec<i16>,
  pub sample_rate: u16,
  pub sx: Arc<Mutex<Sender<()>>>,
  pub rx: Receiver<()>,
}

impl ShortPlayer {
  pub fn new(data: &[i16], sample_rate: u16) -> Result<Self> {
    let device = cpal::default_host()
      .default_output_device()
      .ok_or(anyhow!("no output device found"))?;

    let config = device.default_output_config()?;

    let (sx, rx) = unbounded::<()>();

    Ok(Self {
      device,
      config,
      buffer: data.to_vec(),
      sample_rate,
      sx: Arc::new(Mutex::new(sx)),
      rx,
    })
  }

  pub fn play(&self) -> Result<()> {
    let buffer_f32 = self
      .buffer
      .iter()
      .map(|d| f32::from_sample(*d))
      .collect::<Vec<f32>>();
    let mut resampler = resample::resample::<f32>(
      self.sample_rate as _,
      self.config.clone().sample_rate().0 as _,
      buffer_f32.len(),
    )?;

    let mut output = resampler.output_buffer_allocate(true);
    resampler.process_into_buffer(&vec![buffer_f32], &mut output, None)?;

    let output = output.pop().ok_or(anyhow!("cannot get first channel"))?;

    let stream = match self.config.sample_format() {
      cpal::SampleFormat::I8 => self.run::<i8>(output),
      cpal::SampleFormat::I16 => self.run::<i16>(output),
      cpal::SampleFormat::I32 => self.run::<i32>(output),
      cpal::SampleFormat::I64 => self.run::<i64>(output),
      cpal::SampleFormat::U8 => self.run::<u8>(output),
      cpal::SampleFormat::U16 => self.run::<u16>(output),
      cpal::SampleFormat::U32 => self.run::<u32>(output),
      cpal::SampleFormat::U64 => self.run::<u64>(output),
      cpal::SampleFormat::F32 => self.run::<f32>(output),
      cpal::SampleFormat::F64 => self.run::<f64>(output),
      e => Err(anyhow!("Unsupported sample format '{e}'")),
    }?;
    
    stream.play()?;

    loop {
      let res = self.rx.try_recv();
      if let Err(e) = res {
        if e.is_empty() {
          continue;
        }
      }
      break;
    }

    drop(stream);

    Ok(())
  }

  fn run<F>(&self, output: Vec<f32>) -> Result<cpal::Stream>
  where
    F: SizedSample + FromSample<f32>,
  {
    let channels = self.config.clone().channels() as usize;
    let mut cursor: usize = 0;
    let sx_c = self.sx.clone();
    let mut next_value = move || {
      if cursor < output.len() {
        cursor += 1;
        return output[cursor - 1];
      } else {
        if let Err(e) = sx_c.lock().send(()) {
          println!("send fail: {}", e);
        }
        return 0f32;
      }
    };
    let stream = self.device.build_output_stream(
      &self.config.clone().into(),
      move |buffer: &mut [F], _: &_| {
        Self::write_data(buffer, channels, &mut next_value);
      },
      |e| println!("output stream error: {}", e),
      None,
    )?;
    Ok(stream)
  }

  fn write_data<F>(output: &mut [F], channels: usize, next_sample: &mut dyn FnMut() -> f32)
  where
    F: SizedSample + FromSample<f32>,
  {
    for frame in output.chunks_mut(channels) {
      let value: F = F::from_sample(next_sample());
      for sample in frame.iter_mut() {
        *sample = value;
      }
    }
  }
}
