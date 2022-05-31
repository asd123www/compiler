use std::collections::HashMap;

use crate::ast::*;
use crate::koopa_ir_gen::{*};
// how to maintain the expression result?
// 1. assign unique id to node for unique variable name.
// 2. attach a `ret` to struct store the result ID.

/*
 * 这里应该非常straight forward没有什么难度.
 * 主要是处理指针, 由于我们scope里面维护了var的类型, 枚举各种情况作对应的操作即可.
 * 
 */
pub trait ExpResult {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size: i32, is_pt: bool) -> ExpRetType;
}


// --------------------------------------- lv3 ------------------------------------------------
// LVal          ::= IDENT {"[" Exp "]"};
impl ExpResult for LVal {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size: i32, _is_pt: bool) -> ExpRetType {
        let mut size = size;
        let mut program = String::from("");

        let var = scope.get(&format!("{}", self.ident)).unwrap();

        if var.0 == CONSTANT_INT { // constant variable.
            return ExpRetType {
                size: size,
                program: program,
                exp_res_id: var.1,
                is_constant: true,
            };
        }

        // %2 = load @x
        let mut pos = var.1; // the position.
        let mut is_first = true;
        for exp in &self.exps {
            let ret_val = exp.eval(scope, size, false); // we want the number.
            size = ret_val.size;
            program.push_str(&ret_val.program); // code for evaluation.

            let name = get_name(ret_val.exp_res_id, ret_val.is_constant);
            if var.0 == VARIABLE_ARRAY {
                if is_first { // `array point` begin with @, but variable begin with `%`.
                    is_first = false;
                    program.push_str(&format!("    %var_{} = getelemptr @var_{}, {}\n", size + 1, pos, name));
                } else {
                    program.push_str(&format!("    %var_{} = getelemptr %var_{}, {}\n", size + 1, pos, name));
                }
            } else {
                if is_first { // a **type !
                    is_first = false;
                    // %0 = load %arr
                    size += 1;
                    program.push_str(&format!("    %var_{} = load @var_{}\n", size, pos));
                    program.push_str(&format!("    %var_{} = getptr %var_{}, {}\n", size + 1, size, name));
                } else {
                    program.push_str(&format!("    %var_{} = getelemptr %var_{}, {}\n", size + 1, pos, name));
                }
            }
            size += 1;
            pos = size;
        }

        return ExpRetType {
            size: size,
            program: program,
            exp_res_id: pos,
            is_constant: false,
        };
    }
}

// Exp ::= LOrExp;
impl ExpResult for Exp {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, is_pt: bool) -> ExpRetType {
        let ret_val = self.lorexp.eval(scope, size, is_pt);
        return ret_val;
    }
}

// PrimaryExp ::= "(" Exp ")" | LVal | Number;
impl ExpResult for PrimaryExp {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, is_pt: bool) -> ExpRetType {
        let mut size = size;
        let mut program = String::from("");
        match self {
            PrimaryExp::Exp(exp) => {
                let ret_val = (*exp).eval(scope, size, is_pt);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    is_constant: ret_val.is_constant,
                };
            },
            // 最基本的情况, 是一个数字.
            PrimaryExp::Num(num) => {
                // constant, we don't need variable.
                return ExpRetType {
                    size: size + 1,
                    program: program,
                    exp_res_id: *num,
                    is_constant: true, // 标记为1.
                };
            }
            PrimaryExp::Lval(lval) => {
                let ret_val = lval.eval(scope, size, is_pt);
                size = ret_val.size;
                if ret_val.is_constant {
                    return ret_val;
                } else {
                    program.push_str(&ret_val.program);
                    if is_pt == false { // if we don't want a pointer.
                        if lval.exps.len() == 0 {
                            program.push_str(&format!("    %var_{} = load @var_{}\n", size + 1, ret_val.exp_res_id));
                        } else {
                            program.push_str(&format!("    %var_{} = load %var_{}\n", size + 1, ret_val.exp_res_id));
                        }
                    } else { // we want a pointer.
                        // special judge....
                        let pair = scope.get(&format!("{}", lval.ident)).unwrap();
                        if pair.0 == VARIABLE_ARRAY {
                            if lval.exps.len() == 0 {
                                program.push_str(&format!("    %var_{} = getelemptr @var_{}, 0\n", size + 1, ret_val.exp_res_id));
                            } else {
                                program.push_str(&format!("    %var_{} = getelemptr %var_{}, 0\n", size + 1, ret_val.exp_res_id));
                            }
                        } else {
                            assert!(pair.0 == PARAMETER_ARRAY);
                            if lval.exps.len() == 0 {
                                size += 1;
                                program.push_str(&format!("    %var_{} = load @var_{}\n", size, ret_val.exp_res_id));
                                program.push_str(&format!("    %var_{} = getptr %var_{}, 0\n", size + 1, size));
                            } else {
                                program.push_str(&format!("    %var_{} = getelemptr %var_{}, 0\n", size + 1, ret_val.exp_res_id));
                            }
                        }
                    }
                }
                return ExpRetType {
                    size: size + 1,
                    program: program,
                    exp_res_id: size + 1,
                    is_constant: false,
                };
            }
        }
    }
}

// UnaryExp ::= PrimaryExp 
//            | UnaryOp UnaryExp
//            | IDENT "(" [FuncRParams] ")"
impl ExpResult for UnaryExp {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, is_pt: bool) -> ExpRetType {
        let mut size = size;
        let mut program = String::from("");
        match self {
            UnaryExp::Primaryexp(primaryexp) => {
                let ret_val = primaryexp.eval(scope, size, is_pt);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    is_constant: ret_val.is_constant,
                };
            },
            UnaryExp::Funcall(ident, params) => {
                // %0 = call @half(10)
                // is it return type `void` or `int`?
                // println!("query: {}_function\n", &ident);
                let is_void = scope.get(&format!("{}_function", &ident)).unwrap();
                let mut bitset = is_void.0 >> TYPE_BITS; // get the function bits.

                let mut params_str = format!("call @{}(", &ident);
                match params {
                    None => {},
                    Some(v) => {
                        let mut is_first = true;
                        for exp in &v.params {
                            let ret_val = exp.eval(scope, size, (bitset & 1) != 0);
                            let name = get_name(ret_val.exp_res_id, ret_val.is_constant);
                            size = ret_val.size;

                            // we should first evaluate all the value, then use it.
                            program.push_str(&ret_val.program);
                            if is_first {
                                params_str.push_str(&format!("{}", &name));
                            } else {
                                params_str.push_str(&format!(", {}", &name));
                            }
                            is_first = false;
                            bitset >>= 1;
                        }
                    },
                }
                params_str.push_str(")\n");

                if (is_void.0 & ((1 << TYPE_BITS) - 1)) == VARIABLE_INT { // it is `int`
                    params_str = format!("    %var_{} = {}", size + 1, params_str);
                    program.push_str(&params_str);
                    return ExpRetType {
                        size: size + 1,
                        program: program,
                        exp_res_id: size + 1,
                        is_constant: false,
                    }
                }
                
                program.push_str(&format!("    {}", &params_str));
                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: -1,
                    is_constant: false,
                }
            },
            UnaryExp::Unaryexp(unaryop, unaryexp) => {
                let ret_val = unaryexp.eval(scope, size, is_pt);

                match unaryop {
                    UnaryOp::Add => {
                        return ExpRetType {
                            size: ret_val.size + 1,
                            program: ret_val.program,
                            exp_res_id: ret_val.exp_res_id,
                            is_constant: ret_val.is_constant,
                        };
                    },
                    UnaryOp::Sub => {
                        let size = ret_val.size + 1;
                        let mut val = -ret_val.exp_res_id;
                        if !ret_val.is_constant { // constant don't need it.
                            program.push_str(&ret_val.program);
                            program.push_str(&format!("    %var_{} = sub 0, %var_{}\n", size, ret_val.exp_res_id));
                            val = size;
                        }

                        return ExpRetType {
                            size: size,
                            program: program,
                            exp_res_id: val,
                            is_constant: ret_val.is_constant,
                        };
                    },
                    UnaryOp::Not => {
                        let size = ret_val.size + 1;
                        let mut val = (ret_val.exp_res_id == 0) as i32;
                        if !ret_val.is_constant { // constant don't need it.
                            program.push_str(&ret_val.program);
                            program.push_str(&format!("    %var_{} = eq 0, %var_{}\n", size, ret_val.exp_res_id));
                            val = size;
                        }

                        return ExpRetType {
                            size: size,
                            program: program,
                            exp_res_id: val,
                            is_constant: ret_val.is_constant,
                        };
                    },
                }
            },
        }
    }
}


fn binary_operation(program: &mut String, size: &mut i32, op: &str, val1: &ExpRetType, val2: &ExpRetType) -> i32 {

    // 太尼玛丑了.
    let mut val = {
        if op == "mul" {
            val1.exp_res_id * val2.exp_res_id
        } else if op == "div" {
            val1.exp_res_id / val2.exp_res_id
        } else if op == "mod" {
            val1.exp_res_id % val2.exp_res_id
        } else if op == "add" {
            val1.exp_res_id + val2.exp_res_id
        } else if op == "sub" {
            val1.exp_res_id - val2.exp_res_id
        } else if op == "eq" {
            (val1.exp_res_id == val2.exp_res_id) as i32
        } else if op == "ne" {
            (val1.exp_res_id != val2.exp_res_id) as i32
        } else if op == "lt" {
            (val1.exp_res_id < val2.exp_res_id) as i32
        } else if op == "gt" {
            (val1.exp_res_id > val2.exp_res_id) as i32
        } else if op == "le" {
            (val1.exp_res_id <= val2.exp_res_id) as i32
        } else if op == "ge" {
            (val1.exp_res_id >= val2.exp_res_id) as i32
        } else {
            0
        }
    };

    if !val1.is_constant || !val2.is_constant {
        let name1 = get_name(val1.exp_res_id, val1.is_constant);
        let name2 = get_name(val2.exp_res_id, val2.is_constant);

        // if any value is constant, then there shouldn't be any code.
        if !val1.is_constant {
            program.push_str(&val1.program);
        }
        if !val2.is_constant {
            program.push_str(&val2.program);
        }
        program.push_str(&format!("    %var_{} = {} {}, {}\n", size, op, name1, name2));
        val = *size; // it is not a constant, so return variable id.
    }

    val
}

// MulExp ::= UnaryExp | MulExp ("*" | "/" | "%") UnaryExp;
impl ExpResult for MulExp {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, is_pt: bool) -> ExpRetType {
        match self {
            MulExp::Unaryexp(unaryexp) => {
                let ret_val = unaryexp.eval(scope, size, is_pt); 

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    is_constant: ret_val.is_constant,
                };
            },
            MulExp::Mulexp(mulexp, unaryexp, op) |
            MulExp::Divexp(mulexp, unaryexp, op) | 
            MulExp::Modexp(mulexp, unaryexp, op) => {
                let ret_val1 = (*mulexp).eval(scope, size, is_pt);
                let ret_val2 = unaryexp.eval(scope, ret_val1.size, is_pt);
                let mut size = ret_val2.size + 1;
                let mut program = String::from("");

                let val = binary_operation(&mut program, &mut size, op, &ret_val1, &ret_val2);

                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: val,
                    is_constant: ret_val1.is_constant && ret_val2.is_constant,
                };
            },
        }
    }
}

// AddExp ::= MulExp | AddExp ("+" | "-") MulExp;
impl ExpResult for AddExp {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, is_pt: bool) -> ExpRetType {
        match self {
            AddExp::Mulexp(mulexp) => {
                let ret_val = mulexp.eval(scope, size, is_pt);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    is_constant: ret_val.is_constant,
                };
            },
            AddExp::Addexp(addexp, mulexp, op) |
            AddExp::Subexp(addexp, mulexp, op) => {
                let ret_val1 = (*addexp).eval(scope, size, is_pt);
                let ret_val2 = (*mulexp).eval(scope, ret_val1.size, is_pt);
                let mut size = ret_val2.size + 1;
                let mut program = String::from("");

                let val = binary_operation(&mut program, &mut size, op, &ret_val1, &ret_val2);

                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: val,
                    is_constant: ret_val1.is_constant && ret_val2.is_constant,
                };
            },
        }
    }
}

// RelExp ::= AddExp | RelExp ("<" | ">" | "<=" | ">=") AddExp;
impl ExpResult for RelExp {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, is_pt: bool) -> ExpRetType {
        match self {
            RelExp::Addexp(addexp) => {
                let ret_val = addexp.eval(scope, size, is_pt); 

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    is_constant: ret_val.is_constant,
                };
            },
            RelExp::Ltexp(relexp, addexp, op) |
            RelExp::Gtexp(relexp, addexp, op) |
            RelExp::Geexp(relexp, addexp, op) | 
            RelExp::Leexp(relexp, addexp, op) => {
                let ret_val1 = (*relexp).eval(scope, size, is_pt);
                let ret_val2 = addexp.eval(scope, ret_val1.size, is_pt); 
                let mut size = ret_val2.size + 1;
                let mut program = String::from("");

                let val = binary_operation(&mut program, &mut size, op, &ret_val1, &ret_val2);

                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: val,
                    is_constant: ret_val1.is_constant && ret_val2.is_constant,
                };
            }
        }
    }
}

// EqExp ::= RelExp | EqExp ("==" | "!=") RelExp;
impl ExpResult for EqExp {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, is_pt: bool) -> ExpRetType {
        match self {
            EqExp::Relexp(relexp) => {
                let ret_val = relexp.eval(scope, size, is_pt); 

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    is_constant: ret_val.is_constant,
                };
            },
            EqExp::Eqexp(eqexp, relexp, op) |
            EqExp::Neqexp(eqexp, relexp, op) => {
                let ret_val1 = (*eqexp).eval(scope, size, is_pt);
                let ret_val2 = relexp.eval(scope, ret_val1.size + 1, is_pt);
                let mut size = ret_val2.size + 1;
                let mut program = String::from("");

                let val = binary_operation(&mut program, &mut size, op, &ret_val1, &ret_val2);

                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: val,
                    is_constant: ret_val1.is_constant && ret_val2.is_constant,
                };
            },
        }
    }
}

// we need to add short-circuit evaluation.
// LAndExp       ::= EqExp | LAndExp "&&" EqExp;
impl ExpResult for LAndExp {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, is_pt: bool) -> ExpRetType {
        match self {
            LAndExp::Eqexp(eqexp) => {
                let ret_val = eqexp.eval(scope, size, is_pt);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    is_constant: ret_val.is_constant,
                };
            },
            LAndExp::Andexp(landexp, eqexp) => {
                let mut program = String::from("");
                let ret_val1 = (*landexp).eval(scope, size, is_pt);
                let ret_val2 = eqexp.eval(scope, ret_val1.size, is_pt);
                let name1 = get_name(ret_val1.exp_res_id, ret_val1.is_constant);
                let name2 = get_name(ret_val2.exp_res_id, ret_val2.is_constant);

                if ret_val1.is_constant && ret_val2.is_constant { // if it's constant.
                    return ExpRetType {
                        size: ret_val2.size,
                        program: program,
                        exp_res_id: ((ret_val1.exp_res_id != 0) && (ret_val2.exp_res_id != 0)) as i32,
                        is_constant: true,
                    };
                }

                let size = ret_val2.size + 1;

                // init condition to zero.
                program.push_str(&format!("    @condition_{} = alloc i32\n", size));
                program.push_str(&format!("    store 0, @condition_{}\n", size));

                // first evaluate the left.
                program.push_str(&ret_val1.program); // jump according to ret_val.
                program.push_str(&format!("    br {}, %entry_{}, %entry_{}\n", &name1, size, size + 2));
                
                // if first is true, then jump to here.
                program.push_str(&format!("\n%entry_{}:\n", size));
                // then evaluate the left.
                program.push_str(&ret_val2.program); // jump according to ret_val.
                program.push_str(&format!("    br {}, %entry_{}, %entry_{}\n", &name2, size + 1, size + 2));

                // both is true.
                program.push_str(&format!("\n%entry_{}:\n", size + 1));
                program.push_str(&format!("    store 1, @condition_{}\n", size));
                program.push_str(&format!("    jump %entry_{}\n", size + 2));

                program.push_str(&format!("\n%entry_{}:\n", size + 2));
                program.push_str(&format!("    %var_{} = load @condition_{}\n", size + 2, size));

                return ExpRetType {
                    size: size + 2,
                    program: program,
                    exp_res_id: size + 2,
                    is_constant: ret_val1.is_constant && ret_val2.is_constant,
                };
            },
        }
    }
}

// LOrExp ::= LAndExp | LOrExp "||" LAndExp;
impl ExpResult for LOrExp {
    fn eval(&self, scope: &HashMap<String, (i32, i32)>, size:i32, is_pt: bool) -> ExpRetType {
        match self {
            LOrExp::Landexp(landexp) => {
                let ret_val = landexp.eval(scope, size, is_pt);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    is_constant: ret_val.is_constant,
                };
            },
            LOrExp::Orexp(lorexp, landexp) => {
                let mut program = String::from("");
                let ret_val1 = (*lorexp).eval(scope, size, is_pt);
                let ret_val2 = landexp.eval(scope, ret_val1.size, is_pt);
                if ret_val1.is_constant && ret_val2.is_constant { // if it's constant.
                    return ExpRetType {
                        size: ret_val2.size,
                        program: program,
                        exp_res_id: (ret_val1.exp_res_id != 0 || ret_val2.exp_res_id != 0) as i32,
                        is_constant: true,
                    };
                }

                let name1 = get_name(ret_val1.exp_res_id, ret_val1.is_constant);
                let name2 = get_name(ret_val2.exp_res_id, ret_val2.is_constant);

                let size = ret_val2.size + 1;


                // init condition to zero.
                program.push_str(&format!("    @condition_{} = alloc i32\n", size));
                program.push_str(&format!("    store 0, @condition_{}\n", size));

                // first evaluate the left.
                program.push_str(&ret_val1.program); // jump according to ret_val.
                program.push_str(&format!("    br {}, %entry_{}, %entry_{}\n", &name1, size + 1, size));
                
                // if first is false, then jump to here.
                program.push_str(&format!("\n%entry_{}:\n", size));
                // then evaluate the left.
                program.push_str(&ret_val2.program); // jump according to ret_val.
                program.push_str(&format!("    br {}, %entry_{}, %entry_{}\n", &name2, size + 1, size + 2));
                
                // exist one condition is true.
                program.push_str(&format!("\n%entry_{}:\n", size + 1));
                program.push_str(&format!("    store 1, @condition_{}\n", size));
                program.push_str(&format!("    jump %entry_{}\n", size + 2));

                program.push_str(&format!("\n%entry_{}:\n", size + 2));
                program.push_str(&format!("    %var_{} = load @condition_{}\n", size + 2, size));

                return ExpRetType {
                    size: size + 2,
                    program: program,
                    exp_res_id: size + 2,
                    is_constant: ret_val1.is_constant && ret_val2.is_constant,
                };
            },
        }
    }
}