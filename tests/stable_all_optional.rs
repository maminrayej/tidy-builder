#[derive(tidy_builder::Builder)]
struct MyStruct {
    opt1: Option<usize>,
    opt2: Option<usize>,
}

#[test]
fn default_values() {
    let my_struct = MyStruct::builder().build();

    assert!(my_struct.opt1.is_none());
    assert!(my_struct.opt2.is_none());
}
