#![feature(rustc_attrs)]

#[derive(tidy_builder::Builder)]
struct Item {
    field1: Option<usize>,
    field2: usize,
}


fn main() {
    let item = Item::builder()
        .field1(10)
        .build();
}