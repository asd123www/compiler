use crate::ast::*;
use crate::koopa_ir_gen::DeclRetType;


use std::collections::HashMap;
use crate::koopa_ir_gen::expression::ExpResult;


pub trait DeclResult {
    fn eval(&self, scope: &mut HashMap<String, i32>, size: i32) -> DeclRetType;
}


impl DeclResult for BlockItem {
    fn eval(&self, scope: &mut HashMap<String, i32>, size: i32) -> DeclRetType {
        // BlockItem ::= Decl | Stmt;
        let mut program = "".to_string();
        match self {
            BlockItem::Stmt(stmt) => {
                let statement = stmt.eval(&scope, size);
                return DeclRetType {
                    size: statement.size,
                    program: statement.program,
                };
            },
            BlockItem::Decl(decl) => {
                let decl_ret_val = decl.eval(scope, size);
                return DeclRetType {
                    size: decl_ret_val.size,
                    program: decl_ret_val.program,
                };
            }
        }
    }
}

// Decl ::= ConstDecl | VarDecl;
impl DeclResult for Decl {
    fn eval(&self, scope: &mut HashMap<String, i32>, size: i32) -> DeclRetType {
        match self {
            Decl::Constdecl(constdecl) => {
                let ret_val = constdecl.eval(scope, size);
                return ret_val;
            },
            Decl::Vardecl(vardecl) => {
                let ret_val = vardecl.eval(scope, size);
                return ret_val;
            },
        }
    }
}

// ConstDecl ::= "const" BType ConstDef {"," ConstDef} ";";
impl DeclResult for ConstDecl {
    fn eval(&self, scope: &mut HashMap<String, i32>, size: i32) -> DeclRetType {
        let mut size = size;
        let mut program = "".to_string();

        let type_str = match self.btype {
            _ => "int",
        };
        for def in &self.constdefs {
            let ret_val = def.eval(scope, size);
            program.push_str(&ret_val.program);
            size = ret_val.size;
        }
        return DeclRetType{size, program};
    }
}

// VarDecl ::= BType VarDef {"," VarDef} ";";
impl DeclResult for VarDecl {
    fn eval(&self, scope: &mut HashMap<String, i32>, size: i32) -> DeclRetType {
        let mut size = size;
        let mut program = "".to_string();

        let type_str = match self.btype {
            _ => "int",
        };
        for def in &self.vardefs {
            let ret_val = def.eval(scope, size);
            program.push_str(&ret_val.program);
            size = ret_val.size;
        }
        return DeclRetType{size, program};
    }
}


// ConstDef ::= IDENT "=" ConstInitVal;
impl DeclResult for ConstDef {
    fn eval(&self, scope: &mut HashMap<String, i32>, size: i32) -> DeclRetType {
        let ret_val = self.constinitval.eval(size);
        let size = ret_val.size;

        // the constant's value is the expression.
        scope.insert(format!("imu_{}", self.ident), ret_val.exp_res_id);

        return DeclRetType {size, program: ret_val.program};
    }
}

// VarDef ::= IDENT | IDENT "=" InitVal;
impl DeclResult for VarDef {
    fn eval(&self, scope: &mut HashMap<String, i32>, size: i32) -> DeclRetType {
        let mut program = "".to_string();
        match self {
            VarDef::Ident(ident) => {
                // define.
                scope.insert(format!("mut_{}", ident), size + 1);
                // @x = alloc i32
                program.push_str(&format!("    @var_{} = alloc i32\n", size + 1)); // currently only i32.

                return DeclRetType {size: size + 1, program};
            },
            VarDef::Identinitval(ident, initval) => {
                let ret_val = initval.eval(size);
                let size = ret_val.size;
                // define.
                scope.insert(format!("mut_{}", ident), size + 1);
                // @x = alloc i32
                program.push_str(&format!("    @var_{} = alloc i32\n", size + 1)); // currently only i32.
                // assignment: store %1, @x
                program.push_str(&format!("    store %var_{}, @var_{}\n", ret_val.exp_res_id, size + 1)); // currently only i32.

                return DeclRetType {size: size + 1, program};
            },
        }
    }
}


// // BType ::= "int";
// TreePoint::BType(node) => {
// },