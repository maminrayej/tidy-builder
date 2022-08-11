#[derive(tidy_builder::Builder)]
pub struct MyStruct {
    #[builder(each = "arg")]
    args: Vec<String>,

    #[builder(each = "opt_args")]
    optional_args: Option<Vec<String>>,
}

#[test]
fn repeated_setters() {
    let my_struct = MyStruct::builder()
        .arg("arg1".to_string())
        .arg("arg2".to_string())
        .opt_args("opt_arg1".to_string())
        .opt_args("opt_arg2".to_string())
        .build();

    assert_eq!(my_struct.args.len(), 2);
    assert!(my_struct.args.contains(&"arg1".to_string()));
    assert!(my_struct.args.contains(&"arg1".to_string()));

    assert_eq!(my_struct.optional_args.as_ref().unwrap().len(), 2);
    assert!(my_struct
        .optional_args
        .as_ref()
        .unwrap()
        .contains(&"opt_arg1".to_string()));
    assert!(my_struct
        .optional_args
        .as_ref()
        .unwrap()
        .contains(&"opt_arg2".to_string()));
}
