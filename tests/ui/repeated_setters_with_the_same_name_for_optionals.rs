#[derive(tidy_builder::Builder)]
pub struct MyStruct {
    #[builder(each = "optional_args")]
    optional_args: Option<Vec<String>>,
}

fn main() {
    let my_struct = MyStruct::builder().optional_args(vec![]).build();
}
