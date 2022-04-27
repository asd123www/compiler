
mod ret_types;
mod expression;


use ret_types::*;
use crate::ast::*;
use crate::koopa_ir_gen::expression::ExpResult;

enum TreePoint {
    CompUnit(CompUnit),
    FuncDef(FuncDef),
    FuncType(FuncType),
    Block(Block),
    Stmt(Stmt),
}

// tranverse the syntax tree to translate.
// return (size, Program), size for unique identify of the node.
fn dfs(pt: TreePoint, size: i32) -> ExpRetType {
    // consider the indent!
    let mut program = String::from("");

    match pt {

        TreePoint::CompUnit(node) => {
            let mut program = dfs(TreePoint::FuncDef(node.func_def), size);
            program.size += 1;
            return program;
        },

        TreePoint::FuncDef(node) => {
            program.push_str(&format!("fun @{}(): ", node.ident));

            // get the type of return value.
            let ret_val = dfs(TreePoint::FuncType(node.func_type), size);
            program.push_str(&ret_val.program);


            // begin the structure of body.
            program.push_str(" {\n");
            let body = dfs(TreePoint::Block(node.block), ret_val.size);
            program.push_str(&body.program);

            program.push_str("}\n");

            return ExpRetType {
                size: body.size + 1,
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
                _ => ExpRetType {
                    size: size + 1,
                    program: "WrongType".to_string(),
                    exp_res_id: -1,
                },
            }
        },

        TreePoint::Block(node) => {
            let statement = dfs(TreePoint::Stmt(node.stmt), size);
            program.push_str(&statement.program);
            return ExpRetType {
                size: statement.size + 1,
                program: program,
                exp_res_id: statement.exp_res_id,
            };
        },

        TreePoint::Stmt(node) => {
            program.push_str(&format!("\n%entry{}:\n", size));

            let instrs = node.exp.eval(size);
            program.push_str(&instrs.program);

            program.push_str(&format!("    ret %var_{}\n", instrs.exp_res_id));

            return ExpRetType {
                size: instrs.size + 1,
                program: program,
                exp_res_id: instrs.exp_res_id,
            }
        },

        _ => ExpRetType{size: 0, 
                    program: "wrong".to_string(),
                    exp_res_id: -1},
    }
}


pub fn generator(start: CompUnit) -> String {
    let size = 0;
    return dfs(TreePoint::CompUnit(start), size).program;
}