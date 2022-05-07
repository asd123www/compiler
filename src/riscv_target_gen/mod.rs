
use core::panic;

use koopa::front::ast::Return;
use koopa::ir::Program;
use koopa::ir::Value;
use koopa::ir::ValueKind;


trait GenerateAsm {
    fn gen(&self, /* 其他必要的参数 */) -> String;
}

// 为什么impl不行, impl trait就行呢.
impl GenerateAsm for koopa::ir::FunctionData {
    fn gen(&self) -> String {

        if self.name() != "@main" {
            return "".to_string();
        }

        let mut program = "".to_string();
        program.push_str(&format!("    .globl {}\n", &self.name()[1..self.name().len()]));
        program.push_str(&format!("{}:\n", &self.name()[1..self.name().len()]));
        for (&_bb, node) in self.layout().bbs() {
            // 一些必要的处理
            // ... 
            let data_graph = self.dfg();
            // 遍历指令列表
            for &inst in node.insts().keys() {
                let value_data = data_graph.value(inst);
                // 访问指令
                // ...
                match value_data.kind() {
                    ValueKind::Integer(_int) => {
                      // 处理 integer 指令
                      // ...
                    }
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
                                // if data.
                                // if data_graph.value(x).ty() == "i32" { // return a fixed value.
                                //     program.push_str("    ret {}\n", data_graph.value(x).);
                                // } else {

                                // }
                                println!("    {:?}:\n", data);
                            },
                            None => {
                                panic!("asd123www");
                            },
                        }
                    }
                    // 其他种类暂时遇不到
                    _ => unreachable!(),
                  }                  
            }
        }

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