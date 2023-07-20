#[derive(tidy_builder::Builder)]
pub struct Test {
    field0: u8,
    field1: u16,
    field2: u32,
    field3: u64,
}

#[test]
fn all_required() {
    let test = Test::builder()
        .field0(0)
        .field1(1)
        .field2(2)
        .field3(3)
        .build();
    assert_eq!(test.field0, 0);
    assert_eq!(test.field1, 1);
    assert_eq!(test.field2, 2);
    assert_eq!(test.field3, 3);
}
