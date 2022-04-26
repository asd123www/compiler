mod ast;
mod koopa_ir_gen;

use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs;
use std::fs::read_to_string;
use std::fs::File;
// use std::io::prelude::*;
use std::io::Result;
use std::io::Write;


// 引用 lalrpop 生成的解析器
// 因为我们刚刚创建了 sysy.lalrpop, 所以模块名是 sysy
lalrpop_mod!(sysy);

fn main() -> Result<()> {
    // 解析命令行参数
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    println!("mode is {}.",mode);
    println!("input is {}.", input);
    println!("output is {}.", output);

    // 读取输入文件
    let input = read_to_string(input)?;

    // 调用生成的parser: sysy, 指定start non-terminal: CompUnit(结尾默认加入Parser).
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();

    // 输出解析得到的 AST
    let koopa_program = koopa_ir_gen::Generator(ast);

    let mut file = File::create(output)?;
    file.write_all(koopa_program.as_bytes())?;

    // File::create(&output)?;
    // fs::write(output, koopa_program).expect("Unable to write file");

    // println!("{:#?}", ast);
    Ok(())
}

// cargo run -- -koopa hello.c -o hello.koopa
// autotest -koopa -s lv1 /root/compiler