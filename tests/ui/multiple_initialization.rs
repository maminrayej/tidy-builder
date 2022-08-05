#[derive(tidy_builder::Builder)]
struct MyStruct<'a, 'b: 'a, 'c, const N: usize, const FLG: bool, T: std::fmt::Debug>
where
    T: std::fmt::Display,
    'c: 'a,
{
    req1: &'a T,
    req2: &'b T,
    opt1: Option<&'c T>,
}

fn main() {
    let req2 = "req2".to_string();
    let opt1 = "opt1".to_string();

    let req1_new = "req1_new".to_string();
    let req2_new = "req2_new".to_string();
    let opt1_new = "opt1_new".to_string();

    let my_struct: MyStruct<0, false, String> = MyStruct::builder()
        .opt1(&opt1)
        .req2(&req2)
        .opt1(&opt1_new)
        .req2(&req2_new)
        .build();

    assert_eq!(my_struct.req1, &req1_new);
    assert_eq!(my_struct.req2, &req2_new);
    assert_eq!(my_struct.opt1, Some(&opt1_new));
}
