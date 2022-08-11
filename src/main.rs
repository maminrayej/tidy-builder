#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder(each = "arg")]
    args: Option<Vec<usize>>,
}

fn main() {
    let my_struct = MyStruct::builder().arg(1).build();

    assert_eq!(my_struct.args.unwrap(), vec![1]);
}
