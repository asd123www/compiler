use crate::ast::*;



enum TreePoint {
    CompUnit(CompUnit),
    FuncDef(FuncDef),
    FuncType(FuncType),
    Block(Block),
    Stmt(Stmt),
}

/*
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
} */

fn dfs(pt: TreePoint, shift: &str) -> String {
    // consider the indent!
    let mut program = String::from("");

    match pt {
        TreePoint::CompUnit(node) => {
            let program = dfs(TreePoint::FuncDef(node.func_def), shift);
            return program;
        },
        TreePoint::FuncDef(node) => {
            program.push_str(&format!("fun @{}(): ", node.ident));

            // get the type of return value.
            let ret_type = dfs(TreePoint::FuncType(node.func_type), shift);
            program.push_str(&ret_type);

            // begin the structure of body.
            program.push_str(" {\n");
            let body = dfs(TreePoint::Block(node.block), shift);
            program.push_str(&body);
            program.push_str("}\n");

            return program;
        },
        TreePoint::FuncType(node) => {
            match node {
                FuncType::Int => "i32".to_string(),
                _ => "WrongType".to_string(),
            }
        },
        TreePoint::Block(node) => {
            program.push_str("%entry:\n");
            let statement = dfs(TreePoint::Stmt(node.stmt), shift);
            program.push_str(&statement);
            return program;
        },
        TreePoint::Stmt(node) => {
            program.push_str("ret ");
            // program.push_str(&node.ret_number.to_string());
            program.push_str("\n");

            return program;
        },
        _ => "wrong".to_string()
    }
}


pub fn Generator(start: CompUnit) -> String {
    return dfs(TreePoint::CompUnit(start), "");
}