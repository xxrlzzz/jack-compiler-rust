use crate::common::{new_output, panic_writer, OutputTarget};
use crate::parser::Parser;
use crate::token::Token;
use crate::xml::{translate, RAIIWriter};

pub struct TokenXMLGenerator {
  writer: OutputTarget,
  parser: Parser,
}

impl TokenXMLGenerator {
  pub fn new(source: &str, parser: Parser) -> Self {
    Self {
      writer: new_output(source),
      parser,
    }
  }

  pub fn run(mut self) {
    let _v = RAIIWriter::new_with_newline("tokens".to_string(), self.writer.clone());
    while let Some(token) = self.parser.next() {
      let _v = RAIIWriter::new(token.to_string(), self.writer.clone());
      let content = match token {
        Token::KeyWord(k) => k,
        Token::Symbol(c) => translate(c),
        Token::Identifier(i) => i,
        Token::IntVal(i) => i.to_string(),
        Token::StringVal(s) => s,
        _ => "".to_string(),
      };
      panic_writer(format!(" {} ", content), self.writer.clone().borrow_mut());
    }
  }
}
