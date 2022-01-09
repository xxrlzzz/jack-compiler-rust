use crate::common::{new_output, panic_writer, OutputTarget};

use crate::vm::segment_type::SegmentType;

pub struct VmWriter {
  output: OutputTarget,
}

impl VmWriter {
  pub fn new(source: &str) -> Self {
    Self {
      output: new_output(source),
    }
  }

  pub fn write_push(&mut self, seg_t: SegmentType, idx: usize) {
    panic_writer(
      format!("push {} {}\n", seg_t.to_vm_string(), idx),
      self.output.clone().borrow_mut(),
    );
  }

  pub fn write_pop(&mut self, seg_t: SegmentType, idx: usize) {
    panic_writer(
      format!("pop {} {}\n", seg_t.to_vm_string(), idx),
      self.output.clone().borrow_mut(),
    );
  }

  pub fn write_push_stack(&mut self, var: usize) {
    panic_writer(format!("push {}\n", var), self.output.clone().borrow_mut());
  }

  pub fn write_pop_stack(&mut self) {
    panic_writer(format!("pop\n"), self.output.clone().borrow_mut());
  }

  pub fn write_arithmetic(&mut self, op: char) {
    let cmd = match op {
      '+' => "add",
      '-' => "sub",
      '&' => "and",
      '|' => "or",
      '>' => "gt",
      '=' => "eq",
      '<' => "lt",
      // Single operand.
      '~' => "not",
      '^' => "neg",
      // Mathlib
      '*' => "call Math.mul",
      '/' => "call Math.div",
      _ => "",
    };
    panic_writer(cmd.to_string() + "\n", self.output.clone().borrow_mut());
  }

  pub fn write_label(&mut self, label: String) {
    panic_writer(
      format!("{} {}\n", "label", label),
      self.output.clone().borrow_mut(),
    );
  }
  pub fn write_goto(&mut self, label: String) {
    panic_writer(
      format!("{} {}\n", "goto", label),
      self.output.clone().borrow_mut(),
    );
  }

  pub fn write_if(&mut self, label: String) {
    panic_writer(
      format!("{} {}\n", "if-goto", label),
      self.output.clone().borrow_mut(),
    );
  }

  pub fn write_call(&mut self, name: String, argc: usize) {
    panic_writer(
      format!("call {} {}\n", name, argc),
      self.output.clone().borrow_mut(),
    );
  }

  ///
  /// argc: Local variable count.
  pub fn write_func(&mut self, name: String, argc: usize) {
    panic_writer(
      format!("function {} {}\n", name, argc),
      self.output.clone().borrow_mut(),
    );
  }

  pub fn write_return(&mut self) {
    panic_writer("return\n".to_string(), self.output.clone().borrow_mut());
  }

  pub fn generate_return_this(&mut self) {
    self.write_push(SegmentType::Pointer, 0);
    self.write_return();
  }

  pub fn generate_alloc_this(&mut self, filed_count: usize) {
    self.write_push_stack(filed_count);
    self.write_call("Memory.alloc".to_string(), 1);
    self.write_push(SegmentType::Pointer, 0);
  }
}
