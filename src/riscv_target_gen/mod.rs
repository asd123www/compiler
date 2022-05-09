
use core::panic;
use std::error::Error;

use koopa::ir::BasicBlock;
use koopa::ir::BinaryOp;
use koopa::ir::FunctionData;
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
const GLOBAL_INTEGER: i32 = 0;
const INTEGER_POINTER: i32 = 1;

// scope use the Value(pointer) to address, not the inherit `variable name`.
// you should accept the API instead of your own convention to code easier.


fn block2str(bb: &BasicBlock) -> String {
    let name = format!("{:?}", bb);
    return format!("{}{}", name[0..10].to_string(), name[11..name.len()-1].to_string());
}

fn calc_funcinstr(func: &FunctionData) -> i32 {
    let mut size = 0;
    for (&_bb, node) in func.layout().bbs() {
        size += node.insts().len() as i32;
    }
    size
}

// global variable and local var is different, you should treat them differently.
fn load2register(scope: &HashMap<Value, (i32, i32)>, pt: &Value, data_graph: &DataFlowGraph, dst: &str) -> String {
    let mut program = "".to_string();
    let is_local = scope.get(pt);

    if is_local.is_none() || is_local.unwrap().0 == INTEGER_POINTER {
        let val = data_graph.value(pt.clone());
        match val.kind() {
            ValueKind::Integer(var) => {
                program.push_str(&format!("    li {}, {}\n", dst, var.value()));
            },
            _ => {
                let pos = is_local.unwrap();
                assert!(pos.0 == INTEGER_POINTER);
                program.push_str(&format!("    lw {}, {}(sp)\n", dst, pos.1))
            },
        }
    } else {
        // there must be a global definition. else crash.
        let pos = is_local.unwrap(); 
        assert!(pos.0 == GLOBAL_INTEGER);
        // la t0, var
        // lw t0, 0(t0)
        program.push_str(&format!("    la {}, glb_var{}\n", dst, pos.1));
        program.push_str(&format!("    lw {}, 0({})\n", dst, dst)); // integer.
    }
    return program;
}

fn load2data(pt: &Value, val: &ValueData) -> String {
    // println!("{:?}", pt);
    let mut program = "".to_string();
    match val.kind() {
        ValueKind::Integer(var) => {
            return var.value().to_string();
        },
        ValueKind::ZeroInit(val) => {
            return "0".to_string();
        },
        _ => {
            println!("Load2Data {:?}", val);
            panic!("fuck off");
        },
    }
    return program;
}


struct RetValue {
    program: String,
    stack_size: i32,
}
trait GenerateAsm {
    fn gen(&self, koopa: &Program, func_data: &koopa::ir::FunctionData, scope: &mut HashMap<Value, (i32, i32)>, stack_size: i32, mx_size: i32) -> RetValue;
}

trait GenerateAsmFunc {
    fn gen(&self, koopa: &Program, scope: &mut HashMap<Value, (i32, i32)>, param_len: i32) -> String;
}


impl GenerateAsm for koopa::ir::layout::BasicBlockNode {

    fn gen(&self, koopa: &Program, func_data: &koopa::ir::FunctionData, scope: &mut HashMap<Value, (i32, i32)>, stack_size: i32, mx_size: i32) -> RetValue {
        let mut stack_size = stack_size;
        let mut program = "".to_string();

        let data_graph = func_data.dfg();

        // 遍历指令列表
        for &inst in self.insts().keys() {
            let value_data = data_graph.value(inst);
            // println!("{:?}\n", value_data);
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
                    stack_size -= match value_data.ty().kind() {
                        TypeKind::Pointer(_base) => MACHINE_BYTE,
                        _ => panic!("Wrong type in Alloc"),
                    };
                    scope.insert(inst, (INTEGER_POINTER, stack_size));
                    // println!("    Inst: {:?}:\n", value_data);
                    // println!("    Alloc: {:?}:\n\n\n", val);
                },
                ValueKind::GlobalAlloc(globl_alloc) => {
                    println!("    GlobalAlloc: {:?}:\n", globl_alloc);
                },
                ValueKind::Load(load) => {
                    // let src = data_graph.value(load.src());
                    let fragment = load2register(&scope, &load.src(), data_graph, "t1");
                    program.push_str(&fragment);

                    stack_size -= MACHINE_BYTE; // only  wrong!!!.
                    scope.insert(inst, (INTEGER_POINTER, stack_size));
                    program.push_str(&format!("    sw t1, {}(sp)\n", stack_size));
                },
                ValueKind::Store(store) => {
                    // # store 10, @x
                    // li t0, 10
                    // sw t0, 0(sp)                      
                    // let src = data_graph.value(store.value());
                    // let dst = data_graph.value(store.dest());
                    let pos = scope.get(&store.dest()).unwrap();

                    let fragment = load2register(&scope, &store.value(), data_graph, "t1");
                    program.push_str(&fragment);
                    if pos.0 == INTEGER_POINTER {
                        program.push_str(&format!("    sw t1, {}(sp)\n", pos.1));
                    } else {
                        assert!(pos.0 == GLOBAL_INTEGER);
                        program.push_str(&format!("    la t2, glb_var{}\n", pos.1));
                        program.push_str(&format!("    sw t1, 0(t2)\n"));
                    }
                },
                ValueKind::GetPtr(getptr) => {
                    println!("    GetPtr: {:?}:\n", getptr);
                },
                ValueKind::GetElemPtr(getelemptr) => {
                    println!("    GetElemPtr: {:?}:\n", getelemptr);
                },
                ValueKind::Binary(binary) => {
                    let lhs = binary.lhs();
                    let rhs = binary.rhs();
                    let fragl = load2register(&scope, &lhs, data_graph, "t1");
                    let fragr = load2register(&scope, &rhs, data_graph, "t2");
                    program.push_str(&fragl);
                    program.push_str(&fragr);

                    let op = binary.op();
                    program.push_str(
                        match op {
                            BinaryOp::Add => "    add t1, t1, t2\n",
                            BinaryOp::Sub => "    sub t1, t1, t2\n",
                            BinaryOp::NotEq => "    sub t1, t1, t2\n    snez t1, t1\n",
                            BinaryOp::Eq => "    sub t1, t1, t2\n    seqz t1, t1\n",
                            BinaryOp::Gt => "    sgt t1, t1, t2\n",
                            BinaryOp::Lt => "    slt t1, t1, t2\n",
                            BinaryOp::Ge => "    add t1, t1, 1\n    sgt t1, t1, t2\n",
                            BinaryOp::Le => "    add t2, t2, 1\n    slt t1, t1, t2\n",
                            BinaryOp::Mul => "    mul t1, t1, t2\n",
                            BinaryOp::Div => "    div t1, t1, t2\n",
                            BinaryOp::Mod => "    rem t1, t1, t2\n",
                            BinaryOp::And => "    and t1, t1, t2\n",
                            BinaryOp::Or =>  "    or t1, t1, t2\n",
                            BinaryOp::Xor => "    xor t1, t1, t2\n",
                            BinaryOp::Shl => "    sll t1, t1, t2\n",
                            BinaryOp::Shr => "    srl t1, t1, t2\n",
                            BinaryOp::Sar => "    sra t1, t1, t2\n",
                        });
                    
                    stack_size -= MACHINE_BYTE;
                    scope.insert(inst, (INTEGER_POINTER, stack_size));
                    program.push_str(&format!("    sw t1, {}(sp)\n", stack_size));

                    // println!("    Binary: {:?}:\n", binary);
                },
                ValueKind::Branch(br) => {
                    let br_true = block2str(&br.true_bb());
                    let br_false = block2str(&br.false_bb());

                    let frag = load2register(scope, &br.cond(), data_graph, "t1");
                    program.push_str(&frag);
                    // bnez t0, then
                    // j else
                    program.push_str(&format!("    bnez t1, {}\n", br_true));
                    program.push_str(&format!("    j {}\n", br_false));
                },
                ValueKind::Jump(jump) => {
                    let dst = block2str(&jump.target());
                    program.push_str(&format!("    j {}\n", dst));
                },
                ValueKind::Call(func_call) => {
                    let args = func_call.args();
                    let name = koopa.func(func_call.callee()).name();

                    // pass parameter.
                    for (i, arg) in args.iter().enumerate() {
                        let dst = {
                            if i <= 7 { // a0-a7.
                                format!("a{}", i.to_string())
                            } else {
                                format!("t1")
                            }
                        };
                        let ret_val = load2register(scope, arg, data_graph, &dst);
                        program.push_str(&ret_val);
                        if i > 7 { // spilled
                            program.push_str(&format!("    sw t1, {}(sp)\n", (i - 8) * 4));
                        }
                    }
                    // call function.
                    program.push_str(&format!("    call {}\n", &name[1..name.len()]));
                    if !value_data.ty().is_unit() {
                        // we only have `integer` return value.
                        stack_size -= 4;
                        scope.insert(inst, (INTEGER_POINTER, stack_size));
                        program.push_str(&format!("    sw a0, {}(sp)\n", stack_size));
                    }
                    // scope.insert(inst, ());
                    println!("    Function: {:?}:\n", value_data);
                    println!("    Call: {:?}:\n", func_call);
                },
                ValueKind::Return(val) => { // ret
                    match val.value() {
                        Some(x) => {
                            let loader = load2register(&scope, &x, data_graph, "a0");
                            program.push_str(&loader);
                        },
                        None => {},
                    }
                    program.push_str(&format!("    lw ra, {}(sp)\n", mx_size - 4));
                    // get the correct return address and recover the stack pointer.
                    program.push_str(&format!("    addi sp, sp, {}\n", mx_size));
                    program.push_str("    ret\n");
                },
            }
        }

        return RetValue {program, stack_size};
    }
}

// 为什么impl不行, impl trait就行呢.
impl GenerateAsmFunc for koopa::ir::FunctionData {
    fn gen(&self, koopa: &Program, scope: &mut HashMap<Value, (i32, i32)>, param_len: i32) -> String {
        if self.layout().bbs().len() == 0 { // `std` function, we don't cope with.
            return "".to_string();
        }


        let mut program = "".to_string();

        // .globl main
        program.push_str(&format!("    .text\n    .globl {}\n", &self.name()[1..self.name().len()]));
        program.push_str(&format!("{}:\n", &self.name()[1..self.name().len()]));

        // saved space for spilled parameter.
        let mut stack_size = (param_len + calc_funcinstr(self) * 4) as i32;
        stack_size += stack_size % 16;
        let origin_stack_size = stack_size;

        // save space for local variable.
        program.push_str(&format!("    addi sp, sp, -{}\n", stack_size));
        // save the `ra`, aka return address. sw ra, 0(sp)
        stack_size -= 4;
        program.push_str(&format!("    sw ra, {}(sp)\n", stack_size));


        // load parameter.
        for (i, param) in self.params().iter().enumerate() {
            if i > 7 {
                // wrong!!!!!!
                // panic!("You have to calculate it previously.");
                scope.insert(param.clone(), (INTEGER_POINTER, origin_stack_size + (4 * (i - 8) as i32)));
            } else { // pass through a0-a7
                stack_size -= 4;
                scope.insert(param.clone(), (INTEGER_POINTER, stack_size));
                program.push_str(&format!("    sw a{}, {}(sp)\n", i, stack_size));
            }
        }

        for (&bb, node) in self.layout().bbs() {
            program.push_str(&format!("\n{}:\n", block2str(&bb)));
            // remember inherit the stack_size!
            let ret_val = node.gen(&koopa, self, scope, stack_size, origin_stack_size);
            program.push_str(&ret_val.program);
            stack_size = ret_val.stack_size;
        }
        program.push_str("\n\n\n");

        // we have to replace stack_size.
        return program;
    }
}



pub fn generate (koopa_program: Program) -> String {

    koopa::ir::Type::set_ptr_size(4); // set 32-bit machine.
    
    let mut program = "".to_string();
    let f = koopa_program.borrow_values().clone();

    // id -> (variable type, position in stack).
    let mut scope: HashMap<Value, (i32, i32)> = HashMap::new();


    // global variable initialize.
    let mut glb_count = 0;
    for glb_var in koopa_program.inst_layout() {
        let data = f.get(glb_var).unwrap();
        scope.insert(*glb_var, (GLOBAL_INTEGER, glb_count));
        let name = format!("glb_var{}", glb_count); 
        program.push_str(&format!("    .data\n    .globl {}\n{}:\n", &name, &name));

        match data.kind() {
            ValueKind::GlobalAlloc(val) => {
                // must be integer now.
                let x = load2data(&val.init(), f.get(&val.init()).unwrap());
                program.push_str(&format!("    .word {}\n", x));
                println!("{:?}\n", x);
            },
            _ => panic!("Global variable initialize error.")
        }

        glb_count += 1;
        program.push_str("\n\n\n\n");
    }

    println!("{:?}\n", koopa_program.inst_layout());

    let mut param_mxlen: i32 = 0;
    for &func in koopa_program.func_layout() {
        let func_data = koopa_program.func(func);
        param_mxlen = std::cmp::max(param_mxlen, (func_data.params().len() as i32) - 8);
        let ret_val = func_data.gen(&koopa_program, &mut scope, param_mxlen);
        program.push_str(&ret_val);
    }

    program.to_string()
}