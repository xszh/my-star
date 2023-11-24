use chrono::{DateTime, Local};
use log::{LevelFilter, Record};
use log4rs::{
  append::{console::ConsoleAppender, Append},
  config::{Appender, Root},
  encode::{self, pattern::PatternEncoder, Encode},
  Config, Handle,
};
use parking_lot::Mutex;
// use parking_lot::Mutex;
use std::{
  fmt::Display,
  fs::{self, File, OpenOptions},
  io::{self, BufWriter, Write},
  path::{Path, PathBuf},
  time::SystemTime,
};

#[derive(Debug)]
struct LogWriter {
  file: BufWriter<File>,
  len: u64,
}

mod env_util {
  const ENV_PREFIX: &str = "$ENV{";
  const ENV_PREFIX_LEN: usize = ENV_PREFIX.len();
  const ENV_SUFFIX: char = '}';
  const ENV_SUFFIX_LEN: usize = 1;

  fn is_env_var_start(c: char) -> bool {
    // Close replacement for old [\w]
    // Note that \w implied \d and '_' and non-ASCII letters/digits.
    c.is_alphanumeric() || c == '_'
  }

  fn is_env_var_part(c: char) -> bool {
    // Close replacement for old [\w\d_.]
    c.is_alphanumeric() || c == '_' || c == '.'
  }

  pub fn expand_env_vars(path: std::path::PathBuf) -> std::path::PathBuf {
    let path: String = path.to_string_lossy().into();
    let mut outpath: String = path.clone();
    for (match_start, _) in path.match_indices(ENV_PREFIX) {
      let env_name_start = match_start + ENV_PREFIX_LEN;
      let (_, tail) = path.split_at(env_name_start);
      let mut cs = tail.chars();
      // Check first character.
      if let Some(ch) = cs.next() {
        if is_env_var_start(ch) {
          let mut env_name = String::new();
          env_name.push(ch);
          // Consume following characters.
          let valid = loop {
            match cs.next() {
              Some(ch) if is_env_var_part(ch) => env_name.push(ch),
              Some(ENV_SUFFIX) => break true,
              _ => break false,
            }
          };
          // Try replacing properly terminated env var.
          if valid {
            if let Ok(env_value) = std::env::var(&env_name) {
              let match_end = env_name_start + env_name.len() + ENV_SUFFIX_LEN;
              // This simply rewrites the entire outpath with all instances
              // of this var replaced. Could be done more efficiently by building
              // `outpath` as we go when processing `path`. Not critical.
              outpath = outpath.replace(&path[match_start..match_end], &env_value);
            }
          }
        }
      }
    }
    outpath.into()
  }
}

impl io::Write for LogWriter {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.file.write(buf).map(|n| {
      self.len += n as u64;
      n
    })
  }

  fn flush(&mut self) -> io::Result<()> {
    self.file.flush()
  }
}

impl encode::Write for LogWriter {}

#[derive(Debug)]
/// An appender which archives log files in a configurable strategy.
pub struct DailyFileAppender {
  writer: Mutex<Option<LogWriter>>,
  path: PathBuf,
  time: Mutex<SystemTime>,
  append: bool,
  encoder: Box<dyn Encode>,
}

impl Append for DailyFileAppender {
  fn append(&self, record: &Record) -> anyhow::Result<()> {
    let mut writer = self.writer.lock();

    let last_time = DateTime::<Local>::from(*self.time.lock());
    let last_time_str = last_time.format("%Y%m%d").to_string();
    let current_time_str = Local::now().format("%Y%m%d").to_string();

    if current_time_str != last_time_str {
      *writer = None;
      *self.time.lock() = SystemTime::now();
    }

    let writer = self.get_writer(&mut writer)?;
    self.encoder.encode(writer, record)?;
    writer.flush()?;

    Ok(())
  }

  fn flush(&self) {}
}

impl DailyFileAppender {
  /// Creates a new `RollingFileAppenderBuilder`.
  pub fn builder() -> DailyFileAppenderBuilder {
    DailyFileAppenderBuilder {
      append: true,
      encoder: None,
    }
  }

  fn get_writer<'a>(&self, writer: &'a mut Option<LogWriter>) -> io::Result<&'a mut LogWriter> {
    if writer.is_none() {
      use chrono::prelude::*;
      let dt: DateTime<Local> = (*self.time.lock()).into();
      let time = dt.format("%Y%m%d");
      let path = self
        .path
        .to_string_lossy()
        .to_string()
        .replace("{}", time.to_string().as_str());
      let file = OpenOptions::new()
        .write(true)
        .append(self.append)
        .truncate(!self.append)
        .create(true)
        .open(&path)?;
      let len = if self.append {
        file.metadata()?.len()
      } else {
        0
      };
      *writer = Some(LogWriter {
        file: BufWriter::with_capacity(1024, file),
        len,
      });
    }

    // :( unwrap
    Ok(writer.as_mut().unwrap())
  }
}

/// A builder for the `RollingFileAppender`.
pub struct DailyFileAppenderBuilder {
  append: bool,
  encoder: Option<Box<dyn Encode>>,
}

impl DailyFileAppenderBuilder {
  /// Determines if the appender will append to or truncate the log file.
  ///
  /// Defaults to `true`.
  pub fn append(mut self, append: bool) -> DailyFileAppenderBuilder {
    self.append = append;
    self
  }

  /// Sets the encoder used by the appender.
  ///
  /// Defaults to a `PatternEncoder` with the default pattern.
  pub fn encoder(mut self, encoder: Box<dyn Encode>) -> DailyFileAppenderBuilder {
    self.encoder = Some(encoder);
    self
  }

  /// Constructs a `RollingFileAppender`.
  /// The path argument can contain environment variables of the form $ENV{name_here},
  /// where 'name_here' will be the name of the environment variable that
  /// will be resolved. Note that if the variable fails to resolve,
  /// $ENV{name_here} will NOT be replaced in the path.
  pub fn build<P>(self, path: P) -> io::Result<DailyFileAppender>
  where
    P: AsRef<Path>,
  {
    let path = env_util::expand_env_vars(path.as_ref().to_path_buf());
    let appender = DailyFileAppender {
      writer: Mutex::new(None),
      path,
      append: self.append,
      time: Mutex::new(SystemTime::now()),
      encoder: self
        .encoder
        .unwrap_or_else(|| Box::new(PatternEncoder::default())),
    };

    if let Some(parent) = appender.path.parent() {
      fs::create_dir_all(parent)?;
    }

    // open the log file immediately
    appender.get_writer(&mut appender.writer.lock())?;

    Ok(appender)
  }
}

pub fn init_log<P: AsRef<Path>>(path: P) -> anyhow::Result<Handle> {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "{d(%Y-%m-%d %H:%M:%S)} -{l}- [{P}-{i}] [{M}] {m}{n}",
    )))
    .build();

  let daily_file = DailyFileAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "{d(%Y-%m-%d %H:%M:%S)} -{l}- [{P}-{i}] [{M}] {m}{n}",
    )))
    .build(path)?;

  let config = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .appender(Appender::builder().build("logfile", Box::new(daily_file)))
    .build(
      Root::builder()
        .appender("stdout")
        .appender("logfile")
        .build(LevelFilter::Info),
    )?;

  Ok(log4rs::init_config(config)?)
}
