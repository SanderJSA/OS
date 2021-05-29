//! This module implements a tty

/*
/// Start and run tty
pub fn run_tty() {
    // Set up shell
    println!(
        "     .~~~~`\\~~\\
     ;       ~~ \\
     |           ;
 ,--------,______|---.
/          \\-----`    \\
`.__________`-_______-'
           {}\n",
        1 as char
    );

    println!("Howdy, welcome to RustOS");

    // Run shell
    loop {
        print!("> ");
        let input = readline();

        match input.split_whitespace().nth(0).unwrap() {
            "poweroff" => exit_qemu(QemuExitCode::Success),
            "ls" => file_system::ls(),
            "touch" => {
                let data: [u8; 0] = [];
                file_system::add_file(input.split_whitespace().nth(1).unwrap(), &data, 0)
            }
            "help" => println!(
                "RustOS tty v1.0\n\
                ls         list files in current directory\n\
                touch FILE Update the access and modification times of each FILE to the current time.\n\
                poweroff   Power off the machine\n\
                "
            ),
            _ => print!("Unknown command: {}", input),
        }
    }
}
*/

mod env;
mod reader;
mod types;

use crate::alloc::string::*;
use crate::alloc::vec::Vec;
use crate::driver::ps2_keyboard::readline;
use crate::{exit_qemu, file_system, print, println, QemuExitCode};
use env::Env;
use reader::Reader;
use types::*;

fn read() -> MalType {
    print!("root> ");
    let line = readline();
    Reader::new(&line).read_form()
}

fn eval_ast(ast: MalType, env: &mut Env) -> MalType {
    match ast {
        MalType::Symbol(sym) => env.get(&sym).expect("Symbol not found in env"),
        MalType::List(list) => MalType::List(list.into_iter().map(|val| eval(val, env)).collect()),
        _ => ast,
    }
}

fn eval(ast: MalType, env: &mut Env) -> MalType {
    match ast {
        MalType::List(ref list) => match list.as_slice() {
            [] => eval_ast(ast, env),
            [MalType::Symbol(sym), MalType::Symbol(key), value] if sym == "def!" => {
                let value = eval(value.clone(), env);
                env.set(key.to_string(), value.clone());
                value
            }
            [MalType::Symbol(sym), MalType::List(bindings), exp] if sym == "let*" => {
                eval_let(bindings, exp, env)
            }
            //[MalType::Symbol(sym), ..] if sym == "do" => eval_let(bindings, exp, env),
            _ => {
                if let MalType::List(list) = eval_ast(ast, env) {
                    let mut values = list.into_iter();
                    if let MalType::Func(func) = values.next().unwrap() {
                        return func(values.next().unwrap(), values.next().unwrap());
                    }
                }
                unreachable!();
            }
        },

        _ => eval_ast(ast, env),
    }
}

fn eval_let(bindings: &Vec<MalType>, exp: &MalType, env: &mut Env) -> MalType {
    let mut new_env = Env::new(Some(env));
    if let [MalType::Symbol(key), value] = bindings.as_slice() {
        let value = eval(value.clone(), &mut new_env);
        new_env.set(key.to_string(), value);
    } else {
        for pair in bindings {
            if let MalType::List(pair) = pair {
                if let [MalType::Symbol(key), value] = pair.as_slice() {
                    let value = eval(value.clone(), &mut new_env);
                    new_env.set(key.to_string(), value);
                }
            }
        }
    }
    eval(exp.clone(), &mut new_env)
}

fn print(ast: MalType) {
    println!("{}", ast);
}

pub fn run_tty() {
    // Greet message
    println!(
        "     .~~~~`\\~~\\
     ;       ~~ \\
     |           ;
 ,--------,______|---.
/          \\-----`    \\
`.__________`-_______-'
           {}\n",
        1 as char
    );

    println!("Howdy, welcome to RustOS");

    // Initialize environment
    let mut env = Env::new(None);
    env.set(
        String::from("+"),
        MalType::Func(|a: MalType, b: MalType| match (a, b) {
            (MalType::Number(a), MalType::Number(b)) => MalType::Number(a + b),
            _ => MalType::Number(0),
        }),
    );
    env.set(
        String::from("-"),
        MalType::Func(|a: MalType, b: MalType| match (a, b) {
            (MalType::Number(a), MalType::Number(b)) => MalType::Number(a - b),
            _ => MalType::Number(0),
        }),
    );
    env.set(
        String::from("*"),
        MalType::Func(|a: MalType, b: MalType| match (a, b) {
            (MalType::Number(a), MalType::Number(b)) => MalType::Number(a * b),
            _ => MalType::Number(0),
        }),
    );
    env.set(
        String::from("/"),
        MalType::Func(|a: MalType, b: MalType| match (a, b) {
            (MalType::Number(a), MalType::Number(b)) => MalType::Number(a / b),
            _ => MalType::Number(0),
        }),
    );

    loop {
        let input = read();
        let ast = eval(input, &mut env);
        print(ast);
    }
}
