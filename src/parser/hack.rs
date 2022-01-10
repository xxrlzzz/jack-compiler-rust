use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::Iterator;

use crate::vm::commands::{Command, CommandType};

pub struct Parser {
  reader: BufReader<File>,
  end: bool,
}

impl Parser {
  pub fn new(source: &str) -> Self {
    // Open the file in read-only mode.
    let file = File::open(source).expect(format!("{} file open failed", source).as_str());
    let reader = BufReader::new(file);
    Self {
      reader: reader,
      end: false,
    }
  }
}

impl Iterator for Parser {
  type Item = Command;

  fn next(&mut self) -> Option<Command> {
    if self.end {
      return None;
    }
    let mut buf = String::new();
    let len = self.reader.read_line(&mut buf).expect("read file failed");
    if len == 0 {
      self.end = true;
      return Some(Command::new(CommandType::None));
    }
    let cmd = buf.trim();
    if cmd.len() == 0 || cmd.starts_with("//") {
      return Some(Command::new(CommandType::None));
    }
    Some(Command::from_str(cmd))
  }
}
