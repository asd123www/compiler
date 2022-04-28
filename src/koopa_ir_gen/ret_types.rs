
pub const REGULAR_STATE: i32 = -1;
pub const RETURN_STATE: i32 = -2;
pub const JUMP_STATE: i32 = -3;

pub struct ExpRetType {
    pub size: i32,
    pub program: String,
    pub exp_res_id: i32,
}


pub struct DeclRetType {
    pub size: i32,
    pub program: String,
    pub flag: i32,
}
