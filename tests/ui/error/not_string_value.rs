#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder(each = 1)]
    args: Vec<String>,
}

fn main() {}
