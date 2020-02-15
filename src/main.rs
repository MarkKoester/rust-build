use std::process::Command;
use std::process::Output;
use std::str;
use std::fs;

fn main() {
    let result = compile("main.cpp");
    let stderr = str::from_utf8(&result.stderr);
    println!("{}", result.status);
    println!("{:#?}", stderr);
    println!("Hello, world!");
}

fn compile(path: &str) -> Output {
    let working_directory = fs::canonicalize("test/").unwrap();

    let mut command = Command::new("gcc");
    command.arg(&path).arg("-o").arg("out/a.out").current_dir(working_directory);
    println!("{:?}", command);
    let result = command.output().expect("failed to compile");
    result
}