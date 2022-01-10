#[derive(std::cmp::PartialEq, Debug, Clone)]
pub enum CommandType {
  None,
  Arithmetic(OperandNum),
  Push,
  Pop,
  Label,
  Goto,
  If,
  Function,
  Return,
  Call,
}
#[derive(std::cmp::PartialEq, Debug, Clone)]
pub enum OperandNum {
  TwoOperand,
  SingleOperand,
}

impl CommandType {
  pub fn parse(cmd: &str) -> (Self, u8) {
    return match cmd {
      "add" | "sub" | "eq" | "gt" | "lt" | "and" | "or" => {
        (CommandType::Arithmetic(OperandNum::TwoOperand), 1)
      }
      "neg" | "not" => (CommandType::Arithmetic(OperandNum::SingleOperand), 1),
      "push" => (CommandType::Push, 2),
      "pop" => (CommandType::Pop, 2),
      "label" => (CommandType::Label, 1),
      "goto" => (CommandType::Goto, 1),
      "if-goto" => (CommandType::If, 1),
      "function" => (CommandType::Function, 2),
      "call" => (CommandType::Call, 2),
      "return" => (CommandType::Return, 0),
      _ => (CommandType::None, 0),
    };
  }
}

#[derive(Debug)]
pub struct Command {
  cmd_type: CommandType,
  arg1: Option<String>,
  arg2: i16,
}

impl Command {
  pub fn from_str(cmd_str: &str) -> Self {
    println!("DEBUG: parsed {}", cmd_str);
    let mut iter = cmd_str.split_whitespace();
    let cmd = iter.next().expect("Parse error: empty command");
    let (cmd_type, arg_length) = CommandType::parse(cmd);
    let mut arg1 = None;
    let mut arg2 = 0;
    if arg_length > 0 {
      if cmd_type == CommandType::Arithmetic(OperandNum::TwoOperand)
        || cmd_type == CommandType::Arithmetic(OperandNum::SingleOperand)
      {
        arg1 = Some(cmd);
      } else {
        arg1 = Some(iter.next().expect("Parse error: empty arg1"));
      }
    }
    if arg_length > 1 {
      assert_eq!(arg2, 0);
      arg2 = iter
        .next()
        .expect("Parse error: empty arg2")
        .parse::<i16>()
        .expect("Parse error: failed to parse arg2");
    }
    if let Some(arg1) = arg1 {
      Self::new_with_arg12(cmd_type, Some(String::from(arg1)), arg2)
    } else {
      Self::new_with_arg12(cmd_type, None, arg2)
    }
  }

  pub fn new(cmd_type: CommandType) -> Self {
    Self::new_with_arg12(cmd_type, None, 0)
  }

  // pub fn new_with_arg1(cmd_type: CommandType, arg1: String) -> Self {
  //   Self::new_with_arg12(cmd_type, Some(arg1), 0)
  // }

  fn new_with_arg12(cmd_type: CommandType, arg1: Option<String>, arg2: i16) -> Self {
    Self {
      cmd_type: cmd_type,
      arg1: arg1,
      arg2: arg2,
    }
  }

  pub fn cmd_type(&self) -> CommandType {
    self.cmd_type.clone()
  }

  pub fn arg1(&self) -> Option<String> {
    self.arg1.clone()
  }

  pub fn arg2(&self) -> i16 {
    self.arg2
  }
}
