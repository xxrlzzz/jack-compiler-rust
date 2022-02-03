use log::debug;

static KEYWORDS: &'static [&str] = &[
  "class",
  "constructor",
  "function",
  "method",
  "field",
  "static",
  "var",
  "int",
  "char",
  "boolean",
  "void",
  "true",
  "false",
  "null",
  "this",
  "let",
  "do",
  "if",
  "else",
  "while",
  "return",
];

#[derive(std::cmp::PartialEq, Debug, Clone)]
pub enum Token {
  None,
  KeyWord(String),
  Symbol(char),
  Identifier(String),
  IntVal(u16),
  StringVal(String),
}

fn is_digit(c: char) -> bool {
  return c <= '9' && c >= '0';
}

fn is_alphabet(c: char) -> bool {
  return (c <= 'z' && c >= 'a') || (c <= 'Z' && c >= 'A');
}

fn is_identifier_char(c: char) -> bool {
  is_digit(c) || is_alphabet(c) || c == '_'
}

pub fn is_keyword_constant(keyword: &String) -> bool {
  static KEYWORD_CONSTANTS: &'static [&str] = &["true", "false", "null", "this"];
  return KEYWORD_CONSTANTS.iter().find(|&c| c == keyword).is_some();
}

pub fn is_unary_operation(op: char) -> bool {
  static UNARY_OPERATIONS: &'static [char] = &['-', '~'];
  return UNARY_OPERATIONS.iter().find(|&&c| c == op).is_some();
}

type ResultType = Result<Vec<Token>, String>;

impl Token {
  pub fn from_line(input_line: &str, is_multiline_comment: &mut bool) -> Result<Vec<Self>, String> {
    if input_line.len() == 0 {
      return Ok(vec![]);
    }
    if *is_multiline_comment {
      if let Some(comment_end) = input_line.find("*/") {
        *is_multiline_comment = false;
        return Token::from_line(input_line.split_at(comment_end + 2).1, is_multiline_comment);
      } else {
        return Ok(vec![]);
      }
    }
    let comment_start = input_line.find("//");
    let multiline_comment_start = input_line.find("/*");
    if comment_start.is_some() || multiline_comment_start.is_some() {
      return Token::handle_comment(
        input_line,
        is_multiline_comment,
        comment_start,
        multiline_comment_start,
      );
    }
    Token::from_line_without_comment(input_line)
  }

  fn handle_comment(
    input_line: &str,
    is_multiline_comment: &mut bool,
    comment_start: Option<usize>,
    multiline_comment_start: Option<usize>,
  ) -> Result<Vec<Self>, String> {
    // println!("DEBUG handle comment");
    if comment_start.is_some() && multiline_comment_start.is_some() {
      let comment_start = comment_start.unwrap();
      let multiline_comment_start = multiline_comment_start.unwrap();
      if comment_start < multiline_comment_start {
        return Token::from_line_without_comment(input_line.split_at(comment_start).0);
      } else {
        let (left, right) = input_line.split_at(multiline_comment_start);
        // println!("DEBUG multiline comment {}\t{}", left, right);

        let left = Token::from_line(left, is_multiline_comment);
        *is_multiline_comment = true;
        let right = Token::from_line(right, is_multiline_comment);

        return Token::merge_result(left, right);
      }
    } else if comment_start.is_some() {
      Token::from_line(
        input_line.split_at(comment_start.unwrap()).0,
        is_multiline_comment,
      )
    } else {
      let (left, right) = input_line.split_at(multiline_comment_start.unwrap());

      // println!("DEBUG multiline comment {}\t{}", left, right);
      let left = Token::from_line(left, is_multiline_comment);

      *is_multiline_comment = true;
      let right = Token::from_line(right, is_multiline_comment);

      Token::merge_result(left, right)
    }
  }

  fn from_line_without_comment(input_line: &str) -> Result<Vec<Self>, String> {
    if let Some(str_start_pos) = input_line.find('\"') {
      let (left, right) = input_line.split_at(str_start_pos);
      // println!("DEBUG left of \": {} right of \": {}", left, right);
      if let Some(str_end_pos) = right[1..].find('\"') {
        let left_res = Token::from_line_without_str(left);
        if left_res.is_err() {
          return left_res;
        }
        let (mid, right) = right.split_at(str_end_pos + 2);
        // println!("DEBUG mid {}, right {}", mid, right);
        assert!(mid.len() >= 2);
        let right_res = Token::from_line_without_comment(right);
        if right_res.is_err() {
          return right_res;
        }
        let mut left_res = left_res.unwrap();
        left_res.push(Token::StringVal(mid[1..mid.len() - 1].to_string()));
        left_res.append(&mut right_res.unwrap());
        return Ok(left_res);
      }
      return Err("Invalid string match".to_string());
    }
    Token::from_line_without_str(input_line)
  }

  fn from_line_without_str(input_line: &str) -> Result<Vec<Self>, String> {
    let mut res = vec![];
    for sub_token in input_line.split(' ') {
      debug!("sub token {}", sub_token);
      let token = Token::from_word(sub_token.trim());
      if token.is_err() {
        return token;
      }
      res.append(&mut token.unwrap());
    }
    Ok(res)
  }

  fn from_int(input_word: &str) -> Result<Self, String> {
    let r = input_word.parse::<u16>();
    match r {
      Ok(r) => Ok(Token::IntVal(r)),
      Err(e) => Err(e.to_string()),
    }
  }

  fn from_symbol(input_symbol: char) -> Result<Self, String> {
    static SYMBOLS: &'static [char] = &[
      '(', ')', '{', '}', '[', ']', '.', ',', ';', '+', '-', '*', '/', '&', '|', '<', '>', '=', '~',
    ];
    for s in SYMBOLS {
      if *s == input_symbol {
        return Ok(Token::Symbol(*s));
      }
    }
    return Err(format!("Invalid symbol: {}", input_symbol));
  }

  fn from_word(input_word: &str) -> Result<Vec<Self>, String> {
    let mut res = vec![];
    if input_word.len() == 0 {
      return Ok(res);
    }
    let mut chars = input_word.chars();
    let mut current_char = chars.next();
    let first_char = current_char.unwrap();
    let mut idx = 0;
    if is_digit(first_char) {
      while current_char.is_some() && is_digit(current_char.unwrap()) {
        current_char = chars.next();
        idx += 1;
      }
      let token = Token::from_int(&input_word[0..idx]);
      if token.is_err() {
        return Err(token.unwrap_err());
      }
      res.push(token.unwrap());
    } else if is_identifier_char(first_char) {
      let mut found = false;
      for s in KEYWORDS {
        if input_word.len() >= s.len() && *s == &input_word[0..s.len()] {
          if input_word.len() > s.len() {
            let next_char = input_word.chars().skip(s.len()).next();
            if is_identifier_char(next_char.unwrap()) {
              continue;
            }
          }
          res.push(Token::KeyWord(s.to_string()));
          found = true;
          idx = s.len();
          break;
        }
      }
      if !found {
        while current_char.is_some() && is_identifier_char(current_char.unwrap()) {
          current_char = chars.next();
          idx += 1;
        }
        res.push(Token::Identifier(input_word[0..idx].to_string()));
      }
    } else {
      let token = Token::from_symbol(first_char);
      if token.is_err() {
        return Err(token.unwrap_err());
      }
      idx = 1;
      res.push(token.unwrap());
    }

    if idx < input_word.len() {
      let tokens = Token::from_word(&input_word[idx..]);
      if tokens.is_err() {
        return Err(tokens.unwrap_err());
      }
      res.append(&mut tokens.unwrap());
    }
    return Ok(res);
  }

  fn merge_result(r1: ResultType, r2: ResultType) -> ResultType {
    if r1.is_err() {
      return r1;
    }
    if r2.is_err() {
      return r2;
    }
    let mut ret = r1.unwrap();
    ret.append(&mut r2.unwrap());
    Ok(ret)
  }
}

impl ToString for Token {
  fn to_string(&self) -> String {
    match self {
      Token::KeyWord(_) => "keyword".to_string(),
      Token::Symbol(_) => "symbol".to_string(),
      Token::Identifier(_) => "identifier".to_string(),
      Token::IntVal(_) => "integerConstant".to_string(),
      Token::StringVal(_) => "stringConstant".to_string(),
      _ => "Dump".to_string(),
    }
  }
}
