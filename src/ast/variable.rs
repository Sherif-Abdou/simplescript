use super::datatype::DataType;


#[derive(Hash)]
pub struct Variable {
    pub name: String,
    pub data_type: DataType,
}