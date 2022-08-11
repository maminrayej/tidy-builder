#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder]
    args: Vec<String>,
}

fn main() {}
