#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder(skip)]
    set_later_opt: Option<usize>,

    #[builder(skip)]
    #[builder(default = 0)]
    set_later_def: usize,

    req1: usize,
}

fn main() {
    let my_struct = MyStruct::builder().req1(1).build();

    assert_eq!(my_struct.set_later_opt, None);
    assert_eq!(my_struct.set_later_def, 0);
    assert_eq!(my_struct.req1, 1);
}
