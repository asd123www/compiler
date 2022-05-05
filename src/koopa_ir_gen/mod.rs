mod ret_types;
mod expression;
mod declare;
mod statement;
mod initialvalue;

use ret_types::*;
use crate::ast::*;
use core::{panic};
use std::{collections::HashMap};
use crate::koopa_ir_gen::declare::DeclResult;
use crate::koopa_ir_gen::declare::evaluate_dimension;
// use self::expression::ExpResult;


// return variable name based on whether it's constant or not.
fn get_name(id: i32, is_constant: bool) -> String {
    if is_constant {
        id.to_string()
    } else {
        format!("%var_{}", id)
    }
}

fn calc_bitset(node: &FuncDef) -> i32 {
    match &node.params {
        None => {0},
        Some(v) => {
            let mut bitset = 0;
            let mut bin = 1;
            for x in &v.params {
                // maybe pointer in the future????  yes.
                match x {
                    FuncFParam::Integer(_) => {} ,
                    FuncFParam::Array(_, _) => bitset |= bin,
                }
                bin = bin << 1;
            }
            bitset
        },
    }
}

enum TreePoint<'a> {
    CompUnit(CompUnit),
    FuncDef(&'a FuncDef),
    FuncType(&'a FuncType),
    FuncFParams(&'a FuncFParams),
    Block(&'a Block),
}


// tranverse the syntax tree to translate.
// return (size, Program), size for unique identify of the node.
fn dfs(pt: TreePoint, par: &HashMap<String, (i32, i32)>, size: i32) -> BodyRetType {
    // consider the indent!
    let mut size = size;
    let mut program = String::from("");
    let mut scope: HashMap<String, (i32, i32)> = par.clone(); // inherit the variables from parent.

    match pt {

        // CompUnit ::= [CompUnit] FuncDef;
        TreePoint::CompUnit(node) => {
            fn insert_function(scope: &mut HashMap<String, (i32, i32)>, func_def: &FuncDef, bitset: i32) {
                // insert the function definition.
                // println!("insert: {}\n", &func_def.ident);
                match func_def.func_type {
                    0 => { // int
                        scope.insert(format!("{}_function", &func_def.ident), (VARIABLE_INT | (bitset << TYPE_BITS), 0));
                    },
                    1 => {
                        scope.insert(format!("{}_function", &func_def.ident), (VOID | (bitset << TYPE_BITS), 1));
                    },
                    _ => {panic!("No function type labeled this.");}
                }
            }
            for pair in &node.funcs {
                match &pair {
                    DeclFuncPair::Func(func) => {
                        let bitset = calc_bitset(&func);
                        insert_function(&mut scope, &func, bitset);
                        let func_val = dfs(TreePoint::FuncDef(func), &scope, size);
                        program.push_str(&func_val.program);

                        assert!(bitset == func_val.exp_res_id);

                        size = func_val.size;
                    },
                    DeclFuncPair::Decl(decl) => {
                        let decl_val = decl.eval(&mut scope, size, true);

                        program.push_str(&decl_val.program);
                        size = decl_val.size;
                    },
                }
            }

            return BodyRetType {
                size: size, 
                program, 
                exp_res_id: -1,
            };
        },
        
        // FuncDef     ::= FuncType IDENT "(" [FuncFParams] ")" Block;
        TreePoint::FuncDef(node) => {
            let mut load_params = "".to_string();
            program.push_str(&format!("\n\nfun @{}(", node.ident));

            let mut bitset = 0;
            // if we have parameter, we have to create variables.
            match &node.params {
                None => {},
                Some(v) => {
                    for x in &v.params {
                        // %x = alloc i32
                        // store @x, %x
                        // we only have i32, hhhh.
                        match x {
                            FuncFParam::Integer(ident) => {
                                size += 1;
                                load_params.push_str(&format!("    @var_{} = alloc i32\n", size));
                                load_params.push_str(&format!("    store @{}, @var_{}\n", ident , size));
        
                                // add parameter to scope. And parameter is variable.
                                scope.insert(format!("{}", ident), (VARIABLE_INT, size));
                            },
                            FuncFParam::Array(ident, dims) => {
                                size += 1;
                                let dim_pair = evaluate_dimension(&mut size, &dims, &scope);
                                if dims.len() == 0 {
                                    load_params.push_str(&format!("    @var_{} = alloc *i32\n", size));
                                } else {
                                    load_params.push_str(&format!("    @var_{} = alloc *{}\n", size, dim_pair.1));
                                }
                                load_params.push_str(&format!("    store @{}, @var_{}\n", ident, size));

                                // wrong!!! 如何区分参数到底是数组还是数字?
                                scope.insert(format!("{}", ident), (PARAMETER_ARRAY, size));
                            },
                        }
                    }
                    let param_val = dfs(TreePoint::FuncFParams(v), &scope, size);
                    program.push_str(&param_val.program);
                    bitset = param_val.exp_res_id;
                    
                    size = param_val.size;
                },
            }
            program.push_str(")");


            let ftype = match node.func_type {
                0 => FuncType::Int,
                1 => FuncType::Void,
                _ => panic!("No function type labeled this."),
            };

            // get the type of return value.
            let ret_val = dfs(TreePoint::FuncType(&ftype), &scope, size);


            program.push_str(&ret_val.program);
            // begin the structure of body.
            program.push_str(" {\n");

            // first label of the function.
            program.push_str(&format!("\n%entry_{}:\n", ret_val.size + 1));
            program.push_str(&load_params); // load the parameters.

            // get the body of the function.
            let body = dfs(TreePoint::Block(&node.block), &scope, ret_val.size + 1);
            program.push_str(&body.program);

            // supplement a final return value.
            if ret_val.exp_res_id == -1 {
                program.push_str(&format!("    ret 0\n"));
            } else {
                program.push_str(&format!("    ret\n"));
            }
            program.push_str("}\n\n\n");

            return BodyRetType {
                size: body.size,
                program: program,
                exp_res_id: bitset, // return the bitset to CompUnit, where functions were defined.
            };
        },

        // generate parameter.
        TreePoint::FuncFParams(node) => {
            let mut is_first = true;
            let mut bitset = 0;
            let mut bin = 1;
            for x in &node.params {
                // maybe pointer in the future????  yes.
                match x {
                    FuncFParam::Integer(ident) => {
                        if is_first {
                            program.push_str(&format!("@{}: i32", ident));
                        } else {
                            program.push_str(&format!(", @{}: i32", ident));
                        }
                    }
                    FuncFParam::Array(ident, dims) => {
                        let dim_pair = evaluate_dimension(&mut size, &dims, &scope);
                        if dims.len() == 0 { // zero special judge.
                            if is_first {
                                program.push_str(&format!("@{}: *i32", ident));
                            } else {
                                program.push_str(&format!(", @{}: *i32", ident));
                            }
                        } else {
                            if is_first {
                                program.push_str(&format!("@{}: *{}", ident, dim_pair.1));
                            } else {
                                program.push_str(&format!(", @{}: *{}", ident, dim_pair.1));
                            }
                        }
                        bitset |= bin; // this parame is 
                    }
                }
                bin = bin << 1;
                is_first = false;
            }

            return BodyRetType {
                size: size,
                program: program,
                exp_res_id: bitset, // return this for function define use.
            }
        },

        TreePoint::FuncType(node) => {
            match node {
                FuncType::Int => BodyRetType {
                    size: size + 1,
                    program: ": i32".to_string(),
                    exp_res_id: -1,
                },
                FuncType::Void => BodyRetType {
                    size: size + 1,
                    program: "".to_string(),
                    exp_res_id: -2,
                },
            }
        },
        
        // Block ::= "{" {BlockItem} "}";
        TreePoint::Block(node) => {
            for item in &node.items { // enumerate the blocks in body.
                // set the label.
                let block = item.eval(&mut scope, size, false);
                size = block.size;
                program.push_str(&block.program);
            }
            return BodyRetType {
                size: size,
                program: program,
                exp_res_id: -1,
            };
        },
    }
}


pub fn generator(start: CompUnit) -> String {
    let size = 0;
// global @var = alloc i32, 12\n\n
    let mut program = "
decl @getint(): i32
decl @getch(): i32
decl @getarray(*i32): i32
decl @putint(i32)
decl @putch(i32)
decl @putarray(i32, *i32)
decl @starttime()
decl @stoptime()\n\n\n\n".to_string();
    let mut scope: HashMap<String, (i32, i32)> = HashMap::new();

    // add std::functions to scope.
    scope.insert("getint_function".to_string(), (VARIABLE_INT, 0));
    scope.insert("getch_function".to_string(), (VARIABLE_INT, 0));
    scope.insert("getarray_function".to_string(), (VARIABLE_INT + (1 << TYPE_BITS), 0));
    scope.insert("putint_function".to_string(), (VOID, 1));
    scope.insert("putch_function".to_string(), (VOID, 1));
    scope.insert("putarray_function".to_string(), (VOID + (2 << TYPE_BITS), 1));
    scope.insert("starttime_function".to_string(), (VOID, 1));
    scope.insert("stoptime_function".to_string(), (VOID, 1));

    let result = dfs(TreePoint::CompUnit(start), &scope, size);
    program.push_str(&result.program);

    return program;
}