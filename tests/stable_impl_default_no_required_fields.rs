#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder(skip)]
    a: Option<usize>,

    #[builder(default = 5)]
    b: usize,
}

#[test]
fn main() {
    let my_struct = MyStruct::default();

    assert_eq!(my_struct.a, None);
    assert_eq!(my_struct.b, 5);
}
