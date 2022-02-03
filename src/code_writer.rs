use indextree::{Node, NodeId};
use log::error;

use crate::operation::tree::OperationTree;
use crate::operation::{BracketType, ConstantType, OperationType, SubroutineType, VarScope};
use crate::symbol_table::*;
use crate::vm::segment_type::SegmentType;
use crate::vm::vm_writer::VmWriter;

struct State {
  class_symbols: SymbolTable,
  func_symbols: SymbolTable,
  vm_writer: VmWriter,
  block_return: bool,

  if_count: usize,
  while_count: usize,
}

// Mutable part of a code generation.
impl State {
  pub fn new(source: &str) -> Self {
    Self {
      class_symbols: SymbolTable::new(),
      func_symbols: SymbolTable::new(),
      vm_writer: VmWriter::new(source),
      block_return: false,
      if_count: 0,
      while_count: 0,
    }
  }

  pub fn get_variable(&self, name: &String) -> Option<&VariableSymbolItem> {
    let r = self.func_symbols.find_item_by_name(name);
    if r.is_some() {
      return r;
    }
    self.class_symbols.find_item_by_name(name)
  }

  pub fn insert_symbol(&mut self, name: String, v_type: String, kind: VarScope) {
    if kind.is_class_scope() {
      self.class_symbols.push_item(name, v_type, kind);
    } else {
      self.func_symbols.push_item(name, v_type, kind);
    }
  }
}

pub struct CodeWriter {
  op_tree: OperationTree,
  source: String,
  class_name: Option<String>,
}

impl CodeWriter {
  pub fn new(source: &str, op_tree: OperationTree) -> Self {
    Self {
      op_tree,
      source: String::from(source),
      class_name: None,
    }
  }

  pub fn generate_vm_code(mut self) {
    let root = self.op_tree.root();
    match self.get_node_data(root) {
      OperationType::Class(class) => self.class_name = Some(class.to_string()),
      _ => panic!(""),
    }
    let mut children = self.op_tree.get_children(root);
    let class = children.next().unwrap();
    let mut state = State::new(&self.source[..]);

    self.handle_tree(class, &mut state);
  }

  /**
   *  class
   *    class level var declaration *
   *    subroutine declaration      *
   */
  fn handle_tree(self, root: NodeId, state: &mut State) {
    for child_id in self.op_tree.get_children(root) {
      let child_data = self.get_node_data(child_id);
      match child_data {
        OperationType::ClassVarDec(scope) => {
          self.handle_var_dec(child_id, *scope, state);
        }
        OperationType::SubroutineDec(subroutine_type) => {
          self.handle_subroutine(child_id, *subroutine_type, state);
        }
        OperationType::Bracket(_) => (),
        _ => panic!("Failed to compile class at {:#?}", child_data),
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
  fn handle_subroutine(&self, root: NodeId, subroutine_t: SubroutineType, state: &mut State) {
    let mut children = self.op_tree.get_children(root).clone();
    // TODO: Add function name table for class.
    let _ret_type = match self.get_node_data(children.next().unwrap()) {
      OperationType::Type(type_name, _) => type_name,
      OperationType::Void => "void",
      _ => panic!(""),
    };
    let func_name = match self.get_node_data(children.next().unwrap()) {
      OperationType::VarName(func_name) => func_name,
      _ => panic!(""),
    };
    let func_name = format!("{}.{}", self.class_name.as_ref().unwrap(), func_name);
    children.next();
    let parameter_list_id = children.next().unwrap();
    match self.get_node_data(parameter_list_id) {
      OperationType::ParameterList => true,
      _ => panic!(""),
    };
    let func_body = children.next().unwrap();

    if subroutine_t == SubroutineType::Method {
      state.func_symbols.push_item(
        String::from("this"),
        String::from("this"),
        VarScope::Argument,
      );
    }
    let _argc = self.handle_parameter_list(parameter_list_id, state);

    match subroutine_t {
      SubroutineType::Constructor => {
        state.class_symbols.enable_field();
        state.block_return = true;
        self.handle_subroutine_body(func_body, func_name, subroutine_t, state);
        state.block_return = false;
        state.vm_writer.generate_return_this();
      }
      SubroutineType::Function => {
        // Function don't have access to field.
        state.class_symbols.disable_field();
        self.handle_subroutine_body(func_body, func_name, subroutine_t, state);
      }
      SubroutineType::Method => {
        state.class_symbols.enable_field();
        self.handle_subroutine_body(func_body, func_name, subroutine_t, state);
      }
    };

    state.func_symbols.clear();
  }

  /**
   *  subroutine body
   *    local var dec *
   *    statements
   */
  fn handle_subroutine_body(
    &self,
    root: NodeId,
    func_name: String,
    subroutine_type: SubroutineType,
    state: &mut State,
  ) {
    let mut var_cnt = 0;
    let mut has_statements = false;
    // Skip Bracket
    for child_id in self.op_tree.get_children(root).skip(1) {
      match self.get_node_data(child_id) {
        OperationType::VarDec => {
          if has_statements {
            panic!("Local var declaration after statements.");
          }
          var_cnt += self.handle_var_dec(child_id, VarScope::Variable, state);
        }
        OperationType::Statements => {
          if has_statements {
            panic!("multiple statements in one function.");
          }
          state.vm_writer.write_func(func_name.clone(), var_cnt);
          if subroutine_type == SubroutineType::Constructor {
            state
              .vm_writer
              .generate_alloc_this(state.class_symbols.scope_item_count(VarScope::Field));
          } else if subroutine_type == SubroutineType::Method {
            // Set this pointer.
            state.vm_writer.write_push(SegmentType::Argument, 0);
            state.vm_writer.write_pop(SegmentType::Pointer, 0);
          }
          has_statements = true;
          self.handle_statements(child_id, state)
        }
        _ => panic!(""),
      }
    }
  }

  /**
   *  var dec
   *    var type
   *    var name *
   */
  fn handle_var_dec(&self, root: NodeId, scope: VarScope, state: &mut State) -> usize {
    let mut children = self.op_tree.get_children(root);
    let var_type_node = children.next().unwrap();
    let var_type = match self.get_node_data(var_type_node) {
      OperationType::Type(var_type, _) => var_type.clone(),
      _ => panic!(""),
    };
    let var_name_list = children.next().unwrap();
    let var_name_list = self.get_node_data(var_name_list);
    match var_name_list {
      OperationType::VarNameList(names) => {
        for name in names {
          state.insert_symbol(name.clone(), var_type.clone(), scope.clone());
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
  fn handle_statements(&self, root: NodeId, state: &mut State) {
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
            self.handle_let_statement(var_name.clone(), child_id, state)
          }
          OperationType::IfStatement => self.handle_if_statement(child_id, state),
          OperationType::WhileStatement => self.handle_while_statement(child_id, state),
          OperationType::DoStatement => self.handle_do_statement(child_id, state),
          OperationType::ReturnStatement => self.handle_return_statement(child_id, state),
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
  fn handle_parameter_list(&self, root: NodeId, state: &mut State) -> usize {
    let mut children = self.op_tree.get_children(root);
    let mut argc = 0;

    let mut may_child_node = children.next();
    if let None = may_child_node {
      return argc;
    }
    loop {
      let child_node_id = may_child_node.unwrap();
      argc += 1;
      let var_type = match self.get_node_data(child_node_id) {
        OperationType::Type(type_name, _) => type_name.clone(),
        _ => panic!(""),
      };
      let var_name_id = children.next().unwrap();
      let var_name = match self.get_node_data(var_name_id) {
        OperationType::VarName(name) => name.clone(),
        _ => panic!(""),
      };
      may_child_node = children.next();
      state
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
  fn handle_expression_list(&self, root: NodeId, state: &mut State) -> usize {
    let mut children = self.op_tree.get_children(root);
    let mut may_expression = children.next();
    let mut argc = 0;
    loop {
      if may_expression.is_none() {
        break;
      }
      let expression = may_expression.unwrap();
      argc += 1;
      self.generate_expression(expression, state);
      let may_next = children.next();
      if let Some(_) = may_next {
        may_expression = children.next();
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
  fn handle_do_statement(&self, root: NodeId, state: &mut State) {
    self.generate_subroutine_call(root, state);
    state.vm_writer.write_pop(SegmentType::Temp, 0);
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
  fn handle_if_statement(&self, root: NodeId, state: &mut State) {
    let mut children = self.op_tree.get_children(root);
    // Bracket
    children.next();
    // Expression
    let condition = children.next().unwrap();
    // Bracket
    children.next();
    // Statements
    let if_body = children.next().unwrap();
    let if_failed_label = format!("IFFAILEDLABEL{}", state.if_count);
    let if_end_label = format!("IFENDLABEL{}", state.if_count);
    state.if_count += 1;

    if let Some(_) = children.next() {
      children.next();
      let else_body = children.next().unwrap();
      self.generate_expression(condition, state);
      state.vm_writer.write_arithmetic('~');
      state.vm_writer.write_if(if_failed_label.clone());
      self.handle_statements(if_body, state);
      state.vm_writer.write_goto(if_end_label.clone());
      state.vm_writer.write_label(if_failed_label);
      self.handle_statements(else_body, state);
      state.vm_writer.write_label(if_end_label);
    } else {
      self.generate_expression(condition, state);
      state.vm_writer.write_arithmetic('~');
      state.vm_writer.write_if(if_failed_label.clone());
      self.handle_statements(if_body, state);
      state.vm_writer.write_label(if_failed_label);
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
  fn handle_while_statement(&self, root: NodeId, state: &mut State) {
    let mut children = self.op_tree.get_children(root);
    // Bracket (
    children.next();
    let condition = children.next().unwrap();
    // Bracket {
    children.next();
    let statement = children.next().unwrap();
    let while_start_label = format!("WHILESTART{}", state.while_count);
    let while_end_label = format!("WHILEEND{}", state.while_count);
    state.while_count += 1;
    state.vm_writer.write_label(while_start_label.clone());
    // expression
    self.generate_expression(condition, state);
    state.vm_writer.write_arithmetic('~');
    state.vm_writer.write_if(while_end_label.clone());
    // statements
    self.handle_statements(statement, state);
    state.vm_writer.write_goto(while_start_label);
    state.vm_writer.write_label(while_end_label);
  }

  /**
   *  return statement
   *  syntax:
   *    return [expression]
   *  impl:
   *    if no expression, push constant 0 into stack
   */
  fn handle_return_statement(&self, root: NodeId, state: &mut State) {
    if state.block_return {
      return;
    }
    let mut children = self.op_tree.get_children(root);
    if let Some(first_child) = children.next() {
      match self.get_node_data(first_child) {
        OperationType::Expression => {
          self.generate_expression(first_child, state);
        }
        _ => panic!(""),
      };
    } else {
      state.vm_writer.write_push(SegmentType::Constant, 0);
    }
    state.vm_writer.write_return();
  }

  /**
   *  let statement
   *  syntax:
   *    1. let var = expression;
   *    2. let var[idx] = expression;
   */
  fn handle_let_statement(&self, var_name: String, root: NodeId, state: &mut State) {
    let mut children = self.op_tree.get_children(root);
    let var_name_item = state.get_variable(&var_name);
    if var_name_item.is_none() {
      error!("error when compile, var {} not found", var_name);
      return;
    }
    let var_name_item = var_name_item.unwrap().clone();
    let first_child_id = children.next().unwrap();
    let first_child_data = self.get_node_data(first_child_id).clone();
    match first_child_data {
      OperationType::Bracket(BracketType::Square) => {
        let array_index_node = children.next().unwrap();
        children.next();
        let right_expression_node = children.next().unwrap();
        self.generate_expression(right_expression_node, state);
        self.generate_expression(array_index_node, state);
        state.vm_writer.write_push(
          SegmentType::from(var_name_item.get_kind()),
          var_name_item.get_idx(),
        );
        state.vm_writer.write_arithmetic('+');
        state.vm_writer.write_pop(SegmentType::Pointer, 1);
        state.vm_writer.write_pop(SegmentType::That, 0);
      }
      OperationType::Op('=') => {
        let expression_node = children.next().unwrap();
        self.generate_expression(expression_node, state);
        state.vm_writer.write_pop(
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
  fn generate_expression(&self, root: NodeId, state: &mut State) {
    let mut children = self.op_tree.get_children(root);
    assert_eq!(*self.get_node_data(root), OperationType::Expression);
    let mut left = children.next().unwrap();
    self.generate_term(left, state);
    loop {
      let op = children.next();
      if let Some(op) = op {
        left = children.next().unwrap();
        self.generate_term(left, state);
        match self.get_node_data(op) {
          OperationType::Op(c) => state.vm_writer.write_arithmetic(*c),
          _ => panic!(""),
        }
      } else {
        break;
      }
    }
  }

  fn generate_constant(&self, const_t: &ConstantType, vm_writer: &mut VmWriter) {
    match const_t {
      ConstantType::Integer(i) => {
        let constant = *i as usize;
        vm_writer.write_push(SegmentType::Constant, constant);
      }
      ConstantType::String(s) => {
        let s_const = s.clone();
        vm_writer.write_push(SegmentType::Constant, s_const.len());
        vm_writer.write_call("String.new".to_string(), 1);
        vm_writer.write_pop(SegmentType::Temp, 2);
        for c in s_const.as_bytes() {
          vm_writer.write_push(SegmentType::Temp, 2);
          vm_writer.write_push(SegmentType::Constant, *c as usize);
          vm_writer.write_call("String.appendChar".to_string(), 2);
        }
      }
      ConstantType::KeyWord(k) => {
        if k == "true" {
          vm_writer.write_push(SegmentType::Constant, 1);
          vm_writer.write_arithmetic('^');
        } else if k == "false" || k == "null" {
          vm_writer.write_push(SegmentType::Constant, 0);
        } else if k == "this" {
          vm_writer.write_push(SegmentType::Pointer, 0);
        }
      }
    };
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
  fn generate_term(&self, term_node: NodeId, state: &mut State) {
    match self.get_node_data(term_node) {
      OperationType::Term => true,
      _ => panic!(""),
    };
    let mut children = self.op_tree.get_children(term_node);
    let first_child = children.next().unwrap();
    let may_second_child = children.next();
    match self.get_node_data(first_child) {
      OperationType::Constant(const_t) => {
        self.generate_constant(const_t, &mut state.vm_writer);
      }
      OperationType::VarName(var_name) => {
        let may_var = state.get_variable(var_name);
        if may_var.is_none() {
          error!("error when compile, var {} not found", var_name);
          return;
        }
        let var = may_var.unwrap().clone();
        state
          .vm_writer
          .write_push(var.get_kind().into(), var.get_idx());
        if may_second_child.is_some() {
          // array index
          let next_child = children.next().unwrap();
          self.generate_expression(next_child, state);
          state.vm_writer.write_arithmetic('+');
          state.vm_writer.write_pop(SegmentType::Pointer, 1);
          state.vm_writer.write_push(SegmentType::That, 0);
        }
      }
      OperationType::Op(op) => {
        self.generate_term(may_second_child.unwrap(), state);
        state.vm_writer.write_arithmetic(*op);
      }
      OperationType::Bracket(_) => {
        self.generate_expression(may_second_child.unwrap(), state);
      }
      OperationType::SubroutineCall(_, _) => {
        self.generate_subroutine_call(term_node, state);
      }
      _ => {
        panic!(
          "Failed to compile term at {:?}",
          self.get_node_data(first_child)
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
   */
  fn generate_subroutine_call(&self, root: NodeId, state: &mut State) {
    let mut children = self.op_tree.get_children(root);
    let subroutine_call_node = children.next().unwrap();
    let expressions = children.skip(1).next().unwrap();
    let mut has_this = true;
    let subroutine_call = match self.get_node_data(subroutine_call_node) {
      OperationType::SubroutineCall(first_name, second_name) => match first_name {
        Some(first_name) => {
          let tmp_may_var = state.get_variable(&first_name);
          if tmp_may_var.is_some() {
            let var = tmp_may_var.unwrap().clone();
            // Push var into argument list as first parameter.
            state
              .vm_writer
              .write_push(var.get_kind().into(), var.get_idx());
            format!("{}.{}", var.get_type(), second_name)
          } else {
            has_this = false;
            format!("{}.{}", first_name, second_name)
          }
        }
        None => {
          // Push var into argument list as first parameter.
          state.vm_writer.write_push(SegmentType::Pointer, 0);
          format!("{}.{}", self.class_name.as_ref().unwrap(), second_name)
        }
      },
      _ => panic!(""),
    };
    let argc = self.handle_expression_list(expressions, state) + if has_this { 1 } else { 0 };
    state.vm_writer.write_call(subroutine_call, argc);
  }

  fn get_node_data(&self, node_id: NodeId) -> &OperationType {
    self.op_tree.get_node(node_id).get()
  }
}
