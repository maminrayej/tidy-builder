#[derive(tidy_builder::Builder)]
struct Foo {
    #[builder(value = 0)]
    bar: usize,
}

fn main() {}