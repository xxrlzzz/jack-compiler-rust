use std::collections::HashMap;

use crate::vm::commands::{Command, CommandType, OperandNum};
use crate::vm::segment_type::SegmentType;

pub struct AssembleCodeGenerator {
  cmp_counter: usize,
  current_file_static_count: usize,
  static_offset: usize,
  function_called_time: HashMap<String, usize>,
}

impl AssembleCodeGenerator {
  pub fn new() -> Self {
    Self {
      cmp_counter: 0,
      current_file_static_count: 0,
      static_offset: 0,
      function_called_time: HashMap::default(),
    }
  }

  pub fn finish_one_file(&mut self) {
    self.static_offset += self.current_file_static_count;
    self.current_file_static_count = 0;
  }

  fn get_current_cmp_str(&mut self) -> (String, String) {
    let ret = (
      format!("CMPSTART{}", self.cmp_counter),
      format!("CMPEND{}", self.cmp_counter),
    );
    self.cmp_counter += 1;
    ret
  }
  // static mut CMP_COUNTER: usize = 0;

  // fn get_current_cmp_str() -> (String, String) {
  //   unsafe {
  //     let ret = (
  //       format!("CMPSTART{}", CMP_COUNTER),
  //       format!("CMPEND{}", CMP_COUNTER),
  //     );
  //     CMP_COUNTER += 1;
  //     ret
  //   }
  // }

  pub fn init_env() -> Vec<String> {
    vec![
      String::from("@261"),
      // String::from("@256"),
      String::from("D=A"),
      String::from("@R0"),
      String::from("M=D"),
      // String::from("@-1"),
      // String::from("D=A"),
      // String::from("@R1"),
      // String::from("M=D"),
      // String::from("@-2"),
      // String::from("D=A"),
      // String::from("@R2"),
      // String::from("M=D"),
      // String::from("@-3"),
      // String::from("D=A"),
      // String::from("@R3"),
      // String::from("M=D"),
      // String::from("@-4"),
      // String::from("D=A"),
      // String::from("@R4"),
      // String::from("M=D"),
    ]
  }

  pub fn bootstrap() -> Vec<String> {
    // Start with Sys.init function.
    vec![String::from("@Sys.init"), String::from("0;JMP")]
  }

  pub fn get_asm(&mut self, cmd: Command) -> Vec<String> {
    match cmd.cmd_type() {
      CommandType::Push | CommandType::Pop => self.handle_stack(cmd),
      CommandType::Arithmetic(_op_count) => self.handle_arithmetic(cmd),
      CommandType::Goto => self.handle_goto(cmd),
      CommandType::Label => self.handle_label(cmd),
      CommandType::If => self.handle_condition_goto(cmd),
      CommandType::Function => self.handle_function(cmd),
      CommandType::Return => self.handle_return(cmd),
      CommandType::Call => self.handle_call(cmd),
      _ => {
        // panic!("unsupported command type {:?}", cmd.cmd_type());
        vec![]
      }
    }
  }

  fn forward_sp() -> Vec<String> {
    vec![String::from("@R0"), String::from("M=M+1")]
  }

  fn backward_sp() -> Vec<String> {
    vec![String::from("@R0"), String::from("M=M-1")]
  }

  fn load_sp_to_d() -> Vec<String> {
    let mut ret = AssembleCodeGenerator::backward_sp();
    ret.append(&mut vec![String::from("A=M"), String::from("D=M")]);
    ret
  }

  fn set_d_to_sp() -> Vec<String> {
    // mem[mem[sp]] = D
    // mem[sp]++
    let mut ret = vec![
      String::from("@R0"),
      String::from("A=M"),
      String::from("M=D"),
    ];
    ret.append(&mut AssembleCodeGenerator::forward_sp());
    ret
  }

  fn set_constant_to_sp(k: i32) -> Vec<String> {
    let mut ret = vec![format!("@{}", k), String::from("D=A")];
    ret.append(&mut AssembleCodeGenerator::set_d_to_sp());
    ret
  }

  fn handle_stack(&mut self, cmd: Command) -> Vec<String> {
    assert!(cmd.cmd_type() == CommandType::Push || cmd.cmd_type() == CommandType::Pop);
    let seg = SegmentType::new(&cmd.arg1().unwrap()[..]);
    let arg2 = cmd.arg2();
    let seg_str = format!("@{}", seg.to_asm_string());
    if cmd.cmd_type() == CommandType::Push {
      // load data from segment to sp

      // mem[mem[sp]] = data
      // mem[sp]++
      let mut ret = match seg {
        // data = mem[mem[seg] + i]
        SegmentType::Local | SegmentType::That | SegmentType::This | SegmentType::Argument => {
          vec![
            format!("@{}", arg2),
            format!("D=A"),
            seg_str,
            String::from("D=D+M"),
            String::from("A=D"),
            String::from("D=M"),
          ]
        }
        // data = mem[tmp + i]
        SegmentType::Temp => {
          vec![
            format!("@{}", arg2),
            format!("D=A"),
            seg_str,
            String::from("D=D+A"),
            String::from("A=D"),
            String::from("D=M"),
          ]
        }
        // data = mem[this/that]
        SegmentType::Pointer => {
          assert!(arg2 == 1 || arg2 == 0);
          let place = arg2 + 3; // this:3 that:4
          vec![format!("@{}", place), String::from("D=M")]
        }
        // data = constant
        SegmentType::Constant => {
          vec![format!("@{}", arg2), String::from("D=A")]
        }
        // data = static.i (variable)
        SegmentType::Static => {
          self.current_file_static_count += 1;
          vec![
            format!("@static{}", arg2 as usize + self.static_offset),
            String::from("D=M"),
          ]
        }
        _ => vec![],
      };
      if ret.len() > 0 {
        // mem[mem[sp]] = D
        // mem[sp]++
        ret.append(&mut AssembleCodeGenerator::set_d_to_sp());
      }
      ret
    } else {
      let mut ret = match seg {
        // pop data from sp to segment
        // target:
        // mem[sp]--
        // mem[addr] = mem[mem[sp]]
        SegmentType::Local | SegmentType::That | SegmentType::This | SegmentType::Argument => {
          vec![
            format!("@{}", arg2),
            format!("D=A"),
            seg_str.clone(),
            String::from("D=D+M"),
          ]
        }
        SegmentType::Temp => {
          vec![
            format!("@{}", arg2),
            format!("D=A"),
            seg_str.clone(),
            String::from("D=D+A"),
          ]
        }
        // push to this/that
        SegmentType::Pointer => {
          assert!(arg2 == 1 || arg2 == 0);
          let place = arg2 + 3; // this:3 that:4
          vec![format!("@{}", place), String::from("D=A")]
        }
        // data = static.i (variable)
        SegmentType::Static => {
          vec![
            format!("@static{}", arg2 as usize + self.static_offset),
            String::from("D=A"),
          ]
        }
        _ => vec![],
      };
      if ret.len() > 0 {
        // D = mem[seg] + i
        // =>
        // mem[sp]--
        // mem[D] = mem[mem[sp]]
        ret.append(&mut AssembleCodeGenerator::backward_sp());
        ret.append(&mut vec![
          String::from("@R13"),
          String::from("M=D"),
          String::from("@R0"),
          String::from("A=M"), // sp point to nothing, sp-1 is what we want
          String::from("D=M"),
          String::from("@R13"),
          String::from("A=M"),
          String::from("M=D"),
        ]);
      }
      ret
    }
  }

  fn handle_arithmetic(&mut self, cmd: Command) -> Vec<String> {
    // assert!(CommandType::Arithmetic == cmd.cmd_type());
    let mut ret = AssembleCodeGenerator::load_sp_to_d();
    let operand_num = match cmd.cmd_type() {
      CommandType::Arithmetic(operand_num) => operand_num,
      _ => return vec![],
    };
    // let CommandType::Arithmetic(operand_num) = cmd.cmd_type();
    ret.append(&mut vec![String::from("@R13"), String::from("M=D")]);
    if operand_num == OperandNum::TwoOperand {
      ret.append(&mut AssembleCodeGenerator::load_sp_to_d());
      ret.append(&mut vec![String::from("@R13")]);
    }
    match &cmd.arg1().unwrap()[..] {
      "add" => {
        ret.push(String::from("D=D+M"));
      }
      "sub" => {
        ret.push(String::from("D=D-M"));
      }
      "and" => {
        ret.push(String::from("D=D&M"));
      }
      "or" => {
        ret.push(String::from("D=D|M"));
      }
      //single operand
      "neg" => {
        ret.push(String::from("D=-M"));
      }
      "not" => {
        ret.push(String::from("D=!M"));
      }

      "eq" => {
        let (start, end) = self.get_current_cmp_str();
        ret.push(String::from("D=D-M"));
        ret.push(format!("@{}", start));
        ret.push(String::from("D;JNE"));
        ret.push(String::from("D=-1"));
        ret.push(format!("@{}", end));
        ret.push(String::from("0;JMP"));
        ret.push(format!("({})", start));
        ret.push(String::from("D=0"));
        ret.push(format!("({})", end));
      }
      "gt" => {
        let (start, end) = self.get_current_cmp_str();
        ret.push(String::from("D=D-M"));
        ret.push(format!("@{}", start));
        ret.push(String::from("D;JLE"));
        ret.push(String::from("D=-1"));
        ret.push(format!("@{}", end));
        ret.push(String::from("0;JMP"));
        ret.push(format!("({})", start));
        ret.push(String::from("D=0"));
        ret.push(format!("({})", end));
      }
      "lt" => {
        let (start, end) = self.get_current_cmp_str();
        ret.push(String::from("D=D-M"));
        ret.push(format!("@{}", start));
        ret.push(String::from("D;JGE"));
        ret.push(String::from("D=-1"));
        ret.push(format!("@{}", end));
        ret.push(String::from("0;JMP"));
        ret.push(format!("({})", start));
        ret.push(String::from("D=0"));
        ret.push(format!("({})", end));
      }
      _ => (),
    };
    ret.append(&mut vec![
      // save d = r13
      // String::from("@R13"),
      // String::from("M=D"),
      String::from("@R0"),
      String::from("A=M"),
      String::from("M=D"),
    ]);
    ret.append(&mut AssembleCodeGenerator::forward_sp());
    ret
  }

  fn handle_goto(&self, cmd: Command) -> Vec<String> {
    assert_eq!(cmd.cmd_type(), CommandType::Goto);
    let label = cmd.arg1().unwrap().to_uppercase();
    vec![format!("@{}", label), String::from("0;JMP")]
  }

  fn handle_label(&self, cmd: Command) -> Vec<String> {
    assert_eq!(cmd.cmd_type(), CommandType::Label);
    let label = cmd.arg1().unwrap().to_uppercase();
    vec![format!("({})", label)]
  }

  fn handle_condition_goto(&self, cmd: Command) -> Vec<String> {
    assert_eq!(cmd.cmd_type(), CommandType::If);
    let label = cmd.arg1().unwrap().to_uppercase();
    let mut ret = AssembleCodeGenerator::load_sp_to_d();
    ret.append(&mut vec![
      // String::from("D=D+1"),
      format!("@{}", label),
      String::from("D;JNE"),
    ]);
    ret
  }

  fn generate_return_addr_for_call(&mut self, function_name: &String) -> String {
    // TODO: global store return addr
    let new_time = match &self.function_called_time.get(function_name) {
      None => {
        self.function_called_time.insert(function_name.clone(), 1);
        1
      }
      Some(time) => {
        let new_time = 1 + **time;
        self
          .function_called_time
          .insert(function_name.clone(), new_time);
        new_time
      }
    };
    format!("{}_{}", function_name.clone(), new_time)
  }

  fn handle_call(&mut self, cmd: Command) -> Vec<String> {
    assert_eq!(cmd.cmd_type(), CommandType::Call);
    let ret_addr = self.generate_return_addr_for_call(&cmd.arg1().unwrap());
    // let ret_addr = unsafe { generate_return_addr_for_call(&cmd.arg1().unwrap()) };
    let n = cmd.arg2();
    let mut ret = vec![format!("@{}", ret_addr), String::from("D=A")];
    ret.append(&mut AssembleCodeGenerator::set_d_to_sp());
    for i in 1..5 {
      ret.append(&mut vec![format!("@{}", i), String::from("D=M")]);
      ret.append(&mut AssembleCodeGenerator::set_d_to_sp());
    }
    ret.append(&mut vec![
      String::from("@0"),
      String::from("D=M"),
      format!("@{}", n),
      String::from("D=D-A"),
      String::from("@5"),
      String::from("D=D-A"),
      String::from("@2"),
      String::from("M=D"), // ARG=SP-n-5
      String::from("@0"),
      String::from("D=M"),
      String::from("@1"),
      String::from("M=D"), // LCL = SP
      format!("@{}", cmd.arg1().unwrap()),
      String::from("0;JMP"),     // goto f
      format!("({})", ret_addr), // (ret-addr)
    ]);
    ret
  }

  fn handle_function(&self, cmd: Command) -> Vec<String> {
    assert_eq!(cmd.cmd_type(), CommandType::Function);
    let mut ret = vec![format!("({})", cmd.arg1().unwrap())];
    for _ in 0..cmd.arg2() {
      ret.append(&mut AssembleCodeGenerator::set_constant_to_sp(0));
    }
    ret
  }

  fn handle_return(&self, cmd: Command) -> Vec<String> {
    assert_eq!(cmd.cmd_type(), CommandType::Return);
    let mut ret = vec![
      String::from("@1"),
      String::from("D=M"),
      String::from("@R13"),
      String::from("M=D"), // FRAME = LCL,
      String::from("@5"),
      String::from("D=D-A"),
      // String::from("@R14"),
      // String::from("M=D"),
      String::from("A=D"),
      String::from("D=M"),
      String::from("@R14"),
      String::from("M=D"),
    ];
    ret.append(&mut AssembleCodeGenerator::load_sp_to_d());
    ret.append(&mut vec![
      String::from("@2"),
      String::from("A=M"),
      String::from("M=D"), // *arg = pop()
      String::from("@2"),
      String::from("D=M"),
      String::from("@0"),
      String::from("M=D+1"), // SP = arg+1
      String::from("@R13"),
      String::from("AM=M-1"),
      String::from("D=M"),
      String::from("@4"),
      String::from("M=D"), // THAT = *(FRAME-1)
      String::from("@R13"),
      String::from("AM=M-1"),
      String::from("D=M"),
      String::from("@3"),
      String::from("M=D"), // THIS = *(FRAME-2)
      String::from("@R13"),
      String::from("AM=M-1"),
      String::from("D=M"),
      String::from("@2"),
      String::from("M=D"), // ARG = *(FRAME-3)
      String::from("@R13"),
      String::from("AM=M-1"),
      String::from("D=M"),
      String::from("@1"),
      String::from("M=D"), // LCL = *(FRAME-4)
      String::from("@R14"),
      String::from("A=M"),
      String::from("0;JMP"), // goto *(FRAME-5)
    ]);

    ret
  }
}
