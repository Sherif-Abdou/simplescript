use std::path::Path;
use std::{cell::RefCell, collections::HashMap};

use inkwell::{context::Context, execution_engine::JitFunction};

use crate::{ast::Compiler, parsing::Parser};

type MainFunc = unsafe extern "C" fn() -> i8;

pub fn run(file: String) {
    let context = Context::create();
    let module = context.create_module("main");
    let engine = module
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .unwrap();

    let mut parser = Parser::new(file);
    let res = parser.parse().unwrap();
    let compiler = Compiler {
        context: &context,
        module,
        builder: context.create_builder(),
        variable_table: RefCell::new(HashMap::new()),
        variable_type: RefCell::new(HashMap::new()),
        function_table: RefCell::new(HashMap::new()),
        current_function_params: RefCell::new(HashMap::new()),
        data_types: parser.data_types.clone(),
    };

    res.visit(&compiler);
    compiler
        .module
        .print_to_file(Path::new("./test/output.txt"))
        .unwrap();
    unsafe {
        let main: JitFunction<MainFunc> = engine.get_function("main").unwrap();
        println!("Result: {:?}", main.call());
    }
}
