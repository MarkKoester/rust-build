use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Output;
use std::str;

struct Rule {
    input: PathBuf,
    dependencies: Vec<PathBuf>,
    output: PathBuf,
}

struct SourceTask {
    sources: Vec<Rule>,
}

struct LinkTask {
    inputs: Vec<PathBuf>,
    output: PathBuf,
}

fn main() {
    let task = source_task();
    compile_sources(&task, true);
    let link_task = link_task();
    link(&link_task);
}

fn source_task() -> SourceTask {
    let files = fs::read_dir("src").expect("src directory does not exist");
    let rules = files
        .map(|file| file.unwrap())
        .filter(|file| file.path().extension().expect("file missing extension") == "cpp")
        .map(|file| make_rule(&file.path()))
        .collect();

    SourceTask { sources: rules }
}

fn link_task() -> LinkTask {
    let files = fs::read_dir("out/").expect("out directory does not exist");
    let objects: Vec<PathBuf> = files
        .map(|file| file.unwrap())
        .filter(|file| file.path().extension().expect("file missing extension") == "o")
        .map(|file| file.path())
        .collect();

    LinkTask {
        inputs: objects,
        output: PathBuf::from("out/target"),
    }
}

fn link(task: &LinkTask) {
    let mut command = Command::new("g++");
    task.inputs.iter().for_each(|file| {
        command.arg(&file);
    });
    command.arg("-o").arg(&task.output);

    println!("{:?}", command);
    let result = command.output().expect("Link failed");
    let stderr = str::from_utf8(&result.stderr);

    println!("{}", result.status);
    println!("{:#?}", stderr);
}

fn compile_sources(task: &SourceTask, incremental: bool) {
    task.sources
        .iter()
        .filter(|rule| !incremental || is_stale(&rule))
        .map(|rule| compile(&rule))
        .for_each(|result| {
            let stderr = str::from_utf8(&result.stderr);
            println!("{}", result.status);
            println!("{:#?}", stderr);
        });
}

fn compile(rule: &Rule) -> Output {
    let mut command = Command::new("g++");
    command
        .arg(&rule.input)
        .arg("-o")
        .arg(&rule.output)
        .arg("-c");
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
    let dependencies = get_source_dependencies(file);

    Rule {
        input,
        dependencies,
        output,
    }
}

fn get_source_dependencies(file: &Path) -> Vec<PathBuf> {
    let mut command = Command::new("g++");
    command.arg(&file).arg("-MM");
    let result = command
        .output()
        .expect("failed to parse source dependencies");

    let stdout = String::from_utf8(result.stdout).expect("String is not utf-8");
    let dependencies: Vec<PathBuf> = stdout
        .split_whitespace()
        .filter(|&x| x != "\\")
        .skip(1)
        .map(|x| PathBuf::from(&x))
        .collect();

    println!("{:?}", dependencies);
    dependencies
}

fn is_stale(rule: &Rule) -> bool {
    let output_metadata = fs::metadata(&rule.output);

    if output_metadata.is_err() {
        return true;
    }

    let output_time = output_metadata
        .expect("Cannot read output metadata")
        .modified()
        .expect("Cannot read file modified time");

    let stale = rule
        .dependencies
        .iter()
        .map(|x| {
            fs::metadata(&x)
                .expect("Cannot read metadata")
                .modified()
                .expect("Cannot read file modified time")
        })
        .any(|dependency_time| dependency_time > output_time);

    stale
}
