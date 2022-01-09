use std::cell::{RefCell, RefMut};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::rc::Rc;

pub type OutputTarget = Rc<RefCell<BufWriter<File>>>;

pub fn new_output(source: &str) -> OutputTarget {
  let file = File::create(source).expect(format!("{} file open failed", source).as_str());
  Rc::new(RefCell::new(BufWriter::new(file)))
}

pub fn panic_writer(data: String, mut writer: RefMut<BufWriter<File>>) {
  (*writer).write(data.as_bytes()).unwrap();
}
