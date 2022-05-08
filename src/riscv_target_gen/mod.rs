
use core::panic;
use std::hash::Hash;

use koopa::ir::BasicBlock;
use koopa::ir::BinaryOp;
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
const INTEGER_POINTER: i32 = 1;

// scope use the Value(pointer) to address, not the inherit `variable name`.
// you should accept the API instead of your own convention to code easier.


fn block2str(bb: &BasicBlock) -> String {
    let name = format!("{:?}", bb);
    return format!("{}{}", name[0..10].to_string(), name[11..name.len()-1].to_string());
}

fn load2register(scope: &HashMap<Value, (i32, i32)>, pt: &Value, val: &ValueData, dst: &str) -> String {
    // println!("{:?}", pt);
    let mut program = "".to_string();
    match val.kind() {
        ValueKind::Integer(var) => {
            program.push_str(&format!("    li {}, {}\n", dst, var.value()));
        },
        _ => {
            let pos = scope.get(pt).unwrap();
            assert!(pos.0 == INTEGER_POINTER);
            program.push_str(&format!("    lw {}, {}(sp)\n", dst, pos.1))
        },
    }
    return program;
}


struct RetValue {
    program: String,
    stack_size: i32,
}
trait GenerateAsm {
    fn gen(&self, koopa: &Program, func_data: &koopa::ir::FunctionData, scope: &mut HashMap<Value, (i32, i32)>, stack_size: i32) -> RetValue;
}

trait GenerateAsmFunc {
    fn gen(&self, koopa: &Program) -> String;
}


impl GenerateAsm for koopa::ir::layout::BasicBlockNode {

    fn gen(&self, koopa: &Program, func_data: &koopa::ir::FunctionData, scope: &mut HashMap<Value, (i32, i32)>, stack_size: i32) -> RetValue {
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
                    scope.insert(inst, (INTEGER_POINTER, stack_size));
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

                    scope.insert(inst, (INTEGER_POINTER, stack_size));
                    program.push_str(&format!("    sw t1, {}(sp)\n", stack_size));
                    stack_size += MACHINE_BYTE; // only  wrong!!!.
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
                    assert!(pos.0 == INTEGER_POINTER);
                    program.push_str(&format!("    sw t1, {}(sp)\n", pos.1));
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
                    let fragl = load2register(&scope, &lhs, data_graph.value(lhs), "t1");
                    let fragr = load2register(&scope, &rhs, data_graph.value(rhs), "t2");
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
                    
                    scope.insert(inst, (INTEGER_POINTER, stack_size));
                    program.push_str(&format!("    sw t1, {}(sp)\n", stack_size));
                    stack_size += MACHINE_BYTE;

                    // println!("    Binary: {:?}:\n", binary);
                },
                ValueKind::Branch(br) => {
                    let br_true = block2str(&br.true_bb());
                    let br_false = block2str(&br.false_bb());

                    let frag = load2register(scope, &br.cond(), data_graph.value(br.cond()), "t1");
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

                    // 
                    program.push_str(&format!("    call {}\n", &name[1..name.len()]));
                    println!("    Function: {:?}:\n", name);
                    println!("    Call: {:?}:\n", func_call);
                },
                ValueKind::Return(val) => { // ret
                    match val.value() {
                        Some(x) => {
                            let loader = load2register(&scope, &x, data_graph.value(x), "a0");
                            program.push_str(&loader);
                            // get the correct return address and recover the stack pointer.
                            program.push_str("    lw ra, <replace_with_return_address>(sp)\n");
                            program.push_str("    addi sp, sp, <replace_with_stack_size>\n");
                            program.push_str("    ret\n");
                        },
                        None => {
                            panic!("Return a none value?");
                        },
                    }
                },
            }
        }

        return RetValue {program, stack_size};
    }
}

// 为什么impl不行, impl trait就行呢.
impl GenerateAsmFunc for koopa::ir::FunctionData {
    fn gen(&self, koopa: &Program) -> String {

        let mut stack_size = 8;
        let mut program = "".to_string();
        // id -> (variable type, position in stack).
        let mut scope: HashMap<Value, (i32, i32)> = HashMap::new();

        // .globl main
        program.push_str(&format!("    .globl {}\n", &self.name()[1..self.name().len()]));
        program.push_str(&format!("{}:\n", &self.name()[1..self.name().len()]));

        // save space for local variable.
        program.push_str("    addi sp, sp, -<replace_with_stack_size>\n");
        // save the `ra`, aka return address. sw ra, 0(sp)
        program.push_str("    sw ra, <replace_with_return_address>(sp)\n");

        if self.layout().bbs().len() == 0 { // `std` function, we don't cope with.
            return "".to_string();
        }

        for (&bb, node) in self.layout().bbs() {
            program.push_str(&format!("\n{}:\n", block2str(&bb)));
            // remember inherit the stack_size!
            let ret_val = node.gen(&koopa, self, &mut scope, stack_size);
            program.push_str(&ret_val.program);
            stack_size = ret_val.stack_size;
        }
        stack_size += stack_size % 16;
        program.push_str("\n\n\n");

        let program = str::replace(&program, "<replace_with_return_address>", &stack_size.to_string());
        // we have to replace stack_size.
        return str::replace(&program, "<replace_with_stack_size>", &(stack_size + 4).to_string());
    }
}



pub fn generate (koopa_program: Program) -> String {
    let mut program = "".to_string();

    for &func in koopa_program.func_layout() {
        let func_data = koopa_program.func(func);
        let ret_val = func_data.gen(&koopa_program);
        program.push_str(&ret_val);
    }

    program.to_string()
}