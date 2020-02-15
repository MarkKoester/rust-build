use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Output;
use std::str;

struct Rule {
    input: PathBuf,
    output: PathBuf,
}

fn main() {
    let files = fs::read_dir("src").expect("src directory does not exist");

    for file in files {
        let file = file.expect("file does not exist");
        let rule = make_rule(&file.path());
        let result = compile(&rule);
        let stderr = str::from_utf8(&result.stderr);
        println!("{}", result.status);
        println!("{:#?}", stderr);
        println!("Hello, world!");
    }
}

fn compile(rule: &Rule) -> Output {
    let mut command = Command::new("g++");
    command.arg(&rule.input).arg("-o").arg(&rule.output);
    println!("{:?}", command);
    let result = command.output().expect("failed to compile");
    result
}

fn make_rule(file: &Path) -> Rule {
    let source_directory = fs::canonicalize("src/").expect("directory src/ does not exist");
    let out_directory = fs::canonicalize("out/").expect("directory out/ does not exist");

    let file_name = file.file_name().expect("file does not exist");

    let mut output_file_name = PathBuf::from(&file_name);
    output_file_name.set_extension("o");

    let mut input = PathBuf::from(&source_directory);
    input.push(&file_name);

    let mut output = PathBuf::from(&out_directory);
    output.push(&output_file_name);

    Rule { input, output }
}
