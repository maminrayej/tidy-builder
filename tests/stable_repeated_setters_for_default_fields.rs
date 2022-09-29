#[derive(tidy_builder::Builder)]
pub struct MyStruct {
    #[builder(default)]
    #[builder(each = "arg")]
    args: Vec<String>,
}

#[test]
fn repeated_setters() {
    let my_struct = MyStruct::builder()
        .arg("arg1".to_string())
        .arg("arg2".to_string())
        .build();
    let my_struct_empty = MyStruct::builder().build();

    assert_eq!(my_struct_empty.args.len(), 0);

    assert_eq!(my_struct.args.len(), 2);
    assert!(my_struct.args.contains(&"arg1".to_string()));
    assert!(my_struct.args.contains(&"arg2".to_string()));
}
