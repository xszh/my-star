use std::sync::Arc;

use anyhow::{anyhow, Result};
use ipmb::{label, BytesMessage, EndpointReceiver, EndpointSender, Message, Options, Selector};
use num_enum::TryFromPrimitive;
use parking_lot::Mutex;

use crate::record::ShortRecord;

#[derive(Debug, TryFromPrimitive)]
#[repr(u16)]
pub enum AdapterMessageFormat {
  ControlFlow = 0,
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum ControlFlowCommand {
  Open,
  Close,
  Play,
  Stop,
}

pub struct EndPoint {
  pub sx: Arc<Mutex<EndpointSender<BytesMessage>>>,
  pub rx: EndpointReceiver<BytesMessage>,
  record: Arc<ShortRecord>,
}

impl EndPoint {
  pub fn new() -> Result<Self> {
    let (sx, rx) = ipmb::join::<BytesMessage, BytesMessage>(
      Options::new("mystar.audio.adapter", label!("control", "data"), ""),
      None,
    )?;
    Ok(Self {
      sx: Arc::new(Mutex::new(sx)),
      rx: rx,
      record: Arc::new(ShortRecord::new()?),
    })
  }

  pub fn launch(&mut self) -> Result<()> {
    while let Ok(message) = self.rx.recv(None) {
      let bytes_message: BytesMessage = message.payload;
      if let Ok(fmt) = AdapterMessageFormat::try_from(bytes_message.format) {
        let msg_res = match fmt {
          AdapterMessageFormat::ControlFlow => self.handle_control_flow(&bytes_message.data),
        };
        if let Err(e) = msg_res {
          println!("process message fail: {}", e);
        }
      }
    }
    Ok(())
  }

  fn handle_control_flow(&self, buffer: &[u8]) -> Result<()> {
    match buffer
      .get(0)
      .and_then(|val| ControlFlowCommand::try_from(*val).ok())
      .ok_or(anyhow!("invalid control flow command"))?
    {
      ControlFlowCommand::Open => {
        println!("start open");
        let record = self.record.clone();
        std::thread::spawn(move || {
          if let Err(e) = record.open() {
            eprintln!("open fail: {}", e);
          }
        });
      }
      ControlFlowCommand::Close => {
        println!("start close");
        let record = self.record.clone();
        std::thread::spawn(move || {
          if let Err(e) = record.close() {
            eprintln!("close fail: {}", e);
          }
        });
      }
      ControlFlowCommand::Play => {
        println!("start play");
        let record = self.record.clone();
        std::thread::spawn(move || {
          if let Err(e) = record.start() {
            eprintln!("start fail: {}", e);
          }
        });
      }
      ControlFlowCommand::Stop => {
        println!("stop play");
        let buffer = self.record.stop()?;
        let _sx = self.sx.clone();
        std::thread::spawn(move || {
          if let Err(e) = _sx.lock().send(Message::new(
            Selector::multicast("data"),
            BytesMessage {
              format: 1u16,
              data: buffer.iter().flat_map(|v| v.to_le_bytes()).collect(),
            },
          )) {
            eprintln!("send fail: {}", e);
          }
        });
      }
    };
    Ok(())
  }
}
