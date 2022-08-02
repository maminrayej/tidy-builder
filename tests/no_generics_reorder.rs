#[derive(tidy_builder::Builder)]
struct MyStruct {
    req1: String,
    req2: String,
    opt1: Option<String>,
}

#[test]
fn no_generics_reorder() {
    let my_struct = MyStruct::builder()
        .opt1("opt1".to_string())
        .req2("req2".to_string())
        .req1("req1".to_string())
        .build();

    assert_eq!(my_struct.req1, "req1".to_string());
    assert_eq!(my_struct.req2, "req2".to_string());
    assert_eq!(my_struct.opt1, Some("opt1".to_string()));
}
