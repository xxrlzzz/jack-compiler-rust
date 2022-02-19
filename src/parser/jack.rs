use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::Iterator;

use crate::token::{Token, TokenDescriptor};

pub struct Parser {
  reader: BufReader<File>,
  source: String,
  end: bool,
  token_buf: Vec<Token>,
  token_buf_idx: usize,
  cur_line: usize,
  last_token_descriptor: Option<TokenDescriptor>,
}

impl Parser {
  pub fn new(source: &str) -> Self {
    let file = File::open(source).expect(format!("{} file open failed", source).as_str());
    let reader = BufReader::new(file);
    Self {
      reader,
      source: source.to_string(),
      end: false,
      token_buf: vec![],
      token_buf_idx: 0,
      cur_line: 0,
      last_token_descriptor: None,
    }
  }

  fn update_token_descriptor(&mut self, token: &Token) {
    // TODO: Extract position of token in line.
    let dec = TokenDescriptor::new(token.clone(), self.source.clone(), self.cur_line, 0);
    self.last_token_descriptor = Some(dec);
  }

  fn return_token_from_buf(&mut self, take: bool) -> Option<Token> {
    let r = self.token_buf[self.token_buf_idx].clone();
    self.update_token_descriptor(&r);
    if take {
      self.token_buf_idx += 1;
    }
    return Some(r);
  }

  fn read_line(&mut self, buf: &mut String) -> bool {
    let len = self.reader.read_line(buf).expect("read file failed");
    self.cur_line += 1;
    if len == 0 {
      self.end = true;
      return false;
    }
    return true;
  }

  fn forward(&mut self, take: bool) -> Option<Token> {
    if self.token_buf.len() > self.token_buf_idx {
      return self.return_token_from_buf(take);
    }
    if self.end {
      return None;
    }
    let mut buf = String::new();
    let mut is_multiline_comment = false;
    loop {
      buf.clear();
      if !self.read_line(&mut buf) {
        return None;
      }
      let line = buf.trim();
      let tokens = Token::from_line(line, &mut is_multiline_comment);
      if tokens.is_err() {
        panic!("{}", tokens.unwrap_err());
      }
      self.token_buf = tokens.unwrap();
      if self.token_buf.len() == 0 {
        continue;
      }
      self.token_buf_idx = 0;
      return self.return_token_from_buf(take);
    }
  }

  pub fn peek(&mut self) -> Option<Token> {
    self.forward(false)
  }

  pub fn take_if<P>(&mut self, mut predicate: P) -> Option<Token>
  where
    P: FnMut(&Token) -> bool,
  {
    let token = self.peek();
    if let Some(token) = token {
      if predicate(&token) {
        return self.next();
      }
    }
    None
  }

  pub fn get_last_token_descriptor(&self) -> TokenDescriptor {
    self.last_token_descriptor.clone().unwrap()
  }
}

impl Iterator for Parser {
  type Item = Token;

  fn next(&mut self) -> Option<Token> {
    self.forward(true)
  }
}
