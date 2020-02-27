pub trait Task {
    fn run(&self);
    fn is_stale(&self) -> bool;
}
