pub mod tree;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VarScope {
  // Class types.
  Static,
  Field,
  // Local types.
  Argument,
  Variable,

  None,
}
const ARGUMENT: &'static str = "argument";
const LOCAL: &'static str = "local";
const STATIC: &'static str = "static";
const THIS: &str = "this";
impl VarScope {
  pub fn is_class_scope(&self) -> bool {
    match self {
      VarScope::Static => true,
      VarScope::Field => true,

      _ => false,
    }
  }

  pub fn to_str(&self) -> &str {
    match self {
      VarScope::Static => STATIC,
      VarScope::Field => THIS,
      VarScope::Argument => ARGUMENT,
      VarScope::Variable => LOCAL,
      _ => "",
    }
  }
}
use crate::vm::segment_type::SegmentType;
impl From<VarScope> for SegmentType {
  fn from(var_scope: VarScope) -> Self {
    match var_scope {
      VarScope::Static => SegmentType::Static,
      VarScope::Field => SegmentType::This,
      VarScope::Argument => SegmentType::Argument,
      VarScope::Variable => SegmentType::Local,
      _ => SegmentType::Invalid,
    }
  }
}

impl ToString for VarScope {
  fn to_string(&self) -> String {
    match self {
      VarScope::Static => STATIC.to_string(),
      VarScope::Field => "field".to_string(),
      VarScope::Argument => ARGUMENT.to_string(),
      VarScope::Variable => "variable".to_string(),
      _ => "unsupported".to_string(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubroutineType {
  Constructor,
  Function,
  Method,
}

impl ToString for SubroutineType {
  fn to_string(&self) -> String {
    match self {
      SubroutineType::Constructor => "constructor".to_string(),
      SubroutineType::Function => "function".to_string(),
      SubroutineType::Method => "method".to_string(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantType {
  Integer(u16),
  String(String),
  KeyWord(String),
}

impl ConstantType {
  pub fn content(&self) -> String {
    match self {
      ConstantType::Integer(i) => i.to_string(),
      ConstantType::String(s) => s.clone(),
      ConstantType::KeyWord(s) => s.clone(),
    }
  }
}

impl ToString for ConstantType {
  fn to_string(&self) -> String {
    match self {
      ConstantType::Integer(_) => "integer".to_string(),
      ConstantType::String(_) => "string".to_string(),
      ConstantType::KeyWord(_) => "keyword".to_string(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BracketType {
  Square,
  Parenthesis,
  Curly,
  None,
}

impl BracketType {
  pub fn left(&self) -> &'static str {
    match self {
      BracketType::Square => "[",
      BracketType::Parenthesis => "(",
      BracketType::Curly => "{",
      BracketType::None => "",
    }
  }
  pub fn right(&self) -> &'static str {
    match self {
      BracketType::Square => "]",
      BracketType::Parenthesis => ")",
      BracketType::Curly => "}",
      BracketType::None => "",
    }
  }

  pub fn from_char(c: char) -> Self {
    match c {
      '[' => BracketType::Square,
      '(' => BracketType::Parenthesis,
      '{' => BracketType::Curly,
      _ => BracketType::None,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
  Class(String),
  ClassVarDec(VarScope),
  Void,
  Type(String, bool), // type/ is keyword
  SubroutineDec(SubroutineType),
  ParameterList,
  SubroutineBody,
  VarDec,
  VarNameList(Vec<String>), // list of var name
  Statements,
  LetStatement(String), // var name
  IfStatement,
  WhileStatement,
  DoStatement,
  ReturnStatement,
  Expression,
  Term,
  SubroutineCall(Option<String>, String), // typename / func name
  ExpressionList,
  Op(char),
  Constant(ConstantType), // Identifier(String),
  VarName(String),
  Bracket(BracketType),
  ListConcat,
  Else,
}

impl OperationType {
  pub fn has_child(&self) -> bool {
    match self {
      OperationType::Class(_)
      | OperationType::ClassVarDec(_)
      | OperationType::SubroutineDec(_)
      | OperationType::ParameterList
      | OperationType::SubroutineBody
      | OperationType::VarDec
      | OperationType::Statements
      | OperationType::LetStatement(_)
      | OperationType::Expression
      | OperationType::ExpressionList
      | OperationType::Term
      | OperationType::IfStatement
      | OperationType::WhileStatement
      | OperationType::DoStatement
      | OperationType::ReturnStatement => true,
      _ => false,
    }
  }
}
