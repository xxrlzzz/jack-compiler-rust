use std::cell::RefCell;
use std::rc::Rc;

use clap::Parser;

use jack_compiler::code_writer::CodeWriter;
use jack_compiler::compiler::Compiler;
use jack_compiler::operation::tree::OperationTree;
use jack_compiler::xml::token_xml_generator::TokenXMLGenerator;

fn tokenize_one_file(file: &str, token_xml: bool, vm_xml: bool) {
    let parser = jack_compiler::parser::Parser::new(file);
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
}

fn main() {
    let args = Args::parse();
    let file = args.path;
    if file.ends_with(".jack") {
        tokenize_one_file(&file, args.debug_token, args.debug_vm);
    } else {
        match std::fs::read_dir(file) {
            Ok(dir) => {
                for path in dir {
                    let path = path.expect("Failed to read file in directory");
                    let path = format!("{}", path.path().display());
                    if path.ends_with(".jack") {
                        println!("DEBUG: reading file {}", path);
                        tokenize_one_file(path.as_str(), args.debug_token, args.debug_vm);
                    }
                }
            }
            Err(e) => {
                panic!("Read input directory: {}", e);
            }
        }
    }
}
