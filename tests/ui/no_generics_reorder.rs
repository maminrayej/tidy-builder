#[derive(tidy_builder::Builder)]
struct MyStruct {
    req1: String,
    req2: String,
    opt1: Option<String>,
}

fn main() {
    let my_struct = MyStruct::builder()
        .req1("req1".to_string())
        .build();

    assert_eq!(my_struct.req1, "req1".to_string());
    assert_eq!(my_struct.req2, "req2".to_string());
    assert_eq!(my_struct.opt1, Some("opt1".to_string()));
}
