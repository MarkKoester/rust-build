use crate::task::Task;
use colored::*;
use std::fs::{metadata, read_dir};
use std::path::PathBuf;
use std::process::Command;
use std::str;

pub struct LinkTask {
    inputs: Vec<PathBuf>,
    output: PathBuf,
}

impl LinkTask {
    pub fn new() -> LinkTask {
        let files = read_dir("out/").expect("out directory does not exist");
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
        println!("{}", "Linking".bold());

        if !self.is_stale() {
            println!("{}", "Done".bright_green().bold());
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
        println!("{}", "Done".bright_green().bold());
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
            .inputs
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
