
use core::panic;

// use koopa::front::ast::Return;
use koopa::ir::Program;
// use koopa::ir::Value;
use koopa::ir::ValueKind;
use std::collections::HashMap;


trait GenerateAsm {
    fn gen(&self, /* 其他必要的参数 */) -> String;
}

// 为什么impl不行, impl trait就行呢.
impl GenerateAsm for koopa::ir::FunctionData {
    fn gen(&self) -> String {

        // id -> position in stack.
        let mut scope: HashMap<i32, i32> = HashMap::new();

        // we don't know how to deal with these.
        if self.name() != "@main" {
            return "".to_string();
        }

        let mut program = "".to_string();
        program.push_str(&format!("    .globl {}\n", &self.name()[1..self.name().len()]));
        program.push_str(&format!("{}:\n", &self.name()[1..self.name().len()]));
        for (&_bb, node) in self.layout().bbs() {
            let data_graph = self.dfg();
            // 遍历指令列表
            for &inst in node.insts().keys() {
                let value_data = data_graph.value(inst);
                // 访问指令
                match value_data.kind() {
                    ValueKind::Integer(val) => {
                        println!("    Integer: {:?}:\n", val);
                    },
                    ValueKind::ZeroInit(val) => {
                        println!("    ZeroInit: {:?}:\n", val);
                    },
                    ValueKind::Undef(val) => {
                        println!("    Undef: {:?}:\n", val);
                    },
                    ValueKind::Aggregate(aggre) => {
                        println!("    Aggregate: {:?}:\n", aggre);
                    },
                    ValueKind::FuncArgRef(func_argref) => {
                        println!("    FuncArgRef: {:?}:\n", func_argref);
                    },
                    ValueKind::BlockArgRef(block_argref) => {
                        println!("    BlockArgRef: {:?}:\n", block_argref);
                    },
                    ValueKind::Alloc(val) => {
                        
                        println!("    Alloc: {:?}:\n", val);
                    },
                    ValueKind::GlobalAlloc(globl_alloc) => {
                        println!("    GlobalAlloc: {:?}:\n", globl_alloc);
                    },
                    ValueKind::Load(load) => {
                        println!("    Load: {:?}:\n", load);
                    },
                    ValueKind::Store(store) => {
                        println!("    Store: {:?}:\n", store);
                    },
                    ValueKind::GetPtr(getptr) => {
                        println!("    GetPtr: {:?}:\n", getptr);
                    },
                    ValueKind::GetElemPtr(getelemptr) => {
                        println!("    GetElemPtr: {:?}:\n", getelemptr);
                    },
                    ValueKind::Binary(binary) => {
                        println!("    Binary: {:?}:\n", binary);
                    },
                    // Conditional branch.
                    ValueKind::Branch(br) => {
                        println!("    Branch: {:?}:\n", br);
                    },
                    // Unconditional jump.
                    ValueKind::Jump(jump) => {
                        println!("    Jump: {:?}:\n", jump);
                    },
                    // Function call.
                    ValueKind::Call(func_call) => {
                        println!("    Call: {:?}:\n", func_call);
                    },
                    ValueKind::Return(val) => { // ret
                        match val.value() {
                            Some(x) => {
                                let data = data_graph.value(x).kind();
                                match data {
                                    ValueKind::Integer(var) => {
                                        program.push_str(&format!("    li a0, {}\n", var.value()));
                                    },
                                    _ => {
                                        panic!("Return a non-integer type.");
                                    },
                                }
                                program.push_str("    ret\n");
                                println!("    {:?}:\n", data);
                            },
                            None => {
                                panic!("asd123www");
                            },
                        }
                    }
                }
            }
        }
        program.push_str("\n\n\n");
        
        return program;
    }
}



pub fn generate (koopa_program: Program) -> String {
    let mut program = "".to_string();

    for &func in koopa_program.func_layout() {
        let func_data = koopa_program.func(func);
        let ret_val = func_data.gen();
        program.push_str(&ret_val);
    }

    program.to_string()
}