
mod ret_types;
mod expression;
mod declare;

use ret_types::*;
use crate::ast::*;
use std::collections::HashMap;
use crate::koopa_ir_gen::expression::ExpResult;
use crate::koopa_ir_gen::declare::DeclResult;


impl Stmt {
    fn eval(&self, scope: &HashMap<String, i32>, size: i32) -> ExpRetType {
        // Stmt ::= LVal "=" Exp ";"| "return" Exp ";";
        let mut size = size;
        let mut program = "".to_string();
        match self {
            Stmt::LvalExp(lval, exp) => {
                // query the scope to find variable id, and change it.
                let id = scope.get(&lval.ident).unwrap();
                let ret_val = exp.eval(scope, size);
                size = ret_val.size;
                program.push_str(&ret_val.program);
                // store %1, @x
                program.push_str(&format!("    store %var_{}, @var_{}\n", ret_val.exp_res_id, id >> 1));

                return ExpRetType {
                    size: size,
                    program: program,
                    exp_res_id: BODY_STATE,
                }
            },
            Stmt::RetExp(exp) => {
                let instrs = exp.eval(scope, size);
                program.push_str(&instrs.program);
                program.push_str(&format!("    ret %var_{}\n", instrs.exp_res_id));

                return ExpRetType {
                    size: instrs.size + 1,
                    program: program,
                    exp_res_id: RETURN_STATE, // return stmt => -2;
                }
            },
        }
    }
}


enum TreePoint {
    CompUnit(CompUnit),
    FuncDef(FuncDef),
    FuncType(FuncType),
    Block(Block),
}


// tranverse the syntax tree to translate.
// return (size, Program), size for unique identify of the node.
fn dfs(pt: TreePoint, par: &HashMap<String, i32>, size: i32) -> ExpRetType {
    // consider the indent!
    let mut size = size;
    let mut program = String::from("");
    let mut scope: HashMap<String, i32> = par.clone(); // inherit the variables from parent.

    match pt {

        // CompUnit      ::= FuncDef;
        TreePoint::CompUnit(node) => {
            let program = dfs(TreePoint::FuncDef(node.func_def), &scope, size);
            return program;
        },

        // FuncDef       ::= FuncType IDENT "(" ")" Block;
        TreePoint::FuncDef(node) => {
            program.push_str(&format!("fun @{}(): ", node.ident));

            // get the type of return value.
            let ret_val = dfs(TreePoint::FuncType(node.func_type), &scope, size);
            program.push_str(&ret_val.program);


            // begin the structure of body.
            program.push_str(" {\n");

            let body = dfs(TreePoint::Block(node.block), &scope, ret_val.size);
            program.push_str(&body.program);

            program.push_str("}\n");

            return ExpRetType {
                size: body.size,
                program: program,
                exp_res_id: -1,
            };
        },

        TreePoint::FuncType(node) => {
            match node {
                FuncType::Int => ExpRetType {
                    size: size + 1,
                    program: "i32".to_string(),
                    exp_res_id: -1,
                },
            }
        },
        
        // Block ::= "{" {BlockItem} "}";
        TreePoint::Block(node) => {
            let mut is_ret = false;

            for item in node.items { // enumerate the blocks in body.
                // set the label.
                program.push_str(&format!("\n%entry_{}:\n", size + 1));
                let block = item.eval(&mut scope, size + 1);
                size = block.size;
                program.push_str(&block.program);
                is_ret = block.flag == RETURN_STATE;
                if !is_ret {
                    program.push_str(&format!("    jump %entry_{}\n", size + 1));
                }
                println!("flag: {}\n", block.flag);
            }
            return ExpRetType {
                size: size,
                program: program,
                exp_res_id: -1,
            };
        },
    }
}


pub fn generator(start: CompUnit) -> String {
    let size = 0;
    let scope: HashMap<String, i32> = HashMap::new();

    return dfs(TreePoint::CompUnit(start), &scope, size).program;
}