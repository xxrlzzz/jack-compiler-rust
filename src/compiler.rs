use std::cell::RefCell;
use std::rc::Rc;

use crate::operation::*;
use crate::parser::Parser;
use crate::token::{is_keyword_constant, is_unary_operation, Token};
use crate::xml::operation_xml_generator::{OperationXMLGenerator, RAIIWriter};

pub trait WritableStack {
  fn push(&mut self, node_type: OperationType);
  fn pop(&mut self);
}

pub type WriteTarget = Rc<RefCell<dyn WritableStack>>;

pub struct Compiler {
  generator: WriteTarget,
  parser: Parser,
}

impl Compiler {
  pub fn new(source: &str, parser: Parser) -> Self {
    Self {
      generator: Rc::new(RefCell::new(OperationXMLGenerator::new(source))),
      parser: parser,
    }
  }

  pub fn new_with_generator(generator: WriteTarget, parser: Parser) -> Self {
    Self { generator, parser }
  }

  pub fn run(mut self) -> Option<String> {
    // CompileClass
    //  CompileClassVarDec
    //  CompileSubroutine
    if !self.compile_class() {
      let last_token_info = self.parser.get_last_token_descriptor();
      return Some(format!("{:?}", last_token_info));
    }
    return None;
  }

  fn compile_class(&mut self) -> bool {
    if !self.take_keyword("class".to_string()) {
      return false;
    }
    let class_name = self.take_identifier();
    if class_name.is_none() {
      return false;
    }
    let _w = RAIIWriter::new(
      self.generator.clone(),
      OperationType::Class(class_name.unwrap()),
    );
    let compile_class_content = |compiler: &mut Compiler| {
      return compiler.compile_class_var_dec() && compiler.compile_subroutine();
    };
    if !self.compile_symbol_wrapper('{', '}', compile_class_content) {
      return false;
    }
    true
  }

  fn compile_class_var_dec(&mut self) -> bool {
    let class_var_type: VarScope = if self.try_take_keyword("static".to_string()) {
      VarScope::Static
    } else if self.try_take_keyword("field".to_string()) {
      VarScope::Field
    } else {
      return true;
    };
    {
      let _w = RAIIWriter::new(
        self.generator.clone(),
        OperationType::ClassVarDec(class_var_type),
      );
      if !self.compile_var_type_and_name(true, true) {
        return false;
      }
    }
    self.compile_class_var_dec()
  }

  fn compile_subroutine(&mut self) -> bool {
    let subroutine_type = if self.try_take_keyword("constructor".to_string()) {
      SubroutineType::Constructor
    } else if self.try_take_keyword("function".to_string()) {
      SubroutineType::Function
    } else if self.try_take_keyword("method".to_string()) {
      SubroutineType::Method
    } else {
      return true;
    };
    {
      let _w = RAIIWriter::new(
        self.generator.clone(),
        OperationType::SubroutineDec(subroutine_type),
      );
      if self.try_take_keyword("void".to_string()) {
        let _w2 = RAIIWriter::new(self.generator.clone(), OperationType::Void);
      } else if let Some((ret_type, is_keyword)) = self.try_take_type() {
        let _w2 = RAIIWriter::new(
          self.generator.clone(),
          OperationType::Type(ret_type, is_keyword),
        );
      } else {
        return false;
      }
      let func_name = match self.take_identifier() {
        None => return false,
        Some(func_name) => func_name,
      };
      let _w3 = RAIIWriter::new(self.generator.clone(), OperationType::VarName(func_name));
      if !self.compile_symbol_wrapper('(', ')', Compiler::compile_parameter_list) {
        return false;
      }
      let _w4 = RAIIWriter::new(self.generator.clone(), OperationType::SubroutineBody);
      let compile_func_content = |compiler: &mut Compiler| {
        return compiler.compile_var_dec() && compiler.compile_statements();
      };
      if !self.compile_symbol_wrapper('{', '}', compile_func_content) {
        return false;
      }
    }
    // Repeating compile
    self.compile_subroutine()
  }

  fn compile_var_dec(&mut self) -> bool {
    if !self.try_take_keyword("var".to_string()) {
      return true;
    }
    {
      let _w = RAIIWriter::new(self.generator.clone(), OperationType::VarDec);
      let r = self.compile_var_type_and_name(true, true);
      if !r {
        return false;
      }
    }
    return self.compile_var_dec();
  }

  fn compile_parameter_list(&mut self) -> bool {
    {
      let _w = RAIIWriter::new(self.generator.clone(), OperationType::ParameterList);

      self.compile_var_type_and_name(false, false)
    }
  }

  fn compile_statement(&mut self) -> (bool, bool) {
    if self.try_take_keyword("let".to_string()) {
      if !self.compile_let_statement() {
        return (false, false);
      }
    } else if self.try_take_keyword("if".to_string()) {
      if !self.compile_if_statement() {
        return (false, false);
      }
    } else if self.try_take_keyword("while".to_string()) {
      if !self.compile_while_statement() {
        return (false, false);
      }
    } else if self.try_take_keyword("do".to_string()) {
      if !self.compile_do_statement() {
        return (false, false);
      }
    } else if self.try_take_keyword("return".to_string()) {
      let _w = RAIIWriter::new(self.generator.clone(), OperationType::ReturnStatement);
      if !self.try_compile_expression().0 {
        return (false, false);
      }
      if !self.take_symbol(';') {
        return (false, false);
      }
    } else {
      return (true, false);
    }
    return (true, true);
  }

  fn compile_statements(&mut self) -> bool {
    let _w = RAIIWriter::new(self.generator.clone(), OperationType::Statements);
    // println!("DEBUG start statements");
    loop {
      let (success, has) = self.compile_statement();
      if !success {
        return false;
      }
      if !has {
        break;
      }
    }
    // println!("DEBUG end statements");
    true
  }

  fn compile_let_statement(&mut self) -> bool {
    if let Some(val) = self.take_identifier() {
      let _w = RAIIWriter::new(self.generator.clone(), OperationType::LetStatement(val));
      if self.try_take_symbol('[') {
        let _w2 = RAIIWriter::new(
          self.generator.clone(),
          OperationType::Bracket(BracketType::from_char('[')),
        );
        if !self.compile_expression() || !self.take_symbol(']') {
          return false;
        }
      }
      {
        let _w3 = RAIIWriter::new(self.generator.clone(), OperationType::Op('='));
      }
      if !self.take_symbol('=') || !self.compile_expression() || !self.take_symbol(';') {
        return false;
      }
      return true;
    }
    false
  }

  fn compile_if_statement(&mut self) -> bool {
    let _w = RAIIWriter::new(self.generator.clone(), OperationType::IfStatement);
    if !self.compile_symbol_wrapper('(', ')', Compiler::compile_expression) {
      return false;
    }
    if !self.compile_symbol_wrapper('{', '}', Compiler::compile_statements) {
      return false;
    }
    if self.try_take_keyword("else".to_string()) {
      let _w2 = RAIIWriter::new(self.generator.clone(), OperationType::Else);
      if !self.compile_symbol_wrapper('{', '}', Compiler::compile_statements) {
        return false;
      }
    }
    true
  }

  fn compile_while_statement(&mut self) -> bool {
    let _w = RAIIWriter::new(self.generator.clone(), OperationType::WhileStatement);
    if !self.compile_symbol_wrapper('(', ')', Compiler::compile_expression) {
      return false;
    }
    if !self.compile_symbol_wrapper('{', '}', Compiler::compile_statements) {
      return false;
    }
    true
  }

  fn compile_do_statement(&mut self) -> bool {
    let _w = RAIIWriter::new(self.generator.clone(), OperationType::DoStatement);
    let some_name = self.take_identifier();
    if some_name.is_none() {
      return false;
    }
    let some_name = some_name.unwrap();
    if self.try_take_symbol('.') {
      if let Some(func_name) = self.take_identifier() {
        let _w2 = RAIIWriter::new(
          self.generator.clone(),
          OperationType::SubroutineCall(Some(some_name), func_name),
        );
        if !self.compile_symbol_wrapper('(', ')', Compiler::compile_expression_list) {
          return false;
        }
      } else {
        return false;
      }
    } else {
      let _w2 = RAIIWriter::new(
        self.generator.clone(),
        OperationType::SubroutineCall(None, some_name),
      );
      if !self.compile_symbol_wrapper('(', ')', Compiler::compile_expression_list) {
        return false;
      }
    }
    if !self.take_symbol(';') {
      return false;
    }
    true
  }

  fn compile_expression_list(&mut self) -> bool {
    let _w = RAIIWriter::new(self.generator.clone(), OperationType::ExpressionList);
    let (success, has) = self.try_compile_expression();
    if !success {
      return false;
    }
    if !has {
      return true;
    }
    while self.try_take_symbol(',') {
      {
        let _w2 = RAIIWriter::new(self.generator.clone(), OperationType::ListConcat);
      }
      if !self.compile_expression() {
        return false;
      }
    }
    return true;
  }

  fn compile_expression(&mut self) -> bool {
    let _w = RAIIWriter::new(self.generator.clone(), OperationType::Expression);
    if !self.compile_term() {
      return false;
    }
    loop {
      if let Some(op) = self.try_take_op() {
        let _w2 = RAIIWriter::new(self.generator.clone(), OperationType::Op(op));
        if !self.compile_term() {
          return false;
        }
      } else {
        break;
      }
    }
    true
  }

  fn compile_term(&mut self) -> bool {
    let _w = RAIIWriter::new(self.generator.clone(), OperationType::Term);
    let token = self.parser.next();
    let token = token.unwrap_or(Token::None);
    let matched = match token {
      Token::IntVal(v) => {
        let _w2 = RAIIWriter::new(
          self.generator.clone(),
          OperationType::Constant(ConstantType::Integer(v)),
        );
        true
      }
      Token::StringVal(v) => {
        let _w2 = RAIIWriter::new(
          self.generator.clone(),
          OperationType::Constant(ConstantType::String(v)),
        );
        true
      }
      Token::KeyWord(keyword) => {
        if !is_keyword_constant(&keyword) {
          return false;
        }
        let _w2 = RAIIWriter::new(
          self.generator.clone(),
          OperationType::Constant(ConstantType::KeyWord(keyword)),
        );
        true
      }
      Token::Symbol(s) => {
        if is_unary_operation(s) {
          let op = if s == '-' {
            // NOTE turn negative '-' to '^'
            '^'
          } else {
            s
          };
          {
            let _w2 = RAIIWriter::new(self.generator.clone(), OperationType::Op(op));
          }
          self.compile_term()
        } else if s == '(' {
          let _w2 = RAIIWriter::new(
            self.generator.clone(),
            OperationType::Bracket(BracketType::from_char('(')),
          );
          if !self.compile_expression() || !self.take_symbol(')') {
            false
          } else {
            true
          }
        } else {
          false
        }
      }
      Token::Identifier(identifier) => {
        if self.try_take_symbol('[') {
          let _w2 = RAIIWriter::new(self.generator.clone(), OperationType::VarName(identifier));
          let _w3 = RAIIWriter::new(
            self.generator.clone(),
            OperationType::Bracket(BracketType::from_char('[')),
          );
          if !self.compile_expression() || !self.take_symbol(']') {
            return false;
          }
        } else if self.try_take_symbol('.') {
          let func_name = self.take_identifier();
          if func_name.is_none() {
            return false;
          }
          let func_name = func_name.unwrap();
          {
            let _w2 = RAIIWriter::new(
              self.generator.clone(),
              OperationType::SubroutineCall(Some(identifier), func_name),
            );
          }
          if !self.compile_symbol_wrapper('(', ')', Compiler::compile_expression_list) {
            return false;
          }
        } else {
          let _w2 = RAIIWriter::new(self.generator.clone(), OperationType::VarName(identifier));
        }
        true
      }
      Token::None => false,
    };
    matched
  }

  // compile error, has expression
  fn try_compile_expression(&mut self) -> (bool, bool) {
    if let Some(token) = self.parser.peek() {
      if let Token::Symbol(s) = token {
        if s == ')' || s == ';' {
          return (true, false);
        }
      }
    }
    let r = self.compile_expression();
    (r, r)
  }

  // Compile [start, func, end] like ( expression list )
  fn compile_symbol_wrapper<P>(&mut self, start: char, end: char, mut func: P) -> bool
  where
    P: FnMut(&mut Compiler) -> bool,
  {
    let _w2 = RAIIWriter::new(
      self.generator.clone(),
      OperationType::Bracket(BracketType::from_char(start)),
    );
    return self.take_symbol(start) && func(self) && self.take_symbol(end);
  }

  fn compile_var_type_and_name(&mut self, need_semicolons: bool, type_once: bool) -> bool {
    let res = self.try_take_type();
    if res.is_none() {
      return true;
    }
    let (var_type, is_keyword) = res.unwrap();
    {
      let _w = RAIIWriter::new(
        self.generator.clone(),
        OperationType::Type(var_type, is_keyword),
      );
      if let Some(var_name) = self.take_identifier() {
        if type_once {
          let mut var_names = vec![var_name];
          while self.try_take_symbol(',') {
            if let Some(next_var_name) = self.take_identifier() {
              var_names.push(next_var_name);
            } else {
              return false;
            }
          }
          let _w2 = RAIIWriter::new(
            self.generator.clone(),
            OperationType::VarNameList(var_names),
          );
        } else {
          {
            let _w2 = RAIIWriter::new(self.generator.clone(), OperationType::VarName(var_name));
          }
          while self.try_take_symbol(',') {
            {
              let _w3 = RAIIWriter::new(self.generator.clone(), OperationType::ListConcat);
            }
            let next_type = self.try_take_type();
            let next_var = self.take_identifier();
            if next_type.is_none() || next_var.is_none() {
              return false;
            }
            let next_type = next_type.unwrap();
            let next_var = next_var.unwrap();
            let _w4 = RAIIWriter::new(
              self.generator.clone(),
              OperationType::Type(next_type.0, next_type.1),
            );
            let _w5 = RAIIWriter::new(self.generator.clone(), OperationType::VarName(next_var));
          }
        }
      } else {
        return false;
      }
    }
    if need_semicolons {
      self.take_symbol(';')
    } else {
      true
    }
  }

  fn take_keyword(&mut self, keyword: String) -> bool {
    let class_token = self.parser.next();
    let class_token = class_token.unwrap_or(Token::None);
    if let Token::KeyWord(actual_keyword) = class_token {
      if actual_keyword == keyword {
        return true;
      }
      return false;
    }
    return false;
  }

  fn try_take_keyword(&mut self, keyword: String) -> bool {
    let token = self.parser.take_if(|token: &Token| {
      if let Token::KeyWord(actual_keyword) = token {
        return *actual_keyword == keyword;
      }
      false
    });
    if let Some(_) = token {
      return true;
    }
    return false;
  }

  fn take_symbol(&mut self, symbol: char) -> bool {
    let class_token = self.parser.next();
    let class_token = class_token.unwrap_or(Token::None);
    if let Token::Symbol(actual_symbol) = class_token {
      if actual_symbol == symbol {
        return true;
      }
      return false;
    }
    return false;
  }

  fn try_take_symbol(&mut self, symbol: char) -> bool {
    let token = self.parser.take_if(|token: &Token| {
      if let Token::Symbol(actual_symbol) = token {
        return *actual_symbol == symbol;
      }
      false
    });
    if let Some(_) = token {
      return true;
    }
    return false;
  }

  fn take_identifier(&mut self) -> Option<String> {
    let token = self.parser.next().unwrap_or(Token::None);
    if let Token::Identifier(identifier) = token {
      return Some(identifier);
    }
    None
  }

  fn try_take_identifier(&mut self) -> Option<String> {
    if let Some(Token::Identifier(token)) = self.parser.peek() {
      self.parser.next();
      return Some(token);
    }
    None
  }

  fn try_take_type(&mut self) -> Option<(String, bool)> {
    if self.try_take_keyword("int".to_string()) {
      return Some(("int".to_string(), true));
    } else if self.try_take_keyword("char".to_string()) {
      return Some(("char".to_string(), true));
    } else if self.try_take_keyword("boolean".to_string()) {
      return Some(("boolean".to_string(), true));
    } else {
      if let Some(id) = self.try_take_identifier() {
        return Some((id, false));
      } else {
        None
      }
    }
  }

  fn try_take_op(&mut self) -> Option<char> {
    static OP_LIST: &'static [char] = &['+', '-', '*', '/', '&', '|', '<', '>', '='];
    for op in OP_LIST {
      if self.try_take_symbol(*op) {
        return Some(*op);
      }
    }
    return None;
  }
}
