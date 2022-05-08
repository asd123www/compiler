
use core::panic;

// use koopa::front::ast::Return;
use koopa::ir::Program;
use koopa::ir::TypeKind;
use koopa::ir::Value;
// use koopa::ir::Value;
use koopa::ir::ValueKind;
use koopa::ir::dfg::DataFlowGraph;
use koopa::ir::entities::ValueData;
use std::collections::HashMap;

const MACHINE_BYTE: i32 = 4; // 32-bit machine.
const TYPE_POINTER: i32 = 1;

// scope use the Value(pointer) to address, not the inherit `variable name`.
// you should accept the API instead of your own convention to code easier.


fn load2register(scope: &HashMap<Value, (i32, i32)>, pt: &Value, val: &ValueData, dst: &str) -> String {
    println!("{:?}", pt);
    let mut program = "".to_string();
    match val.kind() {
        ValueKind::Integer(var) => {
            program.push_str(&format!("    li {}, {}\n", dst, var.value()));
        },
        _ => {
            let pos = scope.get(pt).unwrap();
            assert!(pos.0 == TYPE_POINTER);
            program.push_str(&format!("    lw {}, {}(sp)\n", dst, pos.1))
        },
    }
    return program;
}

trait GenerateAsm {
    fn gen(&self, /* 其他必要的参数 */) -> String;
}

// 为什么impl不行, impl trait就行呢.
impl GenerateAsm for koopa::ir::FunctionData {
    fn gen(&self) -> String {

        let mut stack_size = 0;
        // id -> (variable type, position in stack).
        let mut scope: HashMap<Value, (i32, i32)> = HashMap::new();

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
                    ValueKind::Alloc(_val) => {
                        scope.insert(inst, (TYPE_POINTER, stack_size));
                        // scope.insert(value_data.name().clone().unwrap(), (TYPE_POINTER, stack_size));
                        stack_size += match value_data.ty().kind() {
                            TypeKind::Pointer(base) => MACHINE_BYTE,
                            _ => panic!("Wrong type in Alloc"),
                        };
                        // println!("    Inst: {:?}:\n", value_data);
                        // println!("    Alloc: {:?}:\n\n\n", val);
                    },
                    ValueKind::GlobalAlloc(globl_alloc) => {
                        println!("    GlobalAlloc: {:?}:\n", globl_alloc);
                    },
                    ValueKind::Load(load) => {
                        let src = data_graph.value(load.src());
                        let fragment = load2register(&scope, &load.src(), src, "t1");
                        program.push_str(&fragment);

                        scope.insert(inst, (TYPE_POINTER, stack_size));
                        program.push_str(&format!("    sw t1, {}(sp)\n", stack_size));
                        stack_size += MACHINE_BYTE; // only  wrong!!!.

                        // println!("    {:?}\n", inst);
                        // println!("    Load: {:?}:\n", load);
                        // panic!("I'm fucking done");
                    },
                    ValueKind::Store(store) => {
                        // # store 10, @x
                        // li t0, 10
                        // sw t0, 0(sp)                      
                        let src = data_graph.value(store.value());
                        let dst = data_graph.value(store.dest());
                        let pos = scope.get(&store.dest()).unwrap();

                        let fragment = load2register(&scope, &store.value(), src, "t1");
                        program.push_str(&fragment);
                        assert!(pos.0 == TYPE_POINTER);
                        program.push_str(&format!("    sw t1, {}(sp)\n", pos.1));
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
                    ValueKind::Branch(br) => {
                        println!("    Branch: {:?}:\n", br);
                    },
                    ValueKind::Jump(jump) => {
                        println!("    Jump: {:?}:\n", jump);
                    },
                    ValueKind::Call(func_call) => {
                        println!("    Call: {:?}:\n", func_call);
                    },
                    ValueKind::Return(val) => { // ret
                        match val.value() {
                            Some(x) => {
                                let loader = load2register(&scope, &x, data_graph.value(x), "a0");
                                program.push_str(&loader);
                                program.push_str("    ret\n");
                            },
                            None => {
                                panic!("Return a none value?");
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