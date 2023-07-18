use tidy_builder::Builder;

use std::fmt::Debug;

#[derive(Builder)]
pub struct Test<T: Debug> {
    field0: String,
    field1: T,
    field2: usize,
    field3: Option<usize>,
    field4: u32,
}

fn main() {}
