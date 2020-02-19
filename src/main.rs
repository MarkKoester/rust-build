use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

struct Rule {
    input: PathBuf,
    dependencies: Vec<PathBuf>,
    output: PathBuf,
}

struct SourceTask {
    inputs: Vec<Rule>,
}

struct LinkTask {
    inputs: Vec<PathBuf>,
    output: PathBuf,
}

trait Task {
    fn run(&self);
    fn is_stale(&self) -> bool;
}

impl Rule {
    fn new(file: &Path) -> Rule {
        let source_directory = fs::canonicalize("src/").expect("directory src/ does not exist");
        let out_directory = fs::canonicalize("out/").expect("directory out/ does not exist");

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
            .skip(1)
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
        let output_metadata = fs::metadata(&self.output);

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
                fs::metadata(&x)
                    .expect("Cannot read metadata")
                    .modified()
                    .expect("Cannot read file modified time")
            })
            .any(|dependency_time| dependency_time > output_time);

        stale
    }
}

impl SourceTask {
    fn new() -> SourceTask {
        let files = fs::read_dir("src").expect("src directory does not exist");
        let rules = files
            .map(|file| file.unwrap())
            .filter(|file| file.path().extension().expect("file missing extension") == "cpp")
            .map(|file| Rule::new(&file.path()))
            .collect();

        SourceTask { inputs: rules }
    }
}

impl Task for SourceTask {
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

impl LinkTask {
    fn new() -> LinkTask {
        let files = fs::read_dir("out/").expect("out directory does not exist");
        let objects: Vec<PathBuf> = files
            .map(|file| file.unwrap())
            .filter(|file| file.path().extension().is_some())
            .filter(|file| file.path().extension().expect("file missing extension") == "o")
            .map(|file| file.path())
            .collect();

        LinkTask {
            inputs: objects,
            output: PathBuf::from("out/target"),
        }
    }
}

impl Task for LinkTask {
    fn run(&self) {
        if !self.is_stale() {
            return;
        }

        let mut command = Command::new("g++");
        self.inputs.iter().for_each(|file| {
            command.arg(&file);
        });
        command.arg("-o").arg(&self.output);

        println!("{:?}", command);
        let result = command.output().expect("Link failed");
        let stderr = str::from_utf8(&result.stderr);

        println!("{}", result.status);
        println!("{:#?}", stderr);
    }

    fn is_stale(&self) -> bool {
        let output_metadata = fs::metadata(&self.output);

        if output_metadata.is_err() {
            return true;
        }

        let output_time = output_metadata
            .expect("Cannot read output metadata")
            .modified()
            .expect("Cannot read file modified time");

        let stale = self
            .inputs
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
}

fn main() {
    let task = SourceTask::new();
    task.run();
    let link_task = LinkTask::new();
    link_task.run();
}
