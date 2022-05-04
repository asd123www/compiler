use crate::ast::*;
use crate::koopa_ir_gen::DeclRetType;


// use super::ret_types::*;
use crate::koopa_ir_gen::get_name;
use std::collections::HashMap;
use crate::koopa_ir_gen::expression::ExpResult;


pub trait DeclResult {
    fn eval(&self, scope: &mut HashMap<String, (bool, i32)>, size: i32, is_global: bool) -> DeclRetType;
}


impl DeclResult for BlockItem {
    fn eval(&self, scope: &mut HashMap<String, (bool, i32)>, size: i32, is_global: bool) -> DeclRetType {
        // BlockItem ::= Decl | Stmt;
        match self {
            BlockItem::Statement(stmt) => {
                let statement = stmt.eval(&scope, size);
                return DeclRetType {
                    size: statement.size,
                    program: statement.program,
                    flag: statement.exp_res_id,
                };
            },
            BlockItem::Decl(decl) => {
                let decl_ret_val = decl.eval(scope, size, is_global);
                return decl_ret_val;
            }
        }
    }
}

// Decl ::= ConstDecl | VarDecl;
impl DeclResult for Decl {
    fn eval(&self, scope: &mut HashMap<String, (bool, i32)>, size: i32, is_global: bool) -> DeclRetType {
        match self {
            Decl::Constdecl(constdecl) => {
                let ret_val = constdecl.eval(scope, size, is_global);
                return ret_val;
            },
            Decl::Vardecl(vardecl) => {
                let ret_val = vardecl.eval(scope, size, is_global);
                return ret_val;
            },
        }
    }
}

// ConstDecl ::= "const" BType ConstDef {"," ConstDef} ";";
impl DeclResult for ConstDecl {
    fn eval(&self, scope: &mut HashMap<String, (bool, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let mut size = size;
        let mut program = "".to_string();

        for def in &self.constdefs {
            let ret_val = def.eval(scope, size, is_global);
            program.push_str(&ret_val.program);
            size = ret_val.size;
        }
        return DeclRetType{size, program, flag: 0};
    }
}

// VarDecl ::= BType VarDef {"," VarDef} ";";
impl DeclResult for VarDecl {
    fn eval(&self, scope: &mut HashMap<String, (bool, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let mut size = size;
        let mut program = "".to_string();

        for def in &self.vardefs {
            let ret_val = def.eval(scope, size, is_global);
            program.push_str(&ret_val.program);
            size = ret_val.size;
        }
        return DeclRetType{size, program, flag: 0};
    }
}


// ConstDef ::= IDENT "=" ConstInitVal;    old one.
// ConstDef ::= IDENT {"[" ConstExp "]"} "=" ConstInitVal
// wrong!!!
impl DeclResult for ConstDef {
    fn eval(&self, scope: &mut HashMap<String, (bool, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let ret_val = self.constinitval.eval(scope, size);
        let size = ret_val.size;

        assert!(ret_val.is_constant == true);
        // the constant's value is the expression.
        scope.insert(format!("{}", self.ident), (true, ret_val.exp_res_id));

        return DeclRetType {size, program: ret_val.program, flag: 0};
    }
}

// VarDef ::= IDENT | IDENT "=" InitVal;
impl DeclResult for VarDef {
    fn eval(&self, scope: &mut HashMap<String, (bool, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let mut program = "".to_string();
        match self {
            VarDef::Ident(ident) => {
                // define.
                scope.insert(format!("{}", ident), (false, size + 1));

                if !is_global {
                    // @x = alloc i32
                    program.push_str(&format!("    @var_{} = alloc i32\n", size + 1)); // currently only i32.
                } else {
                    // global @var = alloc i32, zeroinit
                    program.push_str(&format!("global @var_{} = alloc i32, zeroinit\n", size + 1)); // currently only i32.
                }

                return DeclRetType {size: size + 1, program, flag: 0};
            },
            VarDef::Identinitval(ident, initval) => {
                let ret_val = initval.eval(scope, size);
                let name = get_name(ret_val.exp_res_id, ret_val.is_constant);
                let size = ret_val.size;

                if !is_global {
                    program.push_str(&ret_val.program);
                    // define.
                    scope.insert(format!("{}", ident), (false, size + 1));
                    // @x = alloc i32
                    program.push_str(&format!("    @var_{} = alloc i32\n", size + 1)); // currently only i32.
                    // assignment: store %1, @x
                    program.push_str(&format!("    store {}, @var_{}\n", name, size + 1)); // currently only i32.
                } else {
                    assert!(ret_val.is_constant == true); // must be constant.
                    // define.
                    scope.insert(format!("{}", ident), (false, size + 1));
                    // @x = alloc i32
                    program.push_str(&format!("global @var_{} = alloc i32, {}\n", size + 1, ret_val.exp_res_id)); // currently only i32.
                }
                return DeclRetType {size: size + 1, program, flag: 0};
            },
        }
    }
}
