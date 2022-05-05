

use std::{collections::HashMap};

use crate::koopa_ir_gen::get_name;
use crate::{koopa_ir_gen::expression::ExpResult, ast::{*}};

use super::{TreePoint, ret_types::ExpRetType, dfs};


// statement: open_statement
//          | closed_statement
//          | "while" "(" Exp ")" Stmt
impl Statement {
    pub fn eval(&self, scope: &HashMap<String, (i32, i32)>, size: i32) -> ExpRetType {
        match self {
            Statement::Open(os) => {
                os.eval(scope, size)
            },
            Statement::Closed(cs) => {
                cs.eval(scope, size)
            },
        }
    }
}


// open_statement: IF '(' expression ')' statement
//               | IF '(' expression ')' closed_statement ELSE open_statement
impl OpenStatement {
    pub fn eval(&self, scope: &HashMap<String, (i32, i32)>, size: i32) -> ExpRetType {
        let size = size;
        let mut program = "".to_string();
        match self {
            OpenStatement::If(exp, stmt) => {
                let exp_val = exp.eval(scope, size, false);
                let stmt_val = stmt.eval(scope, exp_val.size);
                let size = stmt_val.size + 1;
                let name = get_name(exp_val.exp_res_id, exp_val.is_constant);

                // evaluate the condition.
                program.push_str(&exp_val.program);
                program.push_str(&format!("    br {}, %entry_{}, %entry_{}\n", &name, size, size + 1));

                program.push_str(&format!("\n%entry_{}:\n", size)); // body of statement.
                program.push_str(&stmt_val.program);
                program.push_str(&format!("    jump %entry_{}\n", size + 1));

                program.push_str(&format!("\n%entry_{}:\n", size + 1)); // after.

                return ExpRetType{size: size + 1, program, exp_res_id: 0, is_constant: false,};
            },
            OpenStatement::Ifelse(exp, cs, os) => {
                let exp_val = exp.eval(scope, size, false);
                let cs_val = cs.eval(scope, exp_val.size);
                let os_val = os.eval(scope, cs_val.size);
                let size = os_val.size + 1;
                let name = get_name(exp_val.exp_res_id, exp_val.is_constant);

                // evaluate the condition.
                program.push_str(&exp_val.program);
                program.push_str(&format!("    br {}, %entry_{}, %entry_{}\n", &name, size, size + 1));

                // first part.
                program.push_str(&format!("\n%entry_{}:\n", size));
                program.push_str(&cs_val.program);
                program.push_str(&format!("    jump %entry_{}\n", size + 2));
                
                // second part.
                program.push_str(&format!("\n%entry_{}:\n", size + 1));
                program.push_str(&os_val.program);
                program.push_str(&format!("    jump %entry_{}\n", size + 2));
                
                // after.
                program.push_str(&format!("\n%entry_{}:\n", size + 2));

                return ExpRetType{size: size + 2, program, exp_res_id: 0, is_constant: false,};
            },
            OpenStatement::While(exp, stmt) => {
                let exp_val = exp.eval(scope, size, false);
                let stmt_val = stmt.eval(scope, exp_val.size);
                let size = stmt_val.size + 1;
                let name = get_name(exp_val.exp_res_id, exp_val.is_constant);

                // replace `break` and `continue`.
                let real_body = str::replace(&stmt_val.program, "<replace_me_with_break>", &format!("jump %entry_{}", size + 2));
                let real_body = str::replace(&real_body, "<replace_me_with_continue>", &format!("jump %entry_{}", size));

                
                // end last block and jump to condition.
                program.push_str(&format!("    jump %entry_{}\n", size));

                // condition label.
                program.push_str(&format!("\n%entry_{}:\n", size));
                program.push_str(&exp_val.program); // conditional jump.
                program.push_str(&format!("    br {}, %entry_{}, %entry_{}\n", &name, size + 1, size + 2));

                // body label.
                program.push_str(&format!("\n%entry_{}:\n", size + 1));
                program.push_str(&real_body);
                program.push_str(&format!("    jump %entry_{}\n", size));


                // after `while`.
                program.push_str(&format!("\n%entry_{}:\n", size + 2));


                return ExpRetType{size: size + 2, program, exp_res_id: 0, is_constant: false,};
            },
        }
    }
}



// closed_statement: non_if_statement
//                 | IF '(' expression ')' closed_statement ELSE closed_statement
impl ClosedStatement {
    pub fn eval(&self, scope: &HashMap<String, (i32, i32)>, size: i32) -> ExpRetType {
        let mut program = "".to_string();
        match self {
            ClosedStatement::Stmt(stmt) => {
                stmt.eval(scope, size)
            },
            ClosedStatement::Ifelse(exp, cs1, cs2) => {

                let exp_val = exp.eval(scope, size, false);
                let cs1_val = cs1.eval(scope, exp_val.size);
                let cs2_val = cs2.eval(scope, cs1_val.size);
                let size = cs2_val.size + 1;
                let name = get_name(exp_val.exp_res_id, exp_val.is_constant);

                // evaluate the condition.
                program.push_str(&exp_val.program);
                program.push_str(&format!("    br {}, %entry_{}, %entry_{}\n", &name, size, size + 1));

                program.push_str(&format!("\n%entry_{}:\n", size)); // first part.
                program.push_str(&cs1_val.program);
                program.push_str(&format!("    jump %entry_{}\n", size + 2));

                program.push_str(&format!("\n%entry_{}:\n", size + 1)); // second part.
                program.push_str(&cs2_val.program);
                program.push_str(&format!("    jump %entry_{}\n", size + 2));

                program.push_str(&format!("\n%entry_{}:\n", size + 2)); // after.

                return ExpRetType{size: size + 2, program, exp_res_id: 0, is_constant: false,};
            },
        }
    }
}



impl Stmt {
    pub fn eval(&self, scope: &HashMap<String, (i32, i32)>, size: i32) -> ExpRetType {
        // Stmt ::= LVal "=" Exp ";"| "return" Exp ";";
        let mut size = size;
        let mut program = "".to_string();
        match self {
            Stmt::LvalExp(lval, exp) => {
                // query the scope to find variable id, and change it.
                let var = lval.eval(scope, size, false); // assignment must be `int` variable.
                let ret_val = exp.eval(scope, var.size, false);
                let name = get_name(ret_val.exp_res_id, ret_val.is_constant);
                size = ret_val.size;

                assert!(var.is_constant == false); // must be variable.
                program.push_str(&var.program);
                program.push_str(&ret_val.program);

                if lval.exps.len() == 0 {
                    // store %1, @x
                    program.push_str(&format!("    store {}, @var_{}\n", name, var.exp_res_id));
                } else {
                    // store %1, @x
                    program.push_str(&format!("    store {}, %var_{}\n", name, var.exp_res_id));
                }

                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: 0,
                    is_constant: false,
                }
            },
            Stmt::RetExp(exp) => {
                let instrs = exp.eval(scope, size, false);
                let name = get_name(instrs.exp_res_id, instrs.is_constant);
                // println!("asd123www: {}", instrs.exp_res_id);

                program.push_str(&instrs.program);
                program.push_str(&format!("    ret {}\n", &name));
                program.push_str(&format!("\n%entry_{}:\n", instrs.size + 1));

                return ExpRetType {
                    size: instrs.size + 1,
                    program: program,
                    exp_res_id: 0, // return stmt => -2;
                    is_constant: false,
                }
            },
            Stmt::RetNone() => {
                program.push_str(&format!("    ret \n",));
                program.push_str(&format!("\n%entry_{}:\n", size + 1));

                return ExpRetType {
                    size: size + 1,
                    program: program,
                    exp_res_id: 0, // return stmt => -2;
                    is_constant: false,
                }
            },

            Stmt::SingleExp(exp) => {
                let ret_val = exp.eval(scope, size, false);
                ret_val
            },

            Stmt::Block(block) => {
                let ret_val = dfs(TreePoint::Block(block), &scope, size);

                return ExpRetType {
                    size: ret_val.size,
                    program: ret_val.program,
                    exp_res_id: ret_val.exp_res_id, // return stmt => -2;
                    is_constant: false,
                };
            },

            Stmt::ZeroExp() => {
                ExpRetType {
                    size, program, exp_res_id: 0, 
                    is_constant: false,
                }
            }

            Stmt::BreakKeyWord() => { // give `while` a hint, it'll replace it with `jump`.
                program.push_str("    <replace_me_with_break>\n");
                program.push_str(&format!("\n%entry_{}:\n", size + 1));
                ExpRetType {
                    size: size + 1, program, exp_res_id: 0, 
                    is_constant: false,
                }
            }
            Stmt::ContinueKeyWord() => {
                program.push_str("    <replace_me_with_continue>\n");
                program.push_str(&format!("\n%entry_{}:\n", size + 1));
                ExpRetType {
                    size: size + 1, 
                    program, 
                    exp_res_id: 0,
                    is_constant: false,
                }
            }
        }
    }
}