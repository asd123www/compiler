use std::collections::HashMap;

use crate::ast::*;
use crate::koopa_ir_gen::ExpRetType;


// how to maintain the expression result?
// 1. assign unique id to node for unique variable name.
// 2. attach a `ret` to struct store the result ID.

pub trait ExpResult {
    fn eval(&self, scope: &HashMap<String, i32>, size: i32) -> ExpRetType;
}


// --------------------------------------- lv4 ------------------------------------------------

// ConstInitVal ::= ConstExp;
impl ExpResult for ConstInitVal {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        let ret_val = self.constexp.eval(scope, size);
        return ret_val;
    }
}

// ConstExp ::= Exp;
impl ExpResult for ConstExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        let ret_val = self.exp.eval(scope, size);
        return ret_val;
    }
}

// InitVal ::= Exp;
impl ExpResult for InitVal {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        let ret_val = self.exp.eval(scope, size);
        return ret_val;
    }
}




// --------------------------------------- lv3 ------------------------------------------------

// Exp ::= LOrExp;
impl ExpResult for Exp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        let ret_val = self.lorexp.eval(scope, size);
        return ret_val;
    }
}

// PrimaryExp ::= "(" Exp ")" | LVal | Number;
impl ExpResult for PrimaryExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        let mut program = String::from("");
        match self {
            PrimaryExp::Exp(exp) => {
                let ret_val = (*exp).eval(scope, size);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    const_value: ret_val.const_value,
                };
            },
            PrimaryExp::Num(num) => {
                program.push_str(&format!("    %var_{} = add 0, {}\n", size + 1, num));

                return ExpRetType {
                    size: size + 1,
                    program: program,
                    exp_res_id: size + 1,
                    const_value: *num,
                };
            }
            PrimaryExp::Lval(lval) => {
                let var = scope.get(&format!("{}", lval.ident)).unwrap();

                if (var & 1) == 1 { // constant variable.
                    return ExpRetType {
                        size: size,
                        program: program,
                        exp_res_id: var >> 1,
                        const_value: var >> 1,
                    };
                }

                // %2 = load @x
                program.push_str(&format!("    %var_{} = load @var_{}\n", size + 1, var >> 1));
                return ExpRetType {
                    size: size + 1,
                    program: program,
                    exp_res_id: size + 1,
                    const_value: -1,
                };
            }
        }
    }
}

// UnaryExp ::= PrimaryExp 
//            | UnaryOp UnaryExp
//            | IDENT "(" [FuncRParams] ")"
impl ExpResult for UnaryExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        let mut size = size;
        let mut program = String::from("");
        match self {
            UnaryExp::Primaryexp(primaryexp) => {
                let ret_val = primaryexp.eval(scope, size);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    const_value: ret_val.const_value,
                };
            },
            UnaryExp::Funcall(ident, params) => {
                // %0 = call @half(10)
                let mut params_str = format!("call @{}(", &ident);
                match params {
                    None => {},
                    Some(v) => {
                        let mut is_first = true;
                        for exp in &v.params {
                            let ret_val = exp.eval(scope, size);
                            size = ret_val.size;

                            // we should first evaluate all the value, then use it.
                            program.push_str(&ret_val.program);
                            if is_first {
                                params_str.push_str(&format!("%var_{}", ret_val.exp_res_id));
                            } else {
                                params_str.push_str(&format!(", %var_{}", ret_val.exp_res_id));
                            }
                            is_first = false;
                        }
                    },
                }
                params_str.push_str(")\n");

                // is it return type `void` or `int`?
                println!("query: {}_function\n", &ident);
                let is_void = scope.get(&format!("{}_function", &ident)).unwrap();
                if *is_void == 0 { // it is `int`
                    params_str = format!("    %var_{} = {}", size + 1, params_str);
                    program.push_str(&params_str);
                    return ExpRetType {
                        size: size + 1,
                        program: program,
                        exp_res_id: size + 1,
                        const_value: -1,
                    }
                }
                
                program.push_str(&format!("    {}", &params_str));
                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: -1,
                    const_value: -1,
                }
            },
            UnaryExp::Unaryexp(unaryop, unaryexp) => {
                let ret_val = unaryexp.eval(scope, size);

                match unaryop {
                    UnaryOp::Add => {
                        return ExpRetType {
                            size: ret_val.size + 1,
                            program: ret_val.program,
                            exp_res_id: ret_val.exp_res_id,
                            const_value: ret_val.const_value,
                        };
                    },
                    UnaryOp::Sub => {
                        let size = ret_val.size + 1;
                        program.push_str(&ret_val.program);
                        program.push_str(&format!("    %var_{} = sub 0, %var_{}\n", size, ret_val.exp_res_id));

                        return ExpRetType {
                            size: size,
                            program: program,
                            exp_res_id: size,
                            const_value: -ret_val.const_value,
                        };
                    },
                    UnaryOp::Not => {
                        let size = ret_val.size + 1;
                        program.push_str(&ret_val.program);
                        program.push_str(&format!("    %var_{} = eq 0, %var_{}\n", size, ret_val.exp_res_id));

                        return ExpRetType {
                            size: size,
                            program: program,
                            exp_res_id: size,
                            const_value: !ret_val.const_value,
                        };
                    },
                }
            },
        }
    }
}


// MulExp ::= UnaryExp | MulExp ("*" | "/" | "%") UnaryExp;
impl ExpResult for MulExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        match self {
            MulExp::Unaryexp(unaryexp) => {
                let ret_val = unaryexp.eval(scope, size); 

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    const_value: ret_val.const_value,
                };
            },
            MulExp::Mulexp(mulexp, unaryexp, op) |
            MulExp::Divexp(mulexp, unaryexp, op) | 
            MulExp::Modexp(mulexp, unaryexp, op) => {
                let ret_val1 = (*mulexp).eval(scope, size);
                let ret_val2 = unaryexp.eval(scope, ret_val1.size);
                let size = ret_val2.size + 1;
                let mut program = String::from("");

                program.push_str(&ret_val1.program);
                program.push_str(&ret_val2.program);
                program.push_str(&format!("    %var_{} = {} %var_{}, %var_{}\n", size, op, ret_val1.exp_res_id, ret_val2.exp_res_id));
    
                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: size,
                    const_value: {
                        if op == "mul" {
                            ret_val1.const_value * ret_val2.const_value
                        } else if op == "div" {
                            ret_val1.const_value / ret_val2.const_value
                        } else { // "mod"
                            ret_val1.const_value % ret_val2.const_value
                        }
                    }
                };
            },
        }
    }
}

// AddExp ::= MulExp | AddExp ("+" | "-") MulExp;
impl ExpResult for AddExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        match self {
            AddExp::Mulexp(mulexp) => {
                let ret_val = mulexp.eval(scope, size);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    const_value: ret_val.const_value,
                };
            },
            AddExp::Addexp(addexp, mulexp, op) |
            AddExp::Subexp(addexp, mulexp, op) => {
                let ret_val1 = (*addexp).eval(scope, size);
                let ret_val2 = (*mulexp).eval(scope, ret_val1.size);
                let size = ret_val2.size + 1;
                let mut program = String::from("");

                program.push_str(&ret_val1.program);
                program.push_str(&ret_val2.program);
                program.push_str(&format!("    %var_{} = {} %var_{}, %var_{}\n", size, op, ret_val1.exp_res_id, ret_val2.exp_res_id));
    
                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: size,
                    const_value: {
                        if op == "add" {
                            ret_val1.const_value + ret_val2.const_value
                        } else { // "sub"
                            ret_val1.const_value - ret_val2.const_value
                        }
                    }
                };
            },
        }
    }
}

// RelExp ::= AddExp | RelExp ("<" | ">" | "<=" | ">=") AddExp;
impl ExpResult for RelExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        match self {
            RelExp::Addexp(addexp) => {
                let ret_val = addexp.eval(scope, size); 

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    const_value: ret_val.const_value,
                };
            },
            RelExp::Ltexp(relexp, addexp, op) |
            RelExp::Gtexp(relexp, addexp, op) |
            RelExp::Geexp(relexp, addexp, op) | 
            RelExp::Leexp(relexp, addexp, op) => {
                let ret_val1 = (*relexp).eval(scope, size);
                let ret_val2 = addexp.eval(scope, ret_val1.size); 
                let size = ret_val2.size + 1;
                let mut program = String::from("");

                program.push_str(&ret_val1.program);
                program.push_str(&ret_val2.program);
                program.push_str(&format!("    %var_{} = {} %var_{}, %var_{}\n", size, op, ret_val1.exp_res_id, ret_val2.exp_res_id));
    
                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: size,
                    const_value: {
                        if op == "lt" {
                            (ret_val1.const_value < ret_val2.const_value) as i32
                        } else if op == "gt" {
                            (ret_val1.const_value > ret_val2.const_value) as i32
                        } else if op == "le" {
                            (ret_val1.const_value <= ret_val2.const_value) as i32
                        } else { // "ge"
                            (ret_val1.const_value >= ret_val2.const_value) as i32
                        }
                    },
                };
            }
        }
    }
}

// EqExp ::= RelExp | EqExp ("==" | "!=") RelExp;
impl ExpResult for EqExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        match self {
            EqExp::Relexp(relexp) => {
                let ret_val = relexp.eval(scope, size); 

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    const_value: ret_val.const_value,
                };
            },
            EqExp::Eqexp(eqexp, relexp, op) |
            EqExp::Neqexp(eqexp, relexp, op) => {
                let ret_val1 = (*eqexp).eval(scope, size);
                let ret_val2 = relexp.eval(scope, ret_val1.size + 1);
                let size = ret_val2.size + 1;
                let mut program = String::from("");

                program.push_str(&ret_val1.program);
                program.push_str(&ret_val2.program);
                program.push_str(&format!("    %var_{} = {} %var_{}, %var_{}\n", size, op, ret_val1.exp_res_id, ret_val2.exp_res_id));
    
                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: size,
                    const_value: {
                        if op == "eq" {
                            (ret_val1.const_value == ret_val2.const_value) as i32
                        } else { // "ne"
                            (ret_val1.const_value != ret_val2.const_value) as i32
                        }
                    }
                };
            },
        }
    }
}

// we need to add short-circuit evaluation.
// LAndExp       ::= EqExp | LAndExp "&&" EqExp;
impl ExpResult for LAndExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        match self {
            LAndExp::Eqexp(eqexp) => {
                let ret_val = eqexp.eval(scope, size);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    const_value: ret_val.const_value,
                };
            },
            LAndExp::Andexp(landexp, eqexp) => {
                let ret_val1 = (*landexp).eval(scope, size);
                let ret_val2 = eqexp.eval(scope, ret_val1.size);
                let size = ret_val2.size + 1;
                let mut program = String::from("");

                // init condition to zero.
                program.push_str(&format!("    @condition_{} = alloc i32\n", size));
                program.push_str(&format!("    store 0, @condition_{}\n", size));

                // first evaluate the left.
                program.push_str(&ret_val1.program); // jump according to ret_val.
                program.push_str(&format!("    br %var_{}, %entry_{}, %entry_{}\n", ret_val1.exp_res_id, size, size + 2));
                
                // if first is true, then jump to here.
                program.push_str(&format!("\n%entry_{}:\n", size));
                // then evaluate the left.
                program.push_str(&ret_val2.program); // jump according to ret_val.
                program.push_str(&format!("    br %var_{}, %entry_{}, %entry_{}\n", ret_val2.exp_res_id, size + 1, size + 2));
                
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
                    const_value: { // do we really need it?
                        (ret_val1.const_value != 0 && ret_val2.const_value != 0) as i32
                    }
                };
            },
        }
    }
}

// LOrExp ::= LAndExp | LOrExp "||" LAndExp;
impl ExpResult for LOrExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        match self {
            LOrExp::Landexp(landexp) => {
                let ret_val = landexp.eval(scope, size);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                    const_value: ret_val.const_value,
                };
            },
            LOrExp::Orexp(lorexp, landexp) => {
                let ret_val1 = (*lorexp).eval(scope, size);
                let ret_val2 = landexp.eval(scope, ret_val1.size);
                let size = ret_val2.size + 1;
                let mut program = String::from("");


                // init condition to zero.
                program.push_str(&format!("    @condition_{} = alloc i32\n", size));
                program.push_str(&format!("    store 0, @condition_{}\n", size));

                // first evaluate the left.
                program.push_str(&ret_val1.program); // jump according to ret_val.
                program.push_str(&format!("    br %var_{}, %entry_{}, %entry_{}\n", ret_val1.exp_res_id, size + 1, size));
                
                // if first is false, then jump to here.
                program.push_str(&format!("\n%entry_{}:\n", size));
                // then evaluate the left.
                program.push_str(&ret_val2.program); // jump according to ret_val.
                program.push_str(&format!("    br %var_{}, %entry_{}, %entry_{}\n", ret_val2.exp_res_id, size + 1, size + 2));
                
                // exist one condition is true.
                program.push_str(&format!("\n%entry_{}:\n", size + 1));
                program.push_str(&format!("    store 1, @condition_{}\n", size));
                program.push_str(&format!("    jump %entry_{}\n", size + 2));

                program.push_str(&format!("\n%entry_{}:\n", size + 2));
                program.push_str(&format!("    %var_{} = load @condition_{}\n", size + 2, size));

                // program.push_str(&ret_val1.program);
                // program.push_str(&ret_val2.program);
    
                // program.push_str(&format!("    %var_{} = ne 0, %var_{}\n", size, ret_val1.exp_res_id));
                // program.push_str(&format!("    %var_{} = ne 0, %var_{}\n", size + 1, ret_val2.exp_res_id));
                // program.push_str(&format!("    %var_{} = or %var_{}, %var_{}\n", size + 2, size, size + 1));

                return ExpRetType {
                    size: size + 2,
                    program: program,
                    exp_res_id: size + 2,
                    const_value: {
                        (ret_val1.const_value != 0 || ret_val2.const_value != 0) as i32
                    }
                };
            },
        }
    }
}