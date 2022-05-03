mod ret_types;
mod expression;
mod declare;
mod statement;

use ret_types::*;
use crate::ast::*;
use core::{panic};
use std::{collections::HashMap};
use crate::koopa_ir_gen::declare::DeclResult;

// use self::expression::ExpResult;


// return variable name based on whether it's constant or not.
fn get_name(id: i32, is_constant: bool) -> String {
    if is_constant {
        id.to_string()
    } else {
        format!("%var_{}", id)
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
fn dfs(pt: TreePoint, par: &HashMap<String, (bool, i32)>, size: i32) -> BodyRetType {
    // consider the indent!
    let mut size = size;
    let mut program = String::from("");
    let mut scope: HashMap<String, (bool, i32)> = par.clone(); // inherit the variables from parent.

    match pt {

        // CompUnit ::= [CompUnit] FuncDef;
        TreePoint::CompUnit(node) => {
            fn insert_function(scope: &mut HashMap<String, (bool, i32)>, func_def: &FuncDef) {
                // insert the function definition.
                println!("insert: {}\n", &func_def.ident);
                match func_def.func_type {
                    0 => { // int
                        scope.insert(format!("{}_function", &func_def.ident), (false, 0));
                    },
                    1 => {
                        scope.insert(format!("{}_function", &func_def.ident), (true, 1));
                    },
                    _ => {panic!("No function type labeled this.");}
                }
            }
            for pair in &node.funcs {
                match &pair {
                    DeclFuncPair::Func(func) => {
                        insert_function(&mut scope, &func);
                        let func_val = dfs(TreePoint::FuncDef(func), &scope, size);
        
                        program.push_str(&func_val.program);
                        size = func_val.size;
                    },
                    DeclFuncPair::Decl(decl) => {

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
            program.push_str(&format!("fun @{}(", node.ident));

            // if we have parameter, we have to create variables.
            match &node.params {
                None => {},
                Some(v) => {
                    for x in &v.params {
                        // %x = alloc i32
                        // store @x, %x
                        // we only have i32, hhhh.
                        size += 1;
                        load_params.push_str(&format!("    @var_{} = alloc i32\n", size));
                        load_params.push_str(&format!("    store @{}, @var_{}\n", x.ident , size));

                        // add parameter to scope. And parameter is variable.
                        scope.insert(format!("{}", x.ident), (false, size));
                    }
                    let param_val = dfs(TreePoint::FuncFParams(v), &scope, size);
                    program.push_str(&param_val.program);
                    
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
                exp_res_id: -1,
            };
        },

        TreePoint::FuncFParams(node) => {
            let mut is_first = true;
            for x in &node.params {
                // maybe pointer in the future????
                if is_first {
                    program.push_str(&format!("@{}: i32", x.ident));
                } else {
                    program.push_str(&format!(", @{}: i32", x.ident));
                }
                is_first = false;
            }

            return BodyRetType {
                size: size,
                program: program,
                exp_res_id: -1,
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
    let mut scope: HashMap<String, (bool, i32)> = HashMap::new();

    // add std::functions to scope.
    scope.insert("getint_function".to_string(), (false, 0));
    scope.insert("getch_function".to_string(), (false, 0));
    scope.insert("getarray_function".to_string(), (false, 0));
    scope.insert("putint_function".to_string(), (true, 1));
    scope.insert("putch_function".to_string(), (true, 1));
    scope.insert("putarray_function".to_string(), (true, 1));
    scope.insert("starttime_function".to_string(), (true, 1));
    scope.insert("stoptime_function".to_string(), (true, 1));

    let result = dfs(TreePoint::CompUnit(start), &scope, size);
    program.push_str(&result.program);

    return program;
}