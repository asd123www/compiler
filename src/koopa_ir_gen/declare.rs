use crate::ast::*;


// use super::ret_types::*;
use std::collections::HashMap;
use crate::koopa_ir_gen::initialvalue::InitValue;
use crate::koopa_ir_gen::{*};
use super::ret_types::InitRetType;

/*
 * decl, 声明变量.
 * 例如数组, integer变量等.
 * 这里对于global和non-global还得分开处理...
 */

pub trait DeclResult {
    fn eval(&self, scope: &mut HashMap<String, (i32, i32)>, size: i32, is_global: bool) -> DeclRetType;
}


impl DeclResult for BlockItem {
    fn eval(&self, scope: &mut HashMap<String, (i32, i32)>, size: i32, is_global: bool) -> DeclRetType {
        // BlockItem ::= Decl | Stmt;
        match self {
            BlockItem::Statement(stmt) => {
                let statement = stmt.eval(&scope, size);
                return DeclRetType {
                    size: statement.size,
                    program: statement.program,
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
    fn eval(&self, scope: &mut HashMap<String, (i32, i32)>, size: i32, is_global: bool) -> DeclRetType {
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
    fn eval(&self, scope: &mut HashMap<String, (i32, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let mut size = size;
        let mut program = "".to_string();

        for def in &self.constdefs {
            let ret_val = def.eval(scope, size, is_global);
            program.push_str(&ret_val.program);
            size = ret_val.size;
        }
        return DeclRetType{size, program};
    }
}

// VarDecl ::= BType VarDef {"," VarDef} ";";
impl DeclResult for VarDecl {
    fn eval(&self, scope: &mut HashMap<String, (i32, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let mut size = size;
        let mut program = "".to_string();

        for def in &self.vardefs {
            let ret_val = def.eval(scope, size, is_global);
            program.push_str(&ret_val.program);
            size = ret_val.size;
        }
        return DeclRetType{size, program};
    }
}




// calculate a list of `ConstExp`.
pub fn evaluate_dimension(size: &mut i32, exps: & Vec<ConstExp>, scope: &HashMap<String, (i32, i32)>) -> (Vec<i32>, String) {
    let mut dims = Vec::new();
    let mut is_first = true;
    let mut program = "".to_string();

    for const_exp in exps {
        let ret_val = const_exp.eval(scope, *size);
        *size = ret_val.size;

        assert!(ret_val.val.len() == 1);
        assert!(ret_val.val[0].0); // must be constant.
        dims.push(ret_val.val[0].1); // add length of this dimension.
    }
    for x in dims.iter().rev() { // the order is reversed.
        if is_first {
            program = format!("[i32, {}]", x);
        } else {
            program = format!("[{}, {}]", program, x);
        }
        is_first = false;
    }
    (dims, program)
}



// 吧这个东西铺开... {{}, {}, {}} ...
fn get_const_init_value_str(p: &InitRetType, dims: &Vec<i32>) -> String {
    if p.is_allzero {
        return "zeroinit".to_string();
    }

    fn dfs(i: usize, l: usize, r: usize, p: &InitRetType, dims: &Vec<i32>) -> String {
        // println!("index: {}, left: {}, right: {}\n", i, l, r);
        if l == r {
            assert!(p.val[l].0);
            return format!("{}", p.val[l].1);
        }
        let len = (r - l + 1) / (dims[i] as usize);
        let mut program = "".to_string();
        let mut is_first = true;
        for j in 0..dims[i] {
            let x = dfs(i + 1, l + (j as usize) * len, l + (j as usize) * len + len - 1, p, dims);
            if is_first {
                program.push('{');
                program.push_str(&format!("{}", x));
                is_first = false;
            } else {
                program.push_str(&format!(", {}", x));
            }
        }
        program.push('}');

        program
    }

    dfs(0, 0, p.val.len() - 1, p, dims)
}


// ConstDef ::= IDENT {"[" ConstExp "]"} "=" ConstInitVal
impl DeclResult for ConstDef {
    fn eval(&self, scope: &mut HashMap<String, (i32, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let mut size = size;
        let dim_pair = evaluate_dimension(&mut size, &self.dims, scope);
        let dims = dim_pair.0;
        let dim_str = dim_pair.1;

        let ret_val = self.constinitval.eval(scope, size, &dims[0..dims.len()]);
        let size = ret_val.size;

        if self.dims.len() == 0 { // `int` variable.
            assert!(ret_val.val.len() == 1);
            // the constant's value is the expression.
            scope.insert(format!("{}", self.ident), (CONSTANT_INT, ret_val.val[0].1));
            return DeclRetType {size, program: ret_val.program};
        }

        // array.
        // array_init(&dims, is_global);

        // @arr = alloc [[i32, 3], 2]    // @arr 的类型是 *[[i32, 3], 2]
        // %ptr1 = getelemptr @arr, 1    // %ptr1 的类型是 *[i32, 3]
        // %ptr2 = getelemptr %ptr1, 2   // %ptr2 的类型是 *i32
        // %value = load %ptr2           // %value 的类型是 i32
        let mut program = "".to_string();
        let init_value_str = get_const_init_value_str(&ret_val, &dims);

        // wrong???? 把常量数组当作变量数组.
        scope.insert(format!("{}", self.ident), (VARIABLE_ARRAY, size + 1));
        if is_global {
            // global @x = alloc [i32, 2], {10, 20}
            program.push_str(&format!("global @var_{} = alloc {}, {}\n", size + 1, &dim_str, init_value_str));
        } else {
            // @arr = alloc [i32, 5]
            // store {1, 2, 3, 0, 0}, @arr
            program.push_str(&format!("    @var_{} = alloc {}\n", size + 1, &dim_str));
            program.push_str(&format!("    store {}, @var_{}\n", init_value_str, size + 1));
        }
        return DeclRetType {size: size + 1, program: program};
    }
}

// VarDef ::= IDENT {"[" ConstExp "]"}
//          | IDENT {"[" ConstExp "]"} "=" InitVal
impl DeclResult for VarDef {
    fn eval(&self, scope: &mut HashMap<String, (i32, i32)>, size: i32, is_global: bool) -> DeclRetType {
        let mut size = size;
        let mut program = "".to_string();

        match self {
            VarDef::Ident(ident, dims) => {
                let dim_pair = evaluate_dimension(&mut size, dims, scope);
                let dims = dim_pair.0;
                let dim_str = dim_pair.1;
                
                if dims.len() == 0 { // `int` variable.
                    // define.
                    scope.insert(format!("{}", ident), (VARIABLE_INT, size + 1));
                    if !is_global {
                        // @x = alloc i32
                        program.push_str(&format!("    @var_{} = alloc i32\n", size + 1)); // currently only i32.
                    } else {
                        // global @var = alloc i32, zeroinit
                        program.push_str(&format!("global @var_{} = alloc i32, zeroinit\n", size + 1)); // currently only i32.
                    }
                    return DeclRetType {size: size + 1, program};
                }

                scope.insert(format!("{}", ident), (VARIABLE_ARRAY, size + 1));
                // array.
                if is_global {
                    // global @x = alloc [i32, 2], {10, 20}
                    program.push_str(&format!("global @var_{} = alloc {}, zeroinit\n", size + 1, &dim_str));
                } else {
                    // @arr = alloc [i32, 5]
                    // store {1, 2, 3, 0, 0}, @arr
                    program.push_str(&format!("    @var_{} = alloc {}\n", size + 1, &dim_str));
                }
                return DeclRetType {size: size + 1, program};
            },

            VarDef::Identinitval(ident, dims, initval) => {
                let dim_pair = evaluate_dimension(&mut size, dims, scope);
                let dims = dim_pair.0;
                let dim_str = dim_pair.1;

                if dims.len() == 0 {
                    let ret_val = initval.eval(scope, size, &[1]);
                    size = ret_val.size + 1;
                    assert!(ret_val.val.len() == 1);
                    let name = get_name(ret_val.val[0].1, ret_val.val[0].0);
    
                    if !is_global {
                        program.push_str(&ret_val.program);
                        // define.
                        scope.insert(format!("{}", ident), (VARIABLE_INT, size + 1));
                        // @x = alloc i32
                        program.push_str(&format!("    @var_{} = alloc i32\n", size + 1)); // currently only i32.
                        // assignment: store %1, @x
                        program.push_str(&format!("    store {}, @var_{}\n", name, size + 1)); // currently only i32.
                    } else { // global的初始值必须是constant.
                        assert!(ret_val.val[0].0 == true); // must be constant.
                        // define.
                        scope.insert(format!("{}", ident), (VARIABLE_INT, size + 1));
                        // @x = alloc i32
                        program.push_str(&format!("global @var_{} = alloc i32, {}\n", size + 1, ret_val.val[0].1)); // currently only i32.
                    }
                    return DeclRetType {size: size + 1, program};
                }

                // array.
                let ret_val = initval.eval(scope, size, &dims);
                let init_value_str = get_const_init_value_str(&ret_val, &dims);

                scope.insert(format!("{}", ident), (VARIABLE_ARRAY, size + 1));
                if is_global {
                    program.push_str(&format!("global @var_{} = alloc {}, {}\n", size + 1, &dim_str, init_value_str));
                } else {
                    program.push_str(&format!("    @var_{} = alloc {}\n", size + 1, &dim_str));
                    program.push_str(&format!("    store {}, @var_{}\n", init_value_str, size + 1));
                }

                return DeclRetType {size: size + 1, program};
            },
        }
    }
}
