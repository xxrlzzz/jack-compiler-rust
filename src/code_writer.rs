use indextree::{Node, NodeId};

use crate::operation::tree::OperationTree;
use crate::operation::{BracketType, ConstantType, OperationType, SubroutineType, VarScope};
use crate::symbol_table::*;
use crate::vm::segment_type::SegmentType;
use crate::vm::vm_writer::VmWriter;

pub struct CodeWriter {
  class_symbols: SymbolTable,
  func_symbols: SymbolTable,
  op_tree: OperationTree,
  vm_writer: VmWriter,
  class_name: Option<String>,
  block_return: bool,

  if_count: usize,
  while_count: usize,
}

impl CodeWriter {
  pub fn new(source: &str, op_tree: OperationTree) -> Self {
    Self {
      class_symbols: SymbolTable::new(),
      func_symbols: SymbolTable::new(),
      op_tree,
      vm_writer: VmWriter::new(source),
      class_name: None,
      block_return: false,
      if_count: 0,
      while_count: 0,
    }
  }

  pub fn generate_vm_code(mut self) {
    let root = self.op_tree.root();
    match self.get_node(root).get() {
      OperationType::Class(class) => self.class_name = Some(class.to_string()),
      _ => panic!(""),
    }
    let mut children = self.op_tree.get_children(root);
    let class = children.next().unwrap().clone();
    self.handle_tree(class);
  }

  /**
   *  class
   *    class level var declaration *
   *    subroutine declaration      *
   */
  fn handle_tree(mut self, root: NodeId) {
    let mut children = self.op_tree.get_children(root);
    let mut child_node = children.next();
    loop {
      if let Some(child_id) = child_node {
        let (next_child, child_data) = {
          let child = self.get_node(child_id);
          (child.next_sibling(), child.get())
        };
        match child_data.clone() {
          OperationType::ClassVarDec(scope) => {
            self.handle_var_dec(child_id, scope);
          }
          OperationType::SubroutineDec(subroutine_type) => {
            self.handle_subroutine(child_id, subroutine_type);
          }
          OperationType::Bracket(_) => (),
          _ => panic!("Failed to compile class at {:#?}", child_data),
        }
        child_node = next_child;
      } else {
        break;
      }
    }
  }

  /**
   *  subroutine
   *    subroutine type
   *    return type name
   *    parameter list
   *    subroutine body
   */
  fn handle_subroutine(&mut self, root: NodeId, subroutine_t: SubroutineType) {
    let mut children = self.op_tree.get_children(root).clone();
    // TODO: Add function name table for class.
    let _ret_type = match self.get_node(children.next().unwrap()).get() {
      OperationType::Type(type_name, _) => type_name,
      OperationType::Void => "void",
      _ => panic!(""),
    };
    let func_name = match self.get_node(children.next().unwrap()).get() {
      OperationType::VarName(func_name) => func_name,
      _ => panic!(""),
    };
    let func_name = format!("{}.{}", self.class_name.as_ref().unwrap(), func_name);
    children.next();
    let parameter_list_id = children.next().unwrap();
    match self.get_node(parameter_list_id).get() {
      OperationType::ParameterList => true,
      _ => panic!(""),
    };
    let func_body = children.next().unwrap();
    let _argc = self.handle_parameter_list(parameter_list_id);
    match subroutine_t {
      SubroutineType::Constructor => {
        self.class_symbols.enable_field();
        self.block_return = true;
        self.handle_subroutine_body(func_body, func_name, true);
        self.block_return = false;
        self.vm_writer.generate_return_this();
      }
      SubroutineType::Function => {
        // Function don't have access to field.
        self.class_symbols.disable_field();
        self.handle_subroutine_body(func_body, func_name, false);
      }
      SubroutineType::Method => {
        self.class_symbols.enable_field();
        self.handle_subroutine_body(func_body, func_name, false);
      }
    };

    self.func_symbols.clear();
  }

  /**
   *  subroutine body
   *    local var dec *
   *    statements
   */
  fn handle_subroutine_body(&mut self, root: NodeId, func_name: String, is_constructor: bool) {
    let mut children = self.op_tree.get_children(root);
    // Bracket
    children.next();
    let mut child_node = children.next();
    let mut var_cnt = 0;
    let mut has_statements = false;
    loop {
      if let Some(child_id) = child_node {
        let (next_child, child_data) = {
          let child = self.get_node(child_id);
          (child.next_sibling(), child.get())
        };
        match child_data {
          OperationType::VarDec => {
            if has_statements {
              panic!("Local var declaration after statements.");
            }
            var_cnt += self.handle_var_dec(child_id, VarScope::Variable);
          }
          OperationType::Statements => {
            if has_statements {
              panic!("multiple statements in one function.");
            }
            self.vm_writer.write_func(func_name.clone(), var_cnt);
            if is_constructor {
              self
                .vm_writer
                .generate_alloc_this(self.class_symbols.scope_item_count(VarScope::Field));
            }
            has_statements = true;
            self.handle_statements(child_id)
          }
          _ => panic!(""),
        }
        child_node = next_child;
      } else {
        break;
      }
    }
  }

  /**
   *  var dec
   *    var type
   *    var name *
   */
  fn handle_var_dec(&mut self, root: NodeId, scope: VarScope) -> usize {
    let mut children = self.op_tree.get_children(root);
    let var_type_node = children.next().unwrap();
    let var_type = match self.get_node(var_type_node).get() {
      OperationType::Type(var_type, _) => var_type,
      _ => panic!(""),
    }
    .clone();
    let var_name_list = children.next().unwrap();
    let var_name_list = self.get_node(var_name_list).get().clone();
    match var_name_list {
      OperationType::VarNameList(names) => {
        for name in &names {
          self.insert_symbol(name.clone(), var_type.clone(), scope.clone());
        }
        names.len()
      }
      _ => panic!(""),
    }
  }

  /**
   *  statements
   *    Let|If|While|Do|Return *
   */
  fn handle_statements(&mut self, root: NodeId) {
    let mut children = self.op_tree.get_children(root);
    let mut child_node = children.next();
    loop {
      if let Some(child_id) = child_node {
        let (next_child, child_data) = {
          let child = self.op_tree.get_node(child_id);
          (child.next_sibling(), child.get())
        };
        match child_data.clone() {
          OperationType::LetStatement(var_name) => {
            self.handle_let_statement(var_name.clone(), child_id)
          }
          OperationType::IfStatement => self.handle_if_statement(child_id),
          OperationType::WhileStatement => self.handle_while_statement(child_id),
          OperationType::DoStatement => self.handle_do_statement(child_id),
          OperationType::ReturnStatement => self.handle_return_statement(child_id),
          _ => panic!(""),
        };
        child_node = next_child;
      } else {
        break;
      }
    }
  }

  /**
   *  parameter list
   *    [type name]
   *    [concat type name]*
   */
  fn handle_parameter_list(&mut self, root: NodeId) -> usize {
    let mut children = self.op_tree.get_children(root);
    let mut argc = 0;

    let mut may_child_node = children.next();
    if let None = may_child_node {
      return argc;
    }
    loop {
      let child_node_id = may_child_node.unwrap();
      argc += 1;
      let var_type = match self.get_node(child_node_id).get() {
        OperationType::Type(type_name, _) => type_name,
        _ => panic!(""),
      }
      .clone();
      let var_name_id = children.next().unwrap();
      let var_name = match self.get_node(var_name_id).get() {
        OperationType::VarName(name) => name,
        _ => panic!(""),
      }
      .clone();
      may_child_node = children.next();
      self
        .func_symbols
        .push_item(var_name, var_type, VarScope::Argument);
      if let Some(_) = may_child_node {
        may_child_node = children.next();
      } else {
        break;
      }
    }
    argc
  }

  /**
   *   expression list
   *    expression
   *    [concat expression]*
   */
  fn handle_expression_list(&mut self, root: NodeId) -> usize {
    let mut children = self.op_tree.get_children(root);
    let mut may_expression = children.next();
    let mut argc = 0;
    loop {
      if may_expression.is_none() {
        break;
      }
      let expression = may_expression.unwrap();
      argc += 1;
      self.generate_expression(expression);
      let may_next = self.get_node(expression).next_sibling();
      // let may_next = children.next();
      if let Some(concat) = may_next {
        may_expression = self.get_node(concat).next_sibling();
        // may_expression = children.next();
      } else {
        may_expression = may_next;
      }
    }
    argc
  }

  /**
   *  do statement
   *    subroutine call
   */
  fn handle_do_statement(&mut self, root: NodeId) {
    self.generate_subroutine_call(root);
    self.vm_writer.write_pop(SegmentType::Temp, 0);
  }

  /**
   *  if statement
   *  syntax:
   *    ( condition ) {
   *      statements
   *    } [ { else statement }]
   *  impl:
   *    1.
   *      condition
   *      neg
   *      if-goto end-label
   *      statements
   *      end-label
   *    2.
   *      condition
   *      neg
   *      if-goto else-label
   *      statements
   *      goto end-label
   *      else-label
   *      else statements
   *      end-label
   */
  fn handle_if_statement(&mut self, root: NodeId) {
    let mut children = self.op_tree.get_children(root);
    // Bracket
    children.next();
    // Expression
    let condition = children.next().unwrap();
    // Bracket
    children.next();
    // Statements
    let if_body = children.next().unwrap();
    let if_failed_label = format!("IFFAILEDLABEL{}", self.if_count);
    let if_end_label = format!("IFENDLABEL{}", self.if_count);
    self.if_count += 1;

    if let Some(_) = children.next() {
      children.next();
      let else_body = children.next().unwrap();
      self.generate_expression(condition);
      self.vm_writer.write_arithmetic('~');
      self.vm_writer.write_if(if_failed_label.clone());
      self.handle_statements(if_body);
      self.vm_writer.write_goto(if_end_label.clone());
      self.vm_writer.write_label(if_failed_label);
      self.handle_statements(else_body);
      self.vm_writer.write_label(if_end_label);
    } else {
      self.generate_expression(condition);
      self.vm_writer.write_arithmetic('~');
      self.vm_writer.write_if(if_failed_label.clone());
      self.handle_statements(if_body);
      self.vm_writer.write_label(if_failed_label);
    }
  }

  /**
   *  while statement
   *  syntax:
   *    ( condition ) {
   *      statements
   *    }
   *  impl:
   *    start-label
   *    condition
   *    if-goto end-label
   *    statements
   *    goto start-label
   *    end-label
   */
  fn handle_while_statement(&mut self, root: NodeId) {
    let mut children = self.op_tree.get_children(root);
    // Bracket (
    children.next();
    let condition = children.next().unwrap();
    // Bracket {
    children.next();
    let statement = children.next().unwrap();
    let while_start_label = format!("WHILESTART{}", self.while_count);
    let while_end_label = format!("WHILEEND{}", self.while_count);
    self.while_count += 1;
    self.vm_writer.write_label(while_start_label.clone());
    // expression
    self.generate_expression(condition);
    self.vm_writer.write_arithmetic('~');
    self.vm_writer.write_if(while_end_label.clone());
    // statements
    self.handle_statements(statement);
    self.vm_writer.write_goto(while_start_label);
    self.vm_writer.write_label(while_end_label);
  }

  /**
   *  return statement
   *  syntax:
   *    return [expression]
   *  impl:
   *    if no expression, push constant 0 into stack
   */
  fn handle_return_statement(&mut self, root: NodeId) {
    if self.block_return {
      return;
    }
    let mut children = self.op_tree.get_children(root);
    if let Some(first_child) = children.next() {
      let first_child_node = self.get_node(first_child);
      // println!("return {:?}", first_child_node.get());
      match first_child_node.get() {
        OperationType::Expression => {
          self.generate_expression(first_child);
        }
        _ => panic!(""),
      };
    } else {
      self.vm_writer.write_push(SegmentType::Constant, 0);
    }
    self.vm_writer.write_return();
  }

  /**
   *  let statement
   *  syntax:
   *    1. let var = expression;
   *    2. let var[idx] = expression;
   */
  fn handle_let_statement(&mut self, var_name: String, root: NodeId) {
    let mut children = self.op_tree.get_children(root);
    let var_name_item = self.get_variable(&var_name);
    if var_name_item.is_none() {
      println!("error when compile, var {} not found", var_name);
      return;
    }
    let var_name_item = var_name_item.unwrap().clone();
    let first_child_id = children.next().unwrap();
    let first_child_data = self.get_node(first_child_id).get().clone();
    match first_child_data {
      OperationType::Bracket(BracketType::Square) => {
        let array_index_node = children.next().unwrap();
        children.next();
        let right_expression_node = children.next().unwrap();
        self.generate_expression(right_expression_node);
        self.generate_expression(array_index_node);
        self.vm_writer.write_push(
          SegmentType::from(var_name_item.get_kind()),
          var_name_item.get_idx(),
        );
        self.vm_writer.write_arithmetic('+');
        self.vm_writer.write_pop(SegmentType::Pointer, 1);
        self.vm_writer.write_pop(SegmentType::That, 0);
      }
      OperationType::Op('=') => {
        let expression_node = children.next().unwrap();
        self.generate_expression(expression_node);
        self.vm_writer.write_pop(
          SegmentType::from(var_name_item.get_kind()),
          var_name_item.get_idx(),
        );
      }
      _ => panic!(""),
    }
  }

  /**
   *  expression
   *  syntax:
   *    term
   *    (op term)*
   */
  fn generate_expression(&mut self, root: NodeId) {
    let mut children = self.op_tree.get_children(root);
    assert_eq!(*self.get_node(root).get(), OperationType::Expression);
    let mut left = children.next().unwrap();
    self.generate_term(left);
    loop {
      let op = self.get_node(left).next_sibling();
      if let Some(op) = op {
        let op_node = self.get_node(op);
        let op_content = op_node.get().clone();
        left = op_node.next_sibling().unwrap();
        self.generate_term(left);
        match op_content {
          OperationType::Op(op) => self.vm_writer.write_arithmetic(op),
          _ => panic!(""),
        }
      } else {
        break;
      }
    }
  }

  /**
   *  term
   *  syntax:
   *    1. constant
   *    2. var name
   *    3. var name [ index ]
   *    4. ( expression )
   *    5. op term
   */
  fn generate_term(&mut self, term_node: NodeId) {
    match self.get_node(term_node).get() {
      OperationType::Term => true,
      _ => panic!(""),
    };
    let mut children = self.op_tree.get_children(term_node);
    let first_child = children.next().unwrap();
    let may_second_child = children.next();
    match self.get_node(first_child).get() {
      OperationType::Constant(const_t) => {
        match const_t {
          ConstantType::Integer(i) => {
            let constant = *i as usize;
            self.vm_writer.write_push(SegmentType::Constant, constant);
          }
          ConstantType::String(s) => {
            let s_const = s.clone();
            self
              .vm_writer
              .write_push(SegmentType::Constant, s_const.len());
            self.vm_writer.write_call("String.new".to_string(), 1);
            self.vm_writer.write_pop(SegmentType::Temp, 2);
            for c in s_const.as_bytes() {
              self.vm_writer.write_push(SegmentType::Temp, 2);
              self
                .vm_writer
                .write_push(SegmentType::Constant, *c as usize);
              self
                .vm_writer
                .write_call("String.appendChar".to_string(), 2);
            }
          }
          ConstantType::KeyWord(k) => {
            if k == "true" {
              self.vm_writer.write_push(SegmentType::Constant, 1);
              self.vm_writer.write_arithmetic('^');
            } else if k == "false" || k == "null" {
              self.vm_writer.write_push(SegmentType::Constant, 0);
            } else if k == "this" {
              self.vm_writer.write_push(SegmentType::Pointer, 0);
            }
          }
        };
      }
      OperationType::VarName(var_name) => {
        let var = self.get_variable(var_name).unwrap().clone();
        self
          .vm_writer
          .write_push(var.get_kind().into(), var.get_idx());
        if may_second_child.is_some() {
          // array index
          let next_child = children.next().unwrap();
          self.generate_expression(next_child);
          self.vm_writer.write_arithmetic('+');
          self.vm_writer.write_pop(SegmentType::Pointer, 1);
          self.vm_writer.write_push(SegmentType::That, 0);
        }
      }
      OperationType::Op(op) => {
        let op = op.clone();
        self.generate_term(may_second_child.unwrap());
        self.vm_writer.write_arithmetic(op);
      }
      OperationType::Bracket(_) => {
        self.generate_expression(may_second_child.unwrap());
      }
      OperationType::SubroutineCall(_, _) => {
        self.generate_subroutine_call(term_node);
      }
      _ => {
        panic!(
          "Failed to compile term at {:?}",
          self.get_node(first_child).get()
        )
      }
    }
  }

  /**
   *  subroutine call
   *  syntax:
   *    1. class name.function name
   *    2. function name
   *    ( expression list )
   *  note:
   *    Instead push this pointer for method call, we stash this pointer in to stack,
   *    and set up this pointer for method automatically. When method returned, restore
   *    original this pointer back.
   *    (We treat a [local variable name.function name] as method in here.)
   */
  fn generate_subroutine_call(&mut self, root: NodeId) {
    let mut children = self.op_tree.get_children(root);
    let subroutine_call_node = children.next().unwrap();
    let expressions = children.skip(1).next().unwrap();
    let mut may_var: Option<VariableSymbolItem> = None;
    let subroutine_call = match self.get_node(subroutine_call_node).get().clone() {
      OperationType::SubroutineCall(first_name, second_name) => match first_name {
        Some(first_name) => {
          let tmp_may_var = self.get_variable(&first_name);
          if tmp_may_var.is_some() {
            let var = tmp_may_var.unwrap().clone();
            may_var = Some(var.clone());

            // Stash this pointer into stack.
            self.vm_writer.write_push(SegmentType::Pointer, 0);
            format!("{}.{}", var.get_type(), second_name)
          } else {
            format!("{}.{}", first_name, second_name)
          }
        }
        None => format!("{}.{}", self.class_name.as_ref().unwrap(), second_name),
      },
      _ => panic!(""),
    };
    let argc = self.handle_expression_list(expressions);
    if may_var.is_some() {
      let var = may_var.as_ref().unwrap();
      // Set up this pointer for function call.
      self
        .vm_writer
        .write_push(var.get_kind().into(), var.get_idx());
      self.vm_writer.write_pop(SegmentType::Pointer, 0);
    }
    self.vm_writer.write_call(subroutine_call, argc);
    if may_var.is_some() {
      // Restore this pointer from stack .
      self.vm_writer.write_pop(SegmentType::Temp, 1);
      self.vm_writer.write_pop(SegmentType::Pointer, 0);
      self.vm_writer.write_push(SegmentType::Temp, 1);
    }
  }

  fn get_variable(&self, name: &String) -> Option<&VariableSymbolItem> {
    let r = self.func_symbols.find_item_by_name(name);
    if r.is_some() {
      return r;
    }
    self.class_symbols.find_item_by_name(name)
  }

  fn get_node(&self, node_id: NodeId) -> &Node<OperationType> {
    self.op_tree.get_node(node_id)
  }

  fn insert_symbol(&mut self, name: String, v_type: String, kind: VarScope) {
    if kind.is_class_scope() {
      self.class_symbols.push_item(name, v_type, kind);
    } else {
      self.func_symbols.push_item(name, v_type, kind);
    }
  }
}
