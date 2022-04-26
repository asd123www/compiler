use crate::ast::*;



struct RetType {
    size: i32,
    program: String,
    exp_res_id: i32,
}

enum TreePoint {
    CompUnit(CompUnit),
    FuncDef(FuncDef),
    FuncType(FuncType),
    Block(Block),
    Stmt(Stmt),


    Exp(Exp),
    PrimaryExp(PrimaryExp),
    UnaryExp(UnaryExp),
    MulExp(MulExp),
    AddExp(AddExp),
    RelExp(RelExp),
    EqExp(EqExp),
    LAndExp(LAndExp),
    LOrExp(LOrExp),
}

// tranverse the syntax tree to translate.
// return (size, Program), size for unique identify of the node.
fn dfs(pt: TreePoint, shift: &str, size: i32) -> RetType {
    // consider the indent!
    let mut program = String::from("");

    match pt {

        TreePoint::CompUnit(node) => {
            let mut program = dfs(TreePoint::FuncDef(node.func_def), shift, size);
            program.size += 1;
            return program;
        },

        TreePoint::FuncDef(node) => {
            program.push_str(&format!("fun @{}(): ", node.ident));


            // get the type of return value.
            let ret_val = dfs(TreePoint::FuncType(node.func_type), shift, size);
            program.push_str(&ret_val.program);


            // begin the structure of body.
            program.push_str(" {\n");
            let body = dfs(TreePoint::Block(node.block), shift, ret_val.size);
            program.push_str(&body.program);

            program.push_str("}\n");

            return RetType {
                size: body.size + 1,
                program: program,
                exp_res_id: -1,
            };
        },

        TreePoint::FuncType(node) => {
            match node {
                FuncType::Int => RetType {
                    size: size + 1,
                    program: "i32".to_string(),
                    exp_res_id: -1,
                },
                _ => RetType {
                    size: size + 1,
                    program: "WrongType".to_string(),
                    exp_res_id: -1,
                },
            }
        },

        TreePoint::Block(node) => {
            let statement = dfs(TreePoint::Stmt(node.stmt), shift, size);
            program.push_str(&statement.program);
            return RetType {
                size: statement.size + 1,
                program: program,
                exp_res_id: statement.exp_res_id,
            };
        },

        TreePoint::Stmt(node) => {
            program.push_str(&format!("%entry{}:\n", size));

            let instrs = dfs(TreePoint::Exp(node.exp), shift, size);
            program.push_str(&instrs.program);

            program.push_str(&format!("ret %var_{}\n", instrs.exp_res_id));

            return RetType {
                size: instrs.size + 1,
                program: program,
                exp_res_id: instrs.exp_res_id,
            }
        },

        // how to maintain the expression result?
        // 1. assign unique id to node for unique variable name.
        // 2. attach a `ret` to struct store the result ID.

        TreePoint::Exp(node) => {
            let ret_val = dfs(TreePoint::LOrExp(node.lorexp), shift, size);
            return RetType{
                size: ret_val.size + 1,
                program: ret_val.program,
                exp_res_id: ret_val.exp_res_id,
            };
        },

        TreePoint::PrimaryExp(node) => {
            match node {
                PrimaryExp::Exp(exp) => {
                    let ret_val = dfs(TreePoint::Exp(*exp), shift, size);

                    return RetType {
                        size: ret_val.size + 1,
                        program: ret_val.program,
                        exp_res_id: ret_val.exp_res_id,
                    };
                },
                PrimaryExp::Num(num) => {
                    program.push_str(&format!("%var_{} = add 0, {}\n", size + 1, num));

                    return RetType {
                        size: size + 1,
                        program: program,
                        exp_res_id: size + 1,
                    };
                }
            }
        },

        // pub enum UnaryExp {
        //     Primaryexp(PrimaryExp),
        //     Unaryexp(UnaryOp, Box<UnaryExp>),
        // }
        // UnaryExp    ::= PrimaryExp | UnaryOp UnaryExp;
        TreePoint::UnaryExp(node) => {
            match node {
                UnaryExp::Primaryexp(primaryexp) => {
                    let ret_val = dfs(TreePoint::PrimaryExp(primaryexp), shift, size);

                    return RetType {
                        size: ret_val.size + 1,
                        program: ret_val.program,
                        exp_res_id: ret_val.exp_res_id,
                    };
                },
                UnaryExp::Unaryexp(unaryop, unaryexp) => {
                    let ret_val = dfs(TreePoint::UnaryExp(*unaryexp), shift, size);
                    match unaryop {
                        UnaryOp::Add => {
                            return RetType {
                                size: ret_val.size + 1,
                                program: ret_val.program,
                                exp_res_id: ret_val.exp_res_id,
                            };
                        },
                        UnaryOp::Sub => {
                            let size = ret_val.size + 1;
                            program.push_str(&ret_val.program);
                            program.push_str(&format!("%var_{} = sub 0, %var_{}\n", size, ret_val.exp_res_id));

                            return RetType {
                                size: size,
                                program: program,
                                exp_res_id: size,
                            };
                        },
                        UnaryOp::Exclamation => {
                            let size = ret_val.size + 1;
                            program.push_str(&ret_val.program);
                            program.push_str(&format!("%var_{} = eq 0, %var_{}\n", size, ret_val.exp_res_id));

                            return RetType {
                                size: size,
                                program: program,
                                exp_res_id: size,
                            };
                        },
                    }
                }
            }
        },

        TreePoint::MulExp(node) => {
            match node {
                MulExp::Unaryexp(unaryexp) => {
                    let ret_val = dfs(TreePoint::UnaryExp(unaryexp), shift, size);

                    return RetType {
                        size: ret_val.size + 1,
                        program: ret_val.program,
                        exp_res_id: ret_val.exp_res_id,
                    };
                },
                MulExp::Mulexp(mulexp, unaryexp, op) |
                MulExp::Divexp(mulexp, unaryexp, op) | 
                MulExp::Modexp(mulexp, unaryexp, op) => {
                    let ret_val1 = dfs(TreePoint::MulExp(*mulexp), shift, size);
                    let ret_val2 = dfs(TreePoint::UnaryExp(unaryexp), shift, ret_val1.size);
                    let size = ret_val2.size + 1;
                    program.push_str(&ret_val1.program);
                    program.push_str(&ret_val2.program);
                    program.push_str(&format!("%var_{} = {} %var_{}, %var_{}\n", size, op, ret_val1.exp_res_id, ret_val2.exp_res_id));

                    return RetType {
                        size: size,
                        program: program,
                        exp_res_id: size,
                    };
                },
            }
        },

        TreePoint::AddExp(node) => {
            match node {
                AddExp::Mulexp(mulexp) => {
                    let ret_val = dfs(TreePoint::MulExp(mulexp), shift, size);

                    return RetType {
                        size: ret_val.size + 1,
                        program: ret_val.program,
                        exp_res_id: ret_val.exp_res_id,
                    };
                },
                AddExp::Addexp(addexp, mulexp, op) |
                AddExp::Subexp(addexp, mulexp, op) => {
                    let ret_val1 = dfs(TreePoint::AddExp(*addexp), shift, size);
                    let ret_val2 = dfs(TreePoint::MulExp(mulexp), shift, ret_val1.size);

                    let size = ret_val2.size + 1;
                    program.push_str(&ret_val1.program);
                    program.push_str(&ret_val2.program);
                    program.push_str(&format!("%var_{} = {} %var_{}, %var_{}\n", size, op, ret_val1.exp_res_id, ret_val2.exp_res_id));

                    return RetType {
                        size: size,
                        program: program,
                        exp_res_id: size,
                    };
                },
            }
        },

        TreePoint::RelExp(node) => {
            match node {
                RelExp::Addexp(addexp) => {
                    let ret_val = dfs(TreePoint::AddExp(addexp), shift, size);

                    return RetType {
                        size: ret_val.size + 1,
                        program: ret_val.program,
                        exp_res_id: ret_val.exp_res_id,
                    };
                },
                RelExp::Ltexp(relexp, addexp, op) |
                RelExp::Gtexp(relexp, addexp, op) |
                RelExp::Geexp(relexp, addexp, op) | 
                RelExp::Leexp(relexp, addexp, op) => {
                    let ret_val1 = dfs(TreePoint::RelExp(*relexp), shift, size);
                    let ret_val2 = dfs(TreePoint::AddExp(addexp), shift, ret_val1.size);

                    let size = ret_val2.size + 1;
                    program.push_str(&ret_val1.program);
                    program.push_str(&ret_val2.program);
                    program.push_str(&format!("%var_{} = {} %var_{}, %var_{}\n", size, op, ret_val1.exp_res_id, ret_val2.exp_res_id));

                    return RetType {
                        size: size,
                        program: program,
                        exp_res_id: size,
                    };
                }
            }
        },

        TreePoint::EqExp(node) => {
            match node {
                EqExp::Relexp(relexp) => {
                    let ret_val = dfs(TreePoint::RelExp(relexp), shift, size);

                    return RetType {
                        size: ret_val.size + 1,
                        program: ret_val.program,
                        exp_res_id: ret_val.exp_res_id,
                    };
                },
                EqExp::Eqexp(eqexp, relexp, op) |
                EqExp::Neqexp(eqexp, relexp, op) => {
                    let ret_val1 = dfs(TreePoint::EqExp(*eqexp), shift, size);
                    let ret_val2 = dfs(TreePoint::RelExp(relexp), shift, ret_val1.size);

                    let size = ret_val2.size + 1;
                    program.push_str(&ret_val1.program);
                    program.push_str(&ret_val2.program);
                    program.push_str(&format!("%var_{} = {} %var_{}, %var_{}\n", size, op, ret_val1.exp_res_id, ret_val2.exp_res_id));

                    return RetType {
                        size: size,
                        program: program,
                        exp_res_id: size,
                    };
                },
            }
        },
        TreePoint::LAndExp(node) => {
            match node {
                LAndExp::Eqexp(eqexp) => {
                    let ret_val = dfs(TreePoint::EqExp(eqexp), shift, size);

                    return RetType {
                        size: ret_val.size + 1,
                        program: ret_val.program,
                        exp_res_id: ret_val.exp_res_id,
                    };
                },
                LAndExp::Andexp(landexp, eqexp) => {
                    let ret_val1 = dfs(TreePoint::LAndExp(*landexp), shift, size);
                    let ret_val2 = dfs(TreePoint::EqExp(eqexp), shift, ret_val1.size);

                    let size = ret_val2.size + 1;
                    program.push_str(&ret_val1.program);
                    program.push_str(&ret_val2.program);
                    
                    program.push_str(&format!("%var_{} = ne 0, %var_{}\n", size, ret_val1.exp_res_id));
                    program.push_str(&format!("%var_{} = ne 0, %var_{}\n", size + 1, ret_val2.exp_res_id));
                    program.push_str(&format!("%var_{} = and %var_{}, %var_{}\n", size + 2, size, size + 1));

                    return RetType {
                        size: size + 2,
                        program: program,
                        exp_res_id: size + 2,
                    };
                },
            }
        },
        TreePoint::LOrExp(node) => {
            match node {
                LOrExp::Landexp(landexp) => {
                    let ret_val = dfs(TreePoint::LAndExp(landexp), shift, size);

                    return RetType {
                        size: ret_val.size + 1,
                        program: ret_val.program,
                        exp_res_id: ret_val.exp_res_id,
                    };
                },
                LOrExp::Orexp(lorexp, landexp) => {
                    let ret_val1 = dfs(TreePoint::LOrExp(*lorexp), shift, size);
                    let ret_val2 = dfs(TreePoint::LAndExp(landexp), shift, ret_val1.size);

                    let size = ret_val2.size + 1;
                    program.push_str(&ret_val1.program);
                    program.push_str(&ret_val2.program);

                    program.push_str(&format!("%var_{} = ne 0, %var_{}\n", size, ret_val1.exp_res_id));
                    program.push_str(&format!("%var_{} = ne 0, %var_{}\n", size + 1, ret_val2.exp_res_id));
                    program.push_str(&format!("%var_{} = or %var_{}, %var_{}\n", size + 2, size, size + 1));

                    return RetType {
                        size: size + 2,
                        program: program,
                        exp_res_id: size + 2,
                    };
                },
            }
        },

        _ => RetType{size: 0, 
                    program: "wrong".to_string(),
                    exp_res_id: -1},
    }
}


pub fn generator(start: CompUnit) -> String {
    let size = 0;
    return dfs(TreePoint::CompUnit(start), "", size).program;
}