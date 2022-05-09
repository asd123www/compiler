
use core::panic;

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
const ARRAY_POINTER: i32 = 2;
const GLOBAL_ARRAY: i32 = 3;
const REAL_POINTER: i32 = 4; // 我们必须明确区分指针变量和伪指针变量, 因为前者存储的数据是pointer, 后者一定是data.

const SIGN_BITS: i32 = 7;
// scope use the Value(pointer) to address, not the inherit `variable name`.
// you should accept the API instead of your own convention to code easier.


fn block2str(bb: &BasicBlock) -> String {
    let name = format!("{:?}", bb);
    return format!("{}{}", name[0..10].to_string(), name[11..name.len()-1].to_string());
}

fn calc_funcinstr(func: &FunctionData) -> i32 {
    let mut size = 0;
    for (&_bb, node) in func.layout().bbs() {
        // size += node.insts().len() as i32; // wrong!

        for &inst in node.insts().keys() {
            let value_data = func.dfg().value(inst);

            size += match value_data.ty().kind() {
                TypeKind::Pointer(_base) => _base.size() as i32,
                _ => {
                    println!("{:?}\n", value_data);
                    4
                },
            };
        }
    }
    size
}

// corner case. overflow.
fn riscv_addi(dst: &str, src: &str, imme: i32) -> String {
    assert!(src != "t1");
    let mut program = "".to_string();
    if imme < -2048 || imme > 2047 {
        program.push_str(&format!("    li t1, {}\n", imme));
        program.push_str(&format!("    add {}, {}, t1\n", dst, src));
    } else {
        program.push_str(&format!("    addi {}, {}, {}\n", dst, src, &imme));
    }
    return program;
}
fn riscv_lw(dst: &str, src: &str, imme: i32) -> String {
    assert!(src != "t3");
    let mut program = "".to_string();
    if imme < -2048 || imme > 2047 {
        program.push_str(&format!("    li t3, {}\n", imme));
        program.push_str(&format!("    add t3, {}, t3\n", src));
        program.push_str(&format!("    lw {}, 0(t3)\n", dst));
    } else {
        program.push_str(&format!("    lw {}, {}({})\n", dst, imme, src));
    }
    return program;
}
fn riscv_sw(dst: &str, src: &str, imme: i32) -> String {
    assert!(src != "t3");
    let mut program = "".to_string();
    if imme < -2048 || imme > 2047 {
        program.push_str(&format!("    li t3, {}\n", imme));
        program.push_str(&format!("    add t3, {}, t3\n", src)); // 你tm写sp？
        program.push_str(&format!("    sw {}, 0(t3)\n", dst));
    } else {
        program.push_str(&format!("    sw {}, {}({})\n", dst, imme, src));
    }
    return program;
}

// idx must be store in `t1`.
fn loadpointer2register(scope: &HashMap<Value, (i32, i32)>, pt: &Value, idx: &str, dst: &str, type_size: i32) -> String {
    let mut program = "".to_string();
    let var = scope.get(pt).unwrap();
    assert!(idx == "t1");
    if (var.0 & SIGN_BITS) == ARRAY_POINTER || var.0 == INTEGER_POINTER {
        program.push_str(&riscv_addi("t0", "sp", var.1)); // load base.
        // program.push_str(&format!("    addi t0, sp, {}\n", var.1)); 
        // program.push_str(&format!("    li t2, {}\n    mul t1, t1, t2\n", type_size));
        // program.push_str(&format!("    add {}, t0, t1\n", dst));
    } else if (var.0 & SIGN_BITS) == GLOBAL_ARRAY {
        program.push_str(&format!("    la t0, glb_var{}\n", var.1)); // load base.
    } else {
        // println!("type: {}\n", var.0);
        assert!((var.0 & SIGN_BITS) == REAL_POINTER);
        // wrong!!!, 我感觉没什么区别在riscv中. 
        program.push_str(&riscv_addi("t0", "sp", var.1)); // load base.
        program.push_str(&riscv_lw("t0", "t0", 0)); // get the pointer value to dst.
    }
    // idx * size
    program.push_str(&format!("    li t2, {}\n    mul t1, t1, t2\n", type_size));
    program.push_str(&format!("    add {}, t0, t1\n", dst));
    return program;
}

// global variable and local var is different, you should treat them differently.
fn load2register(scope: &HashMap<Value, (i32, i32)>, pt: &Value, data_graph: &DataFlowGraph, dst: &str, not_pointer: bool) -> String {
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
                program.push_str(&riscv_lw(dst, "sp", pos.1));
                // program.push_str(&format!("    lw {}, {}(sp)\n", dst, pos.1))
            },
        }
    } else  {
        // there must be a global definition. else crash.
        let pos = is_local.unwrap();
        // println!("    Query: {:?}:\n\n\n", pos);
        if pos.0 == GLOBAL_INTEGER {
            // la t0, var
            // lw t0, 0(t0)
            program.push_str(&format!("    la {}, glb_var{}\n", dst, pos.1));
            program.push_str(&riscv_lw(dst, dst, 0));// integer.
        } else { 
            assert!(pos.0 == REAL_POINTER);

            program.push_str(&riscv_addi(dst, "sp", pos.1)); // load base.
            program.push_str(&riscv_lw(dst, dst, 0)); // get the pointer value.

            // Fixed bug: in function call, we may want a pointer.
            if not_pointer {
                program.push_str(&riscv_lw(dst, dst, 0)); // get the real value.
            }
        }
    }
    return program;
}

// multi-layer structure, dfs put to flat one.
fn aggre_flatmap_datagraph(aggre: &ValueData, data_graph: &DataFlowGraph) -> Vec<i32> {
    let mut res = Vec::new();
    match aggre.kind() {
        ValueKind::Integer(var) => {
            res.push(var.value());
        },
        ValueKind::Aggregate(var) => {
            for x in var.elems() {
                let mut ret_val = aggre_flatmap_datagraph(data_graph.value(x.clone()), data_graph);
                res.append(&mut ret_val);
            }
        },
        _ => panic!("Fuck off"),
    }
    return res;
}
// multi-layer structure, dfs put to flat one.
fn aggre_flatmap_hashmap(aggre: &ValueData, map: &HashMap<Value, ValueData>) -> Vec<i32> {
    let mut res = Vec::new();
    match aggre.kind() {
        ValueKind::Integer(var) => {
            res.push(var.value());
        },
        ValueKind::Aggregate(var) => {
            for x in var.elems() {
                let mut ret_val = aggre_flatmap_hashmap(map.get(&x).unwrap(), map);
                res.append(&mut ret_val);
            }
        },
        _ => panic!("Fuck off"),
    }
    return res;
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
            // println!("asd123www: {:?}\n", value_data);
            let type_size = match value_data.ty().kind() {
                TypeKind::Pointer(_base) => _base.size() as i32,
                _ => 0, // I don't care others, cause we'll not use it. 
            };
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
                    // println!("    Alloc: {:?}\n{:?}\n\n\n", inst, value_data);
                    stack_size -= type_size;
                    if type_size == MACHINE_BYTE { // equal 4 -> int.
                        scope.insert(inst, (INTEGER_POINTER, stack_size));
                    } else {
                        scope.insert(inst, (ARRAY_POINTER + type_size * 2, stack_size));
                    }
                },
                ValueKind::GlobalAlloc(globl_alloc) => {
                    println!("    GlobalAlloc: {:?}:\n", globl_alloc);
                },
                ValueKind::Load(load) => {
                    // let src = data_graph.value(load.src());
                    let fragment = load2register(&scope, &load.src(), data_graph, "t1", true);
                    program.push_str(&fragment);

                    stack_size -= MACHINE_BYTE; // only  wrong!!!.
                    scope.insert(inst, (INTEGER_POINTER, stack_size));
                    program.push_str(&riscv_sw("t1", "sp", stack_size));
                    // program.push_str(&format!("    sw t1, {}(sp)\n", stack_size));
                },
                ValueKind::Store(store) => {
                    // # store 10, @x
                    // li t0, 10
                    // sw t0, 0(sp)
                    // let src = data_graph.value(store.value());
                    // let dst = ;
                    let pos = scope.get(&store.dest()).unwrap();

                    let x = data_graph.value(store.dest()).ty();

                    if pos.0 == INTEGER_POINTER  {
                        let fragment = load2register(&scope, &store.value(), data_graph, "t1", true);
                        program.push_str(&fragment);
                        program.push_str(&riscv_sw("t1", "sp", pos.1));
                        // program.push_str(&format!("    sw t1, {}(sp)\n", pos.1));
                    } else if pos.0 == GLOBAL_INTEGER {
                        let fragment = load2register(&scope, &store.value(), data_graph, "t1", true);
                        program.push_str(&fragment);
                        program.push_str(&format!("    la t2, glb_var{}\n", pos.1));
                        program.push_str(&riscv_sw("t1", "t2", 0));
                        // program.push_str(&format!("    sw t1, 0(t2)\n"));
                    } else if pos.0 == REAL_POINTER {
                        let fragment = load2register(&scope, &store.value(), data_graph, "t1", true);
                        program.push_str(&fragment);
                        program.push_str(&riscv_lw("t2", "sp", pos.1)); // get pointer value.
                        // program.push_str(&format!("    lw t2, {}(sp)\n", pos.1));
                        program.push_str(&riscv_sw("t1", "t2", 0));
                        // program.push_str(&format!("    sw t1, 0(t2)\n"));
                    } else {
                        assert!((pos.0 & SIGN_BITS) == ARRAY_POINTER);
                        // let fragment = load2register(&scope, &store.value(), data_graph, "t1");
                        let size = pos.0 / (SIGN_BITS + 1);
                        let aggre = data_graph.value(store.value());

                        match aggre.kind() {
                            ValueKind::ZeroInit(_val) => {
                                for i in 0..size {
                                    program.push_str(&format!("    li t1, {}\n", i));
                                    let ret_val = loadpointer2register(scope, &store.dest(), "t1", "t0", 4);
                                    program.push_str(&ret_val);
                                    
                                    program.push_str(&riscv_sw("x0", "t0", 0));
                                    // program.push_str("    sw x0, 0(t0)\n"); // x0 = 0.
                                }
                            },
                            ValueKind::Aggregate(vals) => {
                                // println!("asd123www: {:?}\n", data_graph.value(vals.elems()[0]));
                                let array = aggre_flatmap_datagraph(aggre, data_graph);
                                // println!("asd123www: {:?}\n", &array);
                                println!(" size:{},   array:{:?}\n ",size, array);
                                assert!(size == (array.len() as i32));
                                for (i, ele) in array.iter().enumerate() {
                                    // get array position to `t0`.
                                    program.push_str(&format!("    li t1, {}\n", i));
                                    let ret_val = loadpointer2register(scope, &store.dest(), "t1", "t0", 4);
                                    program.push_str(&ret_val);

                                    // load data to register
                                    program.push_str(&format!("    li t1, {}\n", ele));
                                    program.push_str(&riscv_sw("t1", "t0", 0));
                                    // program.push_str("    sw t1, 0(t0)\n");
                                }
                            },
                            _ => panic!("Store instruction wrong!\n")
                        }
                    }
                },
                // 吃了这么多屎, 我终于感觉到在最后一步, 结构化的IR表示中内嵌了当前指针的类型, 让我写这两个指令无比轻松.
                // 如果没有强类型, 这一步会很麻烦.
                ValueKind::GetPtr(getptr) => {
                    // wrong!!!
                    // go to subtype.
                    let src = getptr.src();
                    let idx = getptr.index();

                    let idx_str = load2register(scope, &idx, data_graph, "t1", true);
                    program.push_str(&idx_str);
                    let pt_str = loadpointer2register(scope, &src, "t1", "t0", type_size);
                    program.push_str(&pt_str);

                    stack_size -= MACHINE_BYTE;

                    program.push_str(&riscv_sw("t0", "sp", stack_size));
                    // program.push_str(&format!("    sw t0, {}(sp)\n", stack_size));
                    scope.insert(inst, (REAL_POINTER, stack_size)); // after one shift, you are an ordinary array.
                    // println!("    GetPtr: {:?}:\n", value_data);
                },
                ValueKind::GetElemPtr(getelemptr) => {
                    // go to subtype.
                    let src = getelemptr.src();
                    let idx = getelemptr.index();

                    let idx_str = load2register(scope, &idx, data_graph, "t1", true);
                    program.push_str(&idx_str);
                    let pt_str = loadpointer2register(scope, &src, "t1", "t0", type_size);
                    program.push_str(&pt_str);

                    stack_size -= MACHINE_BYTE;

                    // the fucking wrong code.
                    // program.push_str(&format!("    lw t0, 0(t0)\n    sw t0, {}(sp)\n", stack_size));
                    // store the pointer.

                    program.push_str(&riscv_sw("t0", "sp", stack_size));
                    // program.push_str(&format!("    sw t0, {}(sp)\n", stack_size));
                    scope.insert(inst, (REAL_POINTER, stack_size)); // after one shift, you are an ordinary array.
                    // wrong!!!
                    // println!("    src: {:?}\n", data_graph.value(src));
                    // println!("    idx: {:?}\n", data_graph.value(idx));
                    // println!("    GetElemPtr: {:?}\n", value_data);
                    // panic!("not implemented");
                },
                ValueKind::Binary(binary) => {
                    let lhs = binary.lhs();
                    let rhs = binary.rhs();
                    let fragl = load2register(&scope, &lhs, data_graph, "t1", true);
                    let fragr = load2register(&scope, &rhs, data_graph, "t2", true);
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

                    program.push_str(&riscv_sw("t1", "sp", stack_size));
                    // program.push_str(&format!("    sw t1, {}(sp)\n", stack_size));

                    // println!("    Binary: {:?}:\n", binary);
                },
                ValueKind::Branch(br) => {
                    let br_true = block2str(&br.true_bb());
                    let br_false = block2str(&br.false_bb());

                    let frag = load2register(scope, &br.cond(), data_graph, "t1", true);
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
                        // if  { // we want to pass a `i32`.
                        let ret_val = load2register(scope, arg, data_graph, &dst, data_graph.value(*arg).ty().is_i32());
                        program.push_str(&ret_val);
                        if i > 7 { // spilled
                            program.push_str(&riscv_sw("t1", "sp", ((i - 8) * 4) as i32));
                            // program.push_str(&format!("    sw t1, {}(sp)\n", (i - 8) * 4));
                        }
                    }
                    // call function.
                    program.push_str(&format!("    call {}\n", &name[1..name.len()]));
                    if !value_data.ty().is_unit() {
                        // we only have `integer` return value.
                        stack_size -= 4;
                        scope.insert(inst, (INTEGER_POINTER, stack_size));

                        program.push_str(&riscv_sw("a0", "sp", stack_size));
                        // program.push_str(&format!("    sw a0, {}(sp)\n", stack_size));
                    }
                    // scope.insert(inst, ());
                    // println!("    Function: {:?}:\n", value_data);
                    // println!("    Call: {:?}:\n", func_call);
                },
                ValueKind::Return(val) => { // ret
                    match val.value() {
                        Some(x) => {
                            let loader = load2register(&scope, &x, data_graph, "a0", true);
                            program.push_str(&loader);
                        },
                        None => {},
                    }
                    
                    program.push_str(&riscv_lw("ra", "sp", mx_size - 4));
                    // program.push_str(&format!("    lw ra, {}(sp)\n", mx_size - 4));
                    // get the correct return address and recover the stack pointer.
                    program.push_str(&riscv_addi("sp", "sp", mx_size));
                    // program.push_str(&format!("    addi sp, sp, {}\n", mx_size));
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
        let mut stack_size = param_len * 4 + calc_funcinstr(self);
        stack_size += stack_size % 16;
        let origin_stack_size = stack_size;

        // save space for local variable.
        program.push_str(&riscv_addi("sp", "sp", -stack_size));
        // program.push_str(&format!("    addi sp, sp, -{}\n", stack_size));
        // save the `ra`, aka return address. sw ra, 0(sp)
        stack_size -= 4;
        println!("stack_size: {}\n", stack_size);
        program.push_str(&riscv_sw("ra", "sp", stack_size));
        // program.push_str(&format!("    sw ra, {}(sp)\n", stack_size));


        // load parameter.
        // must discriminate `pointer` and `i32`.
        for (i, param) in self.params().iter().enumerate() {
            let param_type = {
                if self.dfg().value(*param).ty().is_i32() {
                    INTEGER_POINTER
                } else {
                    REAL_POINTER
                }
            };
            if i > 7 {
                scope.insert(param.clone(), (param_type, origin_stack_size + (4 * (i - 8) as i32)));
            } else { // pass through a0-a7
                stack_size -= 4;
                scope.insert(param.clone(), (param_type, stack_size));
                program.push_str(&riscv_sw(&format!("a{}", i), "sp", stack_size));
                // program.push_str(&format!("    sw a{}, {}(sp)\n", i, stack_size));
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
    // variable type is small, so high bits compress the array length.
    let mut scope: HashMap<Value, (i32, i32)> = HashMap::new();


    // global variable initialize.
    let mut glb_count = 0;
    for glb_var in koopa_program.inst_layout() {
        let data = f.get(glb_var).unwrap();
        let name = format!("glb_var{}", glb_count); 
        program.push_str(&format!("    .data\n    .globl {}\n{}:\n", &name, &name));

        match data.kind() {
            ValueKind::GlobalAlloc(val) => {
                // must be integer now.
                let vals = f.get(&val.init()).unwrap();
                let type_size = vals.ty().size();

                // println!("Value: {:?}\n", vals);
                match vals.kind() {
                    ValueKind::Integer(var) => {
                        program.push_str(&format!("    .word {}\n", var.value()));
                    },
                    ValueKind::ZeroInit(_val) => {
                        program.push_str(&format!("    .zero {}\n", type_size));
                    },
                    ValueKind::Aggregate(aggre) => {
                        let array = aggre_flatmap_hashmap(vals, &f);
                        for ele in array {
                            program.push_str(&format!("    .word {}\n", &ele));
                        }
                    },
                    _ => panic!("Global variable initialize error."),
                }

                if type_size == 4 {
                    scope.insert(*glb_var, (GLOBAL_INTEGER, glb_count));
                } else {
                    scope.insert(*glb_var, (GLOBAL_ARRAY, glb_count));
                }
            },
            _ => panic!("Global variable initialize error.")
        }

        glb_count += 1;
        program.push_str("\n\n\n\n");
    }

    // println!("asd123www: {:?}\n", koopa_program.inst_layout());

    let mut param_mxlen: i32 = 0;
    for &func in koopa_program.func_layout() {
        let func_data = koopa_program.func(func);
        param_mxlen = std::cmp::max(param_mxlen, (func_data.params().len() as i32) - 8);
        let ret_val = func_data.gen(&koopa_program, &mut scope, param_mxlen);
        program.push_str(&ret_val);
    }

    program.to_string()
}