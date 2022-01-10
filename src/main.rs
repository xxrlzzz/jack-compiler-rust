use std::cell::RefCell;
use std::rc::Rc;

use clap::Parser;

use jack_compiler::code_writer::CodeWriter;
use jack_compiler::common::{new_output, panic_writer, OutputTarget};
use jack_compiler::compiler::Compiler;
use jack_compiler::operation::tree::OperationTree;
use jack_compiler::vm::vm_translator::AssembleCodeGenerator;
use jack_compiler::xml::token_xml_generator::TokenXMLGenerator;

fn tokenize_one_file(file: &str, token_xml: bool, vm_xml: bool) {
  let parser = jack_compiler::parser::jack::Parser::new(file);
  if token_xml {
    let out_file = String::from(file.strip_suffix(".jack").unwrap()) + "T.xml.d";
    let generator = TokenXMLGenerator::new(out_file.as_str(), parser);
    generator.run();
  } else if vm_xml {
    let out_file = String::from(file.strip_suffix(".jack").unwrap()) + ".xml.d";
    let compiler = Compiler::new(out_file.as_str(), parser);
    if let Some(r) = compiler.run() {
      println!("compile failed {}", r);
    }
  } else {
    let base_file_name = file.strip_suffix(".jack").unwrap();
    let vm_file_name = format!("{}.vm", base_file_name);
    let class_name = String::from(base_file_name.rsplit("/").next().unwrap());
    let op_tree = Rc::new(RefCell::new(OperationTree::new(class_name)));
    let compiler = Compiler::new_with_generator(op_tree.clone(), parser);
    if let Some(r) = compiler.run() {
      println!("compile failed {}", r);
    }
    // println!("{}", op_tree.borrow());
    let code_writer = CodeWriter::new(&vm_file_name[..], op_tree.take());
    code_writer.generate_vm_code();
  }
}

fn handle_jack(file: String, token_xml: bool, vm_xml: bool) {
  if file.ends_with(".jack") {
    tokenize_one_file(&file, token_xml, vm_xml);
  } else {
    match std::fs::read_dir(file) {
      Ok(dir) => {
        for path in dir {
          let path = path.expect("Failed to read file in directory");
          let path = format!("{}", path.path().display());
          if path.ends_with(".jack") {
            println!("DEBUG: reading file {}", path);
            tokenize_one_file(path.as_str(), token_xml, vm_xml);
          }
        }
      }
      Err(e) => {
        panic!("Read input directory: {}", e);
      }
    }
  }
}

fn write_commands(output: OutputTarget, cmds: Vec<String>) {
  for command in cmds {
    panic_writer(command, output.clone().borrow_mut());
    panic_writer("\n".to_string(), output.clone().borrow_mut());
  }
}
fn translate_one_file(file: String, out_file: OutputTarget, writer: &mut AssembleCodeGenerator) {
  // println!("DEBUG: handle file {}", file);
  let mut parser = jack_compiler::parser::hack::Parser::new(&file);
  while let Some(cmd) = parser.next() {
    let translate = writer.get_asm(cmd);
    write_commands(out_file.clone(), translate);
  }
  writer.finish_one_file();
}

fn handle_vm(file: String) {
  let mut writer = AssembleCodeGenerator::new();
  if file.ends_with(".vm") {
    let out_file = String::from(file.strip_suffix(".vm").unwrap()) + ".asm";
    // let mut output = BufWriter::new(File::create(out_file).expect("Open output file failed"));
    let output = new_output(&out_file[..]);
    write_commands(output.clone(), AssembleCodeGenerator::init_env());
    translate_one_file(file, output, &mut writer);
  } else {
    // Treat file as directory
    match std::fs::read_dir(file.clone()) {
      Ok(dir) => {
        let out_file = format!("{}.asm", file);
        // let mut output = BufWriter::new(File::create(out_file).expect("Open output file failed"));
        let output = new_output(&out_file[..]);
        write_commands(output.clone(), AssembleCodeGenerator::init_env());
        write_commands(output.clone(), AssembleCodeGenerator::bootstrap());
        for path in dir {
          let path = path.expect("Failed to read file in directory");
          let path = format!("{}", path.path().display());
          println!("DEBUG: reading file {}", path);
          if path.ends_with(".vm") {
            translate_one_file(format!("{}", path), output.clone(), &mut writer);
          }
        }
      }
      Err(e) => {
        panic!("Read input directory: {}", e);
      }
    }
  }
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
  // File or Directory to compile.
  #[clap(short, long)]
  path: String,

  #[clap(long)]
  debug_token: bool,

  #[clap(long)]
  debug_vm: bool,

  #[clap(long)]
  translate_vm: bool,
}

fn main() {
  let args = Args::parse();
  let file = args.path;
  if args.translate_vm {
    handle_vm(file);
  } else {
    handle_jack(file, args.debug_token, args.debug_vm);
  }
}
