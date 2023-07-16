use tidy_builder::Builder;

use std::fmt::Debug;

#[derive(Builder)]
struct Test<T: Debug> {
    #[builder(check = |field0| !field0.is_empty())]
    #[builder(lazy = || "Amin".to_string())]
    field0: String,

    #[builder(lazy)]
    #[builder(once)]
    field1: T,

    #[builder(value = 0)]
    field2: usize,

    #[builder(lazy)]
    field3: Option<usize>,

    field4: u32,
}

fn main() {}
