#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder(builder(each = "each"))]
    args: Vec<String>,
}

fn main() {}
