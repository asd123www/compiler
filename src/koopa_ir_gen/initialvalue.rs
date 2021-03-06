use std::{collections::HashMap};

use crate::ast::*;
use super::{ret_types::InitRetType, expression::ExpResult};

/* 
 * 由于数组的存在, 我们单独把InitValue求值拿出来. 
 * decl var = InitValue, 我们的任务就是把InitValue的值求出来, 返回给decl语句进行初始化.
 */

pub trait InitValue {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, dims: &[i32]) -> InitRetType;
}

// only a single fixed value.
// ConstExp ::= Exp
impl ConstExp {
    pub fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32) -> InitRetType {
        let ret_val = self.exp.eval(scope, size, false);

        assert!(ret_val.is_constant);

        return InitRetType {
            size: ret_val.size,
            program: "".to_string(), // no code is needed.
            is_allzero: false,
            val: vec![(ret_val.is_constant, ret_val.exp_res_id)],
        };
    }
}

fn zero_padding(val: &mut Vec<(bool, i32)>, dims: &[i32]) {
    let len = {
        let mut x:u32 = 1;
        for t in dims { x = x * (*t as u32);}
        x
    };
    for _i in 0..len - (val.len() as u32) {
        val.push((true, 0));
    }
}

// ConstInitVal ::= ConstExp | "{" [ConstInitVal {"," ConstInitVal}] "}"
impl InitValue for ConstInitVal {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, dims: &[i32]) -> InitRetType {

        match self {
            ConstInitVal::SingleExp(exp) => { // must have one element.
                let ret_val = exp.eval(scope, size);
                return ret_val;
            },
            ConstInitVal::ZeroInit() => { // we can contain no element.
                return InitRetType {
                    size: size,
                    program: "".to_string(),
                    is_allzero: true,
                    val: Vec::new(), // should here be filled with zero for special case?
                }
            },
            ConstInitVal::MultiExp(const_vals) => {
                let mut size = size;
                let mut val = Vec::<(bool, i32)>::new();

                for ele in const_vals {
                    let mut ret_val;
                    match ele {
                        ConstInitVal::SingleExp(_) => { // if it is a single, then multiple of last dimension.
                            ret_val = ele.eval(scope, size, &[0]);
                        },
                        _ => {
                            while val.len() % (dims[dims.len() - 1] as usize) != 0 {
                                val.push((true, 0));
                            }
                            let pos = {
                                if val.len() == 0 {
                                    1// dims.len() - 1
                                } else {
                                    let mut pd = 1;
                                    let mut pos = dims.len();
                                    while val.len() % pd == 0 {
                                        // println!("gogogo  {} {}", pd, pos);
                                        pos -= 1;
                                        pd = pd * (dims[pos] as usize); // dims must be positive.
                                    }
                                    pos + 1
                                }
                            }; assert!(pos != dims.len()); // must be multiple of last dimension.

                            ret_val = ele.eval(scope, size, &dims[pos..dims.len()]);
                        }
                    }
                    val.append(&mut ret_val.val);
                    size = ret_val.size;
                    // we don't have to care about the program cause it's constant.
                }
                zero_padding(&mut val, dims);

                return InitRetType {
                    size: size,
                    program: "".to_string(), // no code is needed.
                    is_allzero: false,
                    val: val,
                };
            },
        }
    }
}

// InitVal ::= Exp | "{" [InitVal {"," InitVal}] "}"
impl InitValue for InitVal {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, dims: &[i32]) -> InitRetType {
        match self {
            InitVal::SingleExp(exp) => { // must have one element.
                let ret_val = exp.eval(scope, size, false);
                let mut val = Vec::<(bool, i32)>::new();

                val.push((ret_val.is_constant, ret_val.exp_res_id));

                return InitRetType {
                    size: ret_val.size,
                    program: ret_val.program,
                    is_allzero: false,
                    val: val, // should here be filled with zero for special case?
                };
            },
            InitVal::ZeroInit() => { // we can contain no element.
                return InitRetType {
                    size: size,
                    program: "".to_string(),
                    is_allzero: true,
                    val: Vec::new(), // should here be filled with zero for special case?
                }
            },
            InitVal::MultiExp(const_vals) => {
                let mut size = size;
                let mut program = "".to_string();
                let mut val = Vec::<(bool, i32)>::new();

                for ele in const_vals {
                    let mut ret_val;
                    match ele {
                        InitVal::SingleExp(_) => { // if it is a single, then multiple of last dimension.
                            ret_val = ele.eval(scope, size, &dims[dims.len()-1..dims.len()]);
                        },
                        _ => {
                            while val.len() % (dims[dims.len() - 1] as usize) != 0 {
                                val.push((true, 0));
                            }
                            let pos = {
                                if val.len() == 0 {
                                    1// dims.len() - 1
                                } else {
                                    let mut pd = 1;
                                    let mut pos = dims.len();
                                    while val.len() % pd == 0 {
                                        pos -= 1;
                                        pd = pd * (dims[pos] as usize); // dims must be positive.
                                    }
                                    pos + 1
                                }
                            }; assert!(pos != dims.len()); // must be multiple of last dimension.

                            ret_val = ele.eval(scope, size, &dims[pos..dims.len()]);
                        }
                    }
                    val.append(&mut ret_val.val);
                    size = ret_val.size;
                    program.push_str(&ret_val.program);
                }
                zero_padding(&mut val, dims);

                return InitRetType {
                    size: size,
                    program: program, // no code is needed.
                    is_allzero: false,
                    val: val,
                };
            },
        }
    }
}