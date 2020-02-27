use crate::task::Task;
use std::fs::{canonicalize, metadata, read_dir};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

struct Rule {
    input: PathBuf,
    dependencies: Vec<PathBuf>,
    output: PathBuf,
}

pub struct CompileTask {
    inputs: Vec<Rule>,
}

impl Rule {
    fn new(file: &Path) -> Rule {
        let source_directory = canonicalize("src/").expect("directory src/ does not exist");
        let out_directory = canonicalize("out/").expect("directory out/ does not exist");

        let file_name = file.file_name().expect("file does not exist");

        let mut output_file_name = PathBuf::from(&file_name);
        output_file_name.set_extension("o");

        let mut input = PathBuf::from(&source_directory);
        input.push(&file_name);

        let mut output = PathBuf::from(&out_directory);
        output.push(&output_file_name);
        let dependencies = Rule::get_dependencies(file);

        Rule {
            input,
            dependencies,
            output,
        }
    }

    fn get_dependencies(file: &Path) -> Vec<PathBuf> {
        let mut command = Command::new("g++");
        command.arg(&file).arg("-MM");
        let result = command
            .output()
            .expect("failed to parse source dependencies");

        let stdout = String::from_utf8(result.stdout).expect("String is not utf-8");
        let dependencies: Vec<PathBuf> = stdout
            .split_whitespace()
            .filter(|&x| x != "\\")
            .skip(1) // target name
            .map(|x| PathBuf::from(&x))
            .collect();

        println!("{:?}", dependencies);
        dependencies
    }
}

impl Task for Rule {
    fn run(&self) {
        let mut command = Command::new("g++");
        command
            .arg(&self.input)
            .arg("-o")
            .arg(&self.output)
            .arg("-c");
        println!("{:?}", command);
        let result = command.output().expect("failed to compile");
        let stderr = str::from_utf8(&result.stderr);
        println!("{}", result.status);
        println!("{:#?}", stderr);
    }

    fn is_stale(&self) -> bool {
        let output_metadata = metadata(&self.output);

        if output_metadata.is_err() {
            return true;
        }

        let output_time = output_metadata
            .expect("Cannot read output metadata")
            .modified()
            .expect("Cannot read file modified time");

        let stale = self
            .dependencies
            .iter()
            .map(|x| {
                metadata(&x)
                    .expect("Cannot read metadata")
                    .modified()
                    .expect("Cannot read file modified time")
            })
            .any(|dependency_time| dependency_time > output_time);

        stale
    }
}

impl CompileTask {
    pub fn new() -> CompileTask {
        let files = read_dir("src").expect("src directory does not exist");
        let rules = files
            .map(|file| file.unwrap())
            .filter(|file| file.path().extension().expect("file missing extension") == "cpp")
            .map(|file| Rule::new(&file.path()))
            .collect();

        CompileTask { inputs: rules }
    }
}

impl Task for CompileTask {
    fn run(&self) {
        self.inputs
            .iter()
            .filter(|&rule| rule.is_stale())
            .for_each(|rule| rule.run())
    }

    fn is_stale(&self) -> bool {
        true
    }
}
