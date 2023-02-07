use std::{collections::HashMap, cell::RefCell};

use inkwell::{context::Context, module::Module, execution_engine::JitFunction};

use crate::{ast::Compiler, parsing::Parser};

type MainFunc = unsafe extern "C" fn() -> u64;

pub fn run(file: String) {
    let context = Context::create();
    let module = context.create_module("main");
    let engine = module.create_jit_execution_engine(inkwell::OptimizationLevel::None).unwrap();

    let compiler = Compiler {
        context: &context,
        module,
        builder: context.create_builder(),
        variable_table: RefCell::new(HashMap::new()),
    };

    
    let mut parser = Parser::new(file);
    let res = parser.parse().unwrap();
    res.visit(&compiler);

    unsafe {
        let main: JitFunction<MainFunc> = engine.get_function("main").unwrap();
        println!("{}", main.call());
    }
}