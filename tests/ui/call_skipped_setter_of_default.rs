#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder(skip)]
    set_later_opt: Option<usize>,

    #[builder(skip)]
    #[builder(default = 0)]
    set_later_def: usize,
}

fn main() {
    let _ = MyStruct::builder().set_later_def();
}
