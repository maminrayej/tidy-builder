#[derive(tidy_builder::Builder)]
pub struct MyStruct {
    #[builder(each = "args")]
    args: Vec<String>,
}

fn main() {
    let my_struct = MyStruct::builder()
        .args(vec![])
        .build();
}
