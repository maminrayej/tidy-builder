use std::collections::HashMap;

use tidy_builder::Builder;

fn is_kv_valid(kv: &(usize, usize)) -> bool {
    true
}

#[derive(Builder)]
pub struct Test {
    // #[builder(each = kv, |&(k, v)| k % 2 != 0 && v % 2 == 0)]
    #[builder(each = kv, is_kv_valid)]
    kvs: HashMap<usize, usize>,
}

fn main() {}
