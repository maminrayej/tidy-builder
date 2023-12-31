fn zero() -> usize {
    0
}

#[derive(tidy_builder::Builder)]
struct Foo {
    bar: usize,
    baz: Option<usize>,
    qux: String,
}

fn main() {}
