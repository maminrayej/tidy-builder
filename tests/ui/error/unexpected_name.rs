#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder(eac = "arg")]
    args: Vec<String>,
}

fn main() {}
