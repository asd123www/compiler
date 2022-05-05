pub const VOID: i32 = 0;
pub const CONSTANT_INT: i32 = 1;
pub const VARIABLE_INT: i32 = 2;
pub const VARIABLE_ARRAY: i32 = 3;
pub const PARAMETER_ARRAY: i32 = 4;
pub const ARRAY_DIMENSION: i32 = 5;

pub const TYPE_BITS: i32 = 3;


// exp_res_id:
//     if is_constant == true: store constant value.
//     else                    store variable ID.
pub struct ExpRetType {
    pub size: i32,
    pub program: String,
    pub exp_res_id: i32,
    pub is_constant: bool,
}


pub struct DeclRetType {
    pub size: i32,
    pub program: String,
    pub flag: i32,
}

pub struct BodyRetType {
    pub size: i32,
    pub program: String,
    pub exp_res_id: i32,
}


pub struct InitRetType {
    pub size: i32,
    pub program: String,
    pub is_allzero: bool,
    pub val: Vec<(bool, i32)>,
}