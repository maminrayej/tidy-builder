#[derive(Default)]
pub struct MyParam {
    param: usize,
}

#[derive(tidy_builder::Builder)]
struct MyStruct<'a> {
    #[builder(default)]
    my_param: MyParam,

    #[builder(default = 3)]
    my_usize: usize,

    #[builder(default = 1.2)]
    my_float: f64,

    #[builder(default = "Name")]
    my_name: &'a str,

    #[builder(default = false)]
    my_flag: bool,

    #[builder(default)]
    opt2: Option<usize>,

    req1: usize,
    req2: usize,
    opt1: Option<usize>,
}

#[test]
fn default_values() {
    let my_struct = MyStruct::builder().req1(1).req2(2).build();

    assert_eq!(my_struct.my_param.param, 0);
    assert_eq!(my_struct.my_usize, 3);
    assert_eq!(my_struct.my_float, 1.2);
    assert_eq!(my_struct.my_name, "Name");
    assert!(!my_struct.my_flag);

    assert_eq!(my_struct.req1, 1);
    assert_eq!(my_struct.req2, 2);

    assert_eq!(my_struct.opt1, None);
    assert_eq!(my_struct.opt2, None);
}
