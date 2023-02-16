use std::{hash::Hash, collections::HashMap};

use inkwell::{types::{BasicType, BasicTypeEnum}, AddressSpace};

use super::Compiler;

type DataTypeVector = Vec<Box<DataType>>;
type NameMap = HashMap<String, u64>;

#[derive(Clone)]
pub enum DataTypeEnum {
    Primitive,
    Array(Box<DataType>, u64),
    Struct(DataTypeVector, NameMap),
    Pointer(Box<DataType>),
}

#[derive(Clone)]
pub struct DataType {
    pub symbol: String,
    pub value: DataTypeEnum,
}

impl DataType {
    pub fn produce_llvm_type<'a>(&self, compiler: &'a Compiler) -> Box<dyn BasicType<'a> + 'a> {
        match &self.value {
            DataTypeEnum::Primitive => self.produce_primitive_llvm_type(compiler),
            DataTypeEnum::Array(ref interior, len) => Box::new(interior.produce_llvm_type(compiler).array_type(*len as u32)),
            DataTypeEnum::Struct(ref data_types, ref names) => self.produce_struct_llvm_type(compiler, data_types, names),
            DataTypeEnum::Pointer(ref interior) => Box::new(interior.produce_llvm_type(compiler).ptr_type(AddressSpace::default())),
        }
    }

    fn produce_primitive_llvm_type<'a>(&self, compiler: &'a Compiler) -> Box<dyn BasicType<'a> + 'a> {
        match self.symbol.as_str() {
            "i64" => Box::new(compiler.context.i64_type()),
            "f64" => Box::new(compiler.context.f64_type()),
            "bool" => Box::new(compiler.context.bool_type()),
            "char" => Box::new(compiler.context.i8_type()),
            _ => panic!("Unidentified primitive")
        }
    }

    fn produce_struct_llvm_type<'a>(&self, compiler: &'a Compiler, data_types: &DataTypeVector, names: &NameMap) -> Box<dyn BasicType<'a> + 'a> {
        let v: Vec<BasicTypeEnum> = data_types.iter().map(|v| v.produce_llvm_type(compiler).as_basic_type_enum()).collect();
        let slice = v.as_slice();
        let struct_type = compiler.context.struct_type(slice, false);
        Box::new(struct_type)
    }
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
    }
}

impl Hash for DataType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
    }
}