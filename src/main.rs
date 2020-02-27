use crate::task::Task;

mod compile_task;
mod link_task;
mod task;

fn main() {
    let compile = compile_task::CompileTask::new();
    compile.run();
    let link = link_task::LinkTask::new();
    link.run();
}
