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
                };
            },
            PrimaryExp::Num(num) => {
                program.push_str(&format!("    %var_{} = add 0, {}\n", size + 1, num));

                return ExpRetType {
                    size: size + 1,
                    program: program,
                    exp_res_id: size + 1,
                };
            }
            PrimaryExp::Lval(lval) => {
                let var = scope.get(&format!("{}", lval.ident)).unwrap();

                if (var & 1) == 1 {
                    return ExpRetType {
                        size: size,
                        program: program,
                        exp_res_id: var >> 1,
                    };
                }

                // %2 = load @x
                program.push_str(&format!("    %var_{} = load @var_{}\n", size + 1, var >> 1));
                return ExpRetType {
                    size: size + 1,
                    program: program,
                    exp_res_id: size + 1,
                };
            }
        }
    }
}

// UnaryExp ::= PrimaryExp | UnaryOp UnaryExp;
impl ExpResult for UnaryExp {
    fn eval(&self, scope: &HashMap<String, i32>, size:i32) -> ExpRetType {
        match self {
            UnaryExp::Primaryexp(primaryexp) => {
                let ret_val = primaryexp.eval(scope, size);

                return ExpRetType {
                    size: ret_val.size + 1,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id,
                };
            },
            UnaryExp::Unaryexp(unaryop, unaryexp) => {
                let ret_val = unaryexp.eval(scope, size);
                let mut program = String::from("");

                match unaryop {
                    UnaryOp::Add => {
                        return ExpRetType {
                            size: ret_val.size + 1,
                            program: ret_val.program,
                            exp_res_id: ret_val.exp_res_id,
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
                        };
                    },
                }
            }
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
                };
            },
        }
    }
}