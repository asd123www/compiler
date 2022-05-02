
mod ret_types;
mod expression;
mod declare;
mod statement;

use ret_types::*;
use crate::ast::*;
use std::collections::HashMap;
use crate::koopa_ir_gen::declare::DeclResult;



enum TreePoint<'a> {
    CompUnit(CompUnit),
    FuncDef(FuncDef),
    FuncType(FuncType),
    Block(&'a Block),
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

            program.push_str(&format!("\n%entry_{}:\n", ret_val.size + 1));
            let body = dfs(TreePoint::Block(&node.block), &scope, ret_val.size + 1);
            program.push_str(&body.program);
            program.push_str(&format!("    ret 0\n"));

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
            for item in &node.items { // enumerate the blocks in body.
                // set the label.
                let block = item.eval(&mut scope, size);
                size = block.size;
                program.push_str(&block.program);
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