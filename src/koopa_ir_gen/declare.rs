use crate::ast::*;
use crate::koopa_ir_gen::DeclRetType;


// use super::ret_types::*;
use crate::koopa_ir_gen::get_name;
use core::panic;
use std::collections::HashMap;
use crate::koopa_ir_gen::initialvalue::InitValue;


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




// calculate a list of `ConstExp`.
fn evaluate_dimension(size: &mut i32, exps: & Vec<ConstExp>, scope: &HashMap<String, (bool, i32)>) -> Vec<i32> {
    let mut dims = Vec::new();
    for const_exp in exps {
        let ret_val = const_exp.eval(scope, *size);
        *size = ret_val.size;

        assert!(ret_val.val.len() == 1);
        assert!(ret_val.val[0].0); // must be constant.
        dims.push(ret_val.val[0].1); // add length of this dimension.
    }
    dims
}


// ConstDef ::= IDENT {"[" ConstExp "]"} "=" ConstInitVal
impl DeclResult for ConstDef {
    fn eval(&self, scope: &mut HashMap<String, (bool, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let mut size = size;
        let dims = evaluate_dimension(&mut size, &self.dims, scope);

        let ret_val = self.constinitval.eval(scope, size, &dims[0..dims.len()]);
        let size = ret_val.size;

        if self.dims.len() == 0 { // `int` variable.
            assert!(ret_val.val.len() == 1);
            // the constant's value is the expression.
            scope.insert(format!("{}", self.ident), (true, ret_val.val[0].1));
            return DeclRetType {size, program: ret_val.program, flag: 0};
        }

        // array. 
        panic!("fuck");
    }
}

// VarDef ::= IDENT {"[" ConstExp "]"}
//          | IDENT {"[" ConstExp "]"} "=" InitVal
impl DeclResult for VarDef {
    fn eval(&self, scope: &mut HashMap<String, (bool, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let mut size = size;
        let mut program = "".to_string();

        match self {
            VarDef::Ident(ident, dims) => {
                let dims = evaluate_dimension(&mut size, dims, scope);

                if dims.len() == 0 { // `int` variable.
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
                }

                // array. 
                panic!("fuck");
            },
            VarDef::Identinitval(ident, dims, initval) => {
                let dims = evaluate_dimension(&mut size, dims, scope);

                if dims.len() == 0 {
                    let ret_val = initval.eval(scope, size, &[1]);
                    size = ret_val.size + 1;
                    assert!(ret_val.val.len() == 1);
                    let name = get_name(ret_val.val[0].1, ret_val.val[0].0);
    
                    if !is_global {
                        program.push_str(&ret_val.program);
                        // define.
                        scope.insert(format!("{}", ident), (false, size + 1));
                        // @x = alloc i32
                        program.push_str(&format!("    @var_{} = alloc i32\n", size + 1)); // currently only i32.
                        // assignment: store %1, @x
                        program.push_str(&format!("    store {}, @var_{}\n", name, size + 1)); // currently only i32.
                    } else {
                        assert!(ret_val.val[0].0 == true); // must be constant.
                        // define.
                        scope.insert(format!("{}", ident), (false, size + 1));
                        // @x = alloc i32
                        program.push_str(&format!("global @var_{} = alloc i32, {}\n", size + 1, ret_val.val[0].1)); // currently only i32.
                    }
                    return DeclRetType {size: size + 1, program, flag: 0};
                }

                // array. 
                panic!("fuck");
            },
        }
    }
}
