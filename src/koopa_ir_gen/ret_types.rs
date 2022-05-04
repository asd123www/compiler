
// pub const REGULAR_STATE: i32 = -1;
// pub const RETURN_STATE: i32 = -2;
// pub const JUMP_STATE: i32 = -3;


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