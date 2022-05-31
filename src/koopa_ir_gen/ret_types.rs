pub const VOID: i32 = 0;
pub const CONSTANT_INT: i32 = 1;
pub const VARIABLE_INT: i32 = 2;
pub const VARIABLE_ARRAY: i32 = 3;
pub const PARAMETER_ARRAY: i32 = 4;

pub const TYPE_BITS: i32 = 3;

/* 
 *定义了不同模块的返回类型, 比如
 */


/*
 * ExpRetType中定义了对于expression的求值, 
 * 同时为了解决常量求值的问题, 增加is_constant field代表表达式的求值过程是否为constant.
 * 1. is_constant is true, exp_res_id 就是这个常量值, 2. is false, var{xp_res_id}存储了这个变量.
 * 
 * (size, program) 是必须有的系统常量, program就是代码的string表达.
 */
// exp_res_id:
//     if is_constant == true: store constant value.
//     else                    store variable ID.
pub struct ExpRetType {
    pub size: i32,
    pub program: String,
    pub exp_res_id: i32,
    pub is_constant: bool,
}

/* 
 * 这里主要对应于 `koopa_ir_gen/declare.rs` 中定义声明变量.
 * flag 没用删掉.
 */
pub struct DeclRetType {
    pub size: i32,
    pub program: String,
}


/*
 * BodyRetType对应于 `koopa_ir_gen/mod.rs` 中程序主要body.
 * exp_res_id在大部分情况下没用, 但是当body是定义函数时, 我们需要区别array参数和integer参数.
 * 因此这里最多支持32个参数, 但是瓶颈是全局decl索引的HashMap需要3位标记decl类型, 但是这些都很容易扩展.
 */
pub struct BodyRetType {
    pub size: i32,
    pub program: String,
    pub exp_res_id: i32,
}


/* 
 * 这里主要对应于 `koopa_ir_gen/initialvalue.rs` 中的对于初始值的求值.
 * is_allzero对应了0初始化, if is_allzero is true, 我们就不需要val vector了.
 * 否则我们需要val vector中的每个值顺序对应初始化中的每个域. 
 * val vector中包含了我们对于数组初始化 transfer后的结果, 还算是一个non-trivial的功能.
 */
pub struct InitRetType {
    pub size: i32,
    pub program: String,
    pub is_allzero: bool,
    pub val: Vec<(bool, i32)>,
}