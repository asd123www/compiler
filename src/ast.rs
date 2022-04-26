

// CompUnit  ::= FuncDef;
#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}


// FuncDef   ::= FuncType IDENT "(" ")" Block;
#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

// FuncType  ::= "int";
#[derive(Debug)]
pub enum FuncType {
    Int,
    Void,
    Double,
    Float,
    String,
}

// Block     ::= "{" Stmt "}";
#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

// Stmt      ::= "return" Number ";";
#[derive(Debug)]
pub struct Stmt {
    pub ret_number: i32,
}

// // Number    ::= INT_CONST;
// #[derive(Debug)]
// pub struct Number {
//     pub digit_str: String,
// }