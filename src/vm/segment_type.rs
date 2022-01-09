const ARGUMENT: &str = "argument";
const LOCAL: &str = "local";
const STATIC: &str = "static";
const CONSTANT: &str = "constant";
const THIS: &str = "this";
const THAT: &str = "that";
const POINTER: &str = "pointer";
const TEMP: &str = "temp";

#[derive(Debug, Clone, PartialEq)]
pub enum SegmentType {
  Local,
  Static,
  Pointer,
  Constant,
  Temp,
  Argument,
  This,
  That,
  Invalid,
}

impl SegmentType {
  pub fn new(seg: &str) -> Self {
    match seg {
      ARGUMENT => Self::Argument,
      LOCAL => Self::Local,
      STATIC => Self::Static,
      CONSTANT => Self::Constant,
      THIS => Self::This,
      THAT => Self::That,
      POINTER => Self::Pointer,
      TEMP => Self::Temp,
      _ => Self::Invalid,
    }
  }

  pub fn to_asm_string(&self) -> String {
    match self {
      Self::Argument => String::from("2"),
      Self::Local => String::from("1"),
      Self::Static => String::from("static"),
      Self::Constant => String::from("const"),
      Self::This => String::from("3"),
      Self::That => String::from("4"),
      Self::Pointer => String::from("pointer"),
      Self::Temp => String::from("5"),
      Self::Invalid => String::from("invalid"),
    }
  }

  pub fn to_vm_string(&self) -> &str {
    match self {
      Self::Argument => ARGUMENT,
      Self::Local => LOCAL,
      Self::Static => STATIC,
      Self::Constant => CONSTANT,
      Self::This => THIS,
      Self::That => THAT,
      Self::Pointer => POINTER,
      Self::Temp => TEMP,
      Self::Invalid => "",
    }
  }
}
