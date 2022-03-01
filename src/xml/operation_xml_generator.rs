use crate::common::{new_output, OutputTarget};
use crate::compiler::{WritableStack, WriteTarget};
use crate::operation::{OperationType, ConstantType};
use crate::xml::*;

pub struct OperationXMLGenerator {
  writer: OutputTarget,
  node_type_stack: Vec<OperationType>,
  cur_indent: usize,
}

pub struct RAIIWriter {
  target: WriteTarget,
}

impl RAIIWriter {
  pub fn new(target: WriteTarget, node: OperationType) -> Self {
    target.clone().borrow_mut().push(node);
    RAIIWriter { target }
  }
}

impl Drop for RAIIWriter {
  fn drop(&mut self) {
    self.target.borrow_mut().pop();
  }
}

impl OperationXMLGenerator {
  pub fn new(source: &str) -> Self {
    Self {
      writer: new_output(source),
      node_type_stack: vec![],
      cur_indent: 0,
    }
  }

  fn forward_indent(&mut self) {
    self.cur_indent += 2;
  }

  fn backward_indent(&mut self) {
    self.cur_indent -= 2;
  }

  fn indent_write(&self, data: &str) {
    panic_writer_with_indent(data.to_string(), self.cur_indent, self.writer.clone());
  }

  fn tag_indent_write(&self, tag: &str, data: &str) {
    panic_tag_content_with_indent(tag, data, self.cur_indent, self.writer.clone());
  }
}

impl WritableStack for OperationXMLGenerator {
  fn push(&mut self, node_type: OperationType) {
    // println!("push {:?}", node_type);
    match &node_type {
      OperationType::Class(class_name) => {
        self.indent_write("<class>\n");
        self.forward_indent();
        self.tag_indent_write("keyword", "class");
        self.tag_indent_write("identifier", class_name);
      }
      OperationType::ClassVarDec(var_type) => {
        self.indent_write("<classVarDec>\n");
        self.forward_indent();
        self.tag_indent_write("keyword", var_type.to_string().as_str());
      }
      OperationType::SubroutineDec(subroutine_type) => {
        self.indent_write("<subroutineDec>\n");
        self.forward_indent();
        self.tag_indent_write("keyword", subroutine_type.to_string().as_str());
      }
      OperationType::Void => {
        self.tag_indent_write("keyword", "void");
      }
      OperationType::Type(u_type, is_keyword) => {
        if *is_keyword {
          self.tag_indent_write("keyword", u_type);
        } else {
          self.tag_indent_write("identifier", u_type);
        }
      }
      OperationType::ParameterList => {
        self.indent_write("<parameterList>\n");
        self.forward_indent();
      }
      OperationType::SubroutineBody => {
        self.indent_write("<subroutineBody>\n");
        self.forward_indent();
      }
      OperationType::VarDec => {
        self.indent_write("<varDec>\n");
        self.forward_indent();
        self.tag_indent_write("keyword", "var");
      }
      OperationType::VarNameList(names) => {
        let mut iter = names.into_iter();
        if let Some(first_name) = iter.next() {
          self.tag_indent_write("identifier", first_name.as_str());

          for name in iter {
            self.tag_indent_write("symbol", ",");
            self.tag_indent_write("identifier", name.as_str());
          }
        }
      }
      OperationType::Statements => {
        self.indent_write("<statements>\n");
        self.forward_indent();
      }
      OperationType::LetStatement(var_name) => {
        self.indent_write("<letStatement>\n");
        self.forward_indent();
        self.tag_indent_write("keyword", "let");
        self.tag_indent_write("identifier", var_name.as_str());
        // self.tag_indent_write("symbol", "=");
      }
      OperationType::Expression => {
        self.indent_write("<expression>\n");
        self.forward_indent();
      }
      OperationType::ExpressionList => {
        self.indent_write("<expressionList>\n");
        self.forward_indent();
      }
      OperationType::Term => {
        self.indent_write("<term>\n");
        self.forward_indent();
      }
      OperationType::Constant(c_type) => {
        if let ConstantType::KeyWord(c_type) = c_type {
          self.tag_indent_write("keyword", c_type.to_string().as_str());
        } else {
          self.tag_indent_write(
            format!("{}Constant", c_type.to_string()).as_str(),
            c_type.content().as_str(),
          );
        }
      }
      OperationType::VarName(name) => {
        self.tag_indent_write("identifier", name.as_str());
      }
      OperationType::Op(op) => {
        self.tag_indent_write("symbol", translate(*op).as_str());
      }
      OperationType::Bracket(b_type) => {
        self.tag_indent_write("symbol", b_type.left());
      }
      OperationType::SubroutineCall(type_name, function_name) => {
        if let Some(type_name) = type_name {
          self.tag_indent_write("identifier", type_name.as_str());
          self.tag_indent_write("symbol", ".");
        }
        self.tag_indent_write("identifier", function_name.as_str());
      }
      OperationType::IfStatement => {
        self.indent_write("<ifStatement>\n");
        self.forward_indent();
        self.tag_indent_write("keyword", "if");
      }
      OperationType::Else => {
        self.tag_indent_write("keyword", "else");
      }
      OperationType::ListConcat => {
        self.tag_indent_write("symbol", ",");
      }
      OperationType::WhileStatement => {
        self.indent_write("<whileStatement>\n");
        self.forward_indent();
        self.tag_indent_write("keyword", "while");
      }
      OperationType::DoStatement => {
        self.indent_write("<doStatement>\n");
        self.forward_indent();
        self.tag_indent_write("keyword", "do");
      }
      OperationType::ReturnStatement => {
        self.indent_write("<returnStatement>\n");
        self.forward_indent();
        self.tag_indent_write("keyword", "return");
      }
    }
    self.node_type_stack.push(node_type);
  }

  fn pop(&mut self) {
    assert!(self.node_type_stack.len() > 0);
    let node_type = self.node_type_stack.pop().unwrap();
    // println!("pop {:?}", node_type);
    match node_type {
      OperationType::Class(_) => {
        self.backward_indent();
        self.indent_write("</class>\n");
      }
      OperationType::ClassVarDec(_) => {
        self.tag_indent_write("symbol", ";");
        self.backward_indent();
        self.indent_write("</classVarDec>\n");
      }
      OperationType::SubroutineDec(_) => {
        self.backward_indent();
        self.indent_write("</subroutineDec>\n");
      }
      OperationType::Void => {}
      OperationType::Type(_, _) => {}
      OperationType::ParameterList => {
        self.backward_indent();
        self.indent_write("</parameterList>\n");
      }
      OperationType::SubroutineBody => {
        self.backward_indent();
        self.indent_write("</subroutineBody>\n");
      }
      OperationType::VarDec => {
        self.tag_indent_write("symbol", ";");
        self.backward_indent();
        self.indent_write("</varDec>\n");
      }
      OperationType::VarNameList(_) => {}
      OperationType::Statements => {
        self.backward_indent();
        self.indent_write("</statements>\n");
      }
      OperationType::LetStatement(_) => {
        self.tag_indent_write("symbol", ";");
        self.backward_indent();
        self.indent_write("</letStatement>\n");
      }
      OperationType::Expression => {
        self.backward_indent();
        self.indent_write("</expression>\n");
      }
      OperationType::ExpressionList => {
        self.backward_indent();
        self.indent_write("</expressionList>\n");
      }
      OperationType::Term => {
        self.backward_indent();
        self.indent_write("</term>\n");
      }
      OperationType::Constant(_) => {}
      OperationType::VarName(_) => {}
      OperationType::Op(_) => {}
      OperationType::Bracket(b_type) => {
        self.tag_indent_write("symbol", b_type.right());
      }
      OperationType::SubroutineCall(_, _) => {}
      OperationType::IfStatement => {
        self.backward_indent();
        self.indent_write("</ifStatement>\n");
      }
      OperationType::Else => {}
      OperationType::ListConcat => {}
      OperationType::WhileStatement => {
        self.backward_indent();
        self.indent_write("</whileStatement>\n");
      }
      OperationType::DoStatement => {
        self.tag_indent_write("symbol", ";");
        self.backward_indent();
        self.indent_write("</doStatement>\n");
      }
      OperationType::ReturnStatement => {
        self.tag_indent_write("symbol", ";");
        self.backward_indent();
        self.indent_write("</returnStatement>\n");
      }
    }
  }
}
