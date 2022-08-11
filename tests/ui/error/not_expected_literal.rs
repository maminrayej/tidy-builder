#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder("arg")]
    args: Vec<String>,
}

fn main() {}
