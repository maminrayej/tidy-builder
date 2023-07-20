#[derive(tidy_builder::Builder)]
pub struct Test {
    field0: Option<usize>,
    field1: Option<usize>,
    field2: Option<usize>,
    field3: Option<usize>,
}

#[test]
fn all_optional() {
    let test = Test::builder().build();
    assert_eq!(test.field0, None);
    assert_eq!(test.field1, None);
    assert_eq!(test.field2, None);
    assert_eq!(test.field3, None);

    let test = Test::builder().field0(0).field2(2).build();
    assert_eq!(test.field0, Some(0));
    assert_eq!(test.field1, None);
    assert_eq!(test.field2, Some(2));
    assert_eq!(test.field3, None);

    let test = Test::builder()
        .field0(0)
        .field1(1)
        .field2(2)
        .field3(3)
        .build();
    assert_eq!(test.field0, Some(0));
    assert_eq!(test.field1, Some(1));
    assert_eq!(test.field2, Some(2));
    assert_eq!(test.field3, Some(3));
}
