#[derive(tidy_builder::Builder)]
pub struct MyStruct {
    #[builder(each = "arg")]
    args: Vec<String>,
}

#[test]
fn repeated_setters() {
    let my_struct = MyStruct::builder()
        .arg("arg1".to_string())
        .arg("arg2".to_string())
        .build();

    assert_eq!(my_struct.args.len(), 2);
    assert!(my_struct.args.contains(&"arg1".to_string()));
    assert!(my_struct.args.contains(&"arg2".to_string()));
}
