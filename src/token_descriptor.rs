use crate::token::Token;

#[derive(std::cmp::PartialEq, Debug, Clone)]
pub struct TokenDescriptor {
  token: Token,
  file: String,
  line: usize,
  position: usize,
}

impl TokenDescriptor {
  pub fn new(token: Token, file: String, line: usize, position: usize) -> Self {
    Self {
      token,
      file,
      line,
      position,
    }
  }
}
