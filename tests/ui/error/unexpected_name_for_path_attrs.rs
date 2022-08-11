#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder(unknown)]
    args: Vec<String>,
}

fn main() {}
