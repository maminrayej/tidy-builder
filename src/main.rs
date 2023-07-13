use tidy_builder::Builder;

#[derive(Builder)]
struct Test {
    #[builder(unknown)]
    field1: String,
}

fn main() {}
