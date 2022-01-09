pub mod operation_xml_generator;
pub mod token_xml_generator;

use std::collections::HashMap;
// use std::io::Write;
use std::vec::Vec;

use crate::common::{panic_writer, OutputTarget};

// pub type OutputTarget = Rc<RefCell<BufWriter<File>>>;

pub(crate) struct RAIIWriter {
  data: String,
  writer: OutputTarget,
}

impl RAIIWriter {
  pub fn new(data: String, writer: OutputTarget) -> Self {
    panic_writer(format!("<{}>", data), writer.clone().borrow_mut());
    Self {
      data: data,
      writer: writer,
    }
  }

  pub fn new_with_newline(data: String, writer: OutputTarget) -> Self {
    panic_writer(format!("<{}>\n", data), writer.clone().borrow_mut());
    Self {
      data: data,
      writer: writer,
    }
  }
}
impl Drop for RAIIWriter {
  fn drop(&mut self) {
    panic_writer(
      format!("</{}>\n", self.data),
      self.writer.clone().borrow_mut(),
    );
  }
}

pub(crate) fn panic_writer_with_indent(data: String, indent: usize, writer: OutputTarget) {
  let mut indent_v = Vec::with_capacity(indent);
  for _ in 0..indent {
    indent_v.push(' ' as u8);
  }
  panic_writer(String::from_utf8(indent_v).unwrap(), writer.borrow_mut());
  panic_writer(data, writer.borrow_mut());
}

pub(crate) fn panic_tag_content_with_indent(
  tag: &str,
  content: &str,
  indent: usize,
  writer: OutputTarget,
) {
  panic_writer_with_indent(format!("<{}>", tag), indent, writer.clone());
  panic_writer(format!(" {} ", content), writer.clone().borrow_mut());
  panic_writer(format!("</{}>\n", tag), writer.clone().borrow_mut());
}

lazy_static! {
  static ref TRANSLATE_TABLE: HashMap<char, String> = {
    let mut map = HashMap::new();
    map.insert('<', "&lt;".to_string());
    map.insert('>', "&gt;".to_string());
    map.insert('&', "&amp;".to_string());
    map
  };
}

pub(crate) fn translate(c: char) -> String {
  if let Some(s) = TRANSLATE_TABLE.get(&c) {
    return s.clone();
  }
  c.to_string()
}
