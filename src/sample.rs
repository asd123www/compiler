use std::io; // prelude
use rand::Rng; // trait
use std::cmp::Ordering;

const MAX_POINTS: u32 = 100_000;


enum Coin{
    Penny,
    Nickel,
    Dime,
    Quarter(i32),
}
/*
match coin {

    Coin::Quarter(state) // 取出绑定值.
}
*/

fn test_life_time() {
    let r ;
    let x = 5;

    r = &x;

    println!("r: {}", r);
}


fn another_function(x: i32) {
    fn wow_cool() {
        fn hahaha(yy: i32) -> i32 {
            return yy + 5;
        }
        println!("Return value: {}", hahaha(5));
        println!("\nYou can write subroutine!");       
    }

    wow_cool();
    println!("The value of x is: {}.\n", x);



    let number = {
        if true {5} else {6}
    };
    println!("{}", number);
}


// vector + enum, 存储多种对象.

fn main() {

    test_life_time();
    


    println!("Hello, world!");

    println!("The value is: {}", 3i32);


    let s =format!("{}-{}-{}", "asd".to_string(), "123".to_string(), "www".to_string());
    another_function(5);

    panic!("crash and abort.");

    let aa = [0; 10]; // 用0初始化长度为100的数组.

    for element in aa.iter() {
        println!("{}", element);
    }
    for number in (0..10).rev() {
        println!("{}", number);
    }

    let _tup: (i32, f64, bool) = (123456, 234.35, false);
    let (a, b, c) = _tup;
    println!("The value: {} {} {}", _tup.0, b, c);

    let secrete_number:i32 = rand::thread_rng().gen_range(1..101);
    println!("The secret number isn't: {}.", MAX_POINTS);

    loop {
        let mut guess = String::new();
        io::stdin().read_line(&mut guess).expect("can't read line.");
        // println!("guess: {}", guess);
        // shadow the old variable.
        let guess:i32 = match guess.trim().parse() {
            Ok(result) => result,
            Err(_) => continue
        };

        match guess.cmp(&secrete_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
