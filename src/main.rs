#[derive(tidy_builder::Builder)]
pub struct Config<T> where T: std::fmt::Debug {
    value: T
}

fn main() {}

