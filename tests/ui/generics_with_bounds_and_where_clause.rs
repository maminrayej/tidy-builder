#[derive(tidy_builder::Builder)]
struct MyStruct<T: std::fmt::Debug>
where
    T: std::fmt::Display,
{
    req1: T,
    req2: T,
    opt1: Option<T>,
}

fn main() {
    let my_struct = MyStruct::builder()
        .opt1("opt1".to_string())
        .req2("req2".to_string())
        .build();

    assert_eq!(my_struct.req1, "req1".to_string());
    assert_eq!(my_struct.req2, "req2".to_string());
    assert_eq!(my_struct.opt1, Some("opt1".to_string()));
}
