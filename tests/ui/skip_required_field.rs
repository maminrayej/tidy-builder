#[derive(tidy_builder::Builder)]
struct MyStruct {
    #[builder(skip)]
    set_later_opt: Option<usize>,

    #[builder(skip)]
    #[builder(default = 0)]
    set_later_def: usize,

    #[builder(skip)]
    req1: usize,
}

fn main() {}
