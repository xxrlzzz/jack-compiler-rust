use log::{Level, LevelFilter, Metadata, SetLoggerError};

use std::str::FromStr;

struct SimpleLogger {
  level: Level,
}
impl SimpleLogger {
  pub fn set_level(&mut self, level: Level) {
    self.level = level;
  }
}
impl log::Log for SimpleLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= self.level
  }
  fn log(&self, rec: &log::Record) {
    if !self.enabled(rec.metadata()) {
      return;
    }
    let log_str = format!(
      "[{}] {}:{} {}",
      rec.level(),
      rec.file().unwrap_or("unknown file"),
      rec.line().unwrap_or(0),
      rec.args()
    );
    println!("{}", log_str)
  }
  fn flush(&self) {}
}

pub fn setup(level_str: &str) -> Result<(), SetLoggerError> {
  let level = Level::from_str(level_str).unwrap_or(Level::Info);
  let level_filter = LevelFilter::from_str(level_str).unwrap_or(LevelFilter::Info);
  static mut LOGGER: SimpleLogger = SimpleLogger { level: Level::Info };
  unsafe {
    LOGGER.set_level(level);
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level_filter))
  }
}
