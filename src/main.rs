#[derive(tidy_builder::Builder)]
struct MyStruct<'a> {
    #[builder(each = "arg")]
    args: Vec<&'a str>,
}

fn main() {
    let m = MyStruct::builder().arg("arg1").arg("arg2").build();

    assert_eq!(m.args.len(), 2);
    assert!(m.args.contains(&"arg1"));
    assert!(m.args.contains(&"arg2"));
}
