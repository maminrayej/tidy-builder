fn field3_default() -> u64 {
    3
}

#[derive(tidy_builder::Builder)]
pub struct Test {
    #[builder(value = default)]
    field0: u8,

    #[builder(value = 1)]
    field1: u16,

    #[builder(value = || 2)]
    field2: u32,

    #[builder(value = field3_default)]
    field3: u64,
}

#[test]
fn all_required() {
    let test = Test::builder().build();
    assert_eq!(test.field0, 0);
    assert_eq!(test.field1, 1);
    assert_eq!(test.field2, 2);
    assert_eq!(test.field3, 3);

    let test = Test::builder().field0(1).field2(3).build();
    assert_eq!(test.field0, 1);
    assert_eq!(test.field1, 1);
    assert_eq!(test.field2, 3);
    assert_eq!(test.field3, 3);

    let test = Test::builder()
        .field0(1)
        .field1(2)
        .field2(3)
        .field3(4)
        .build();
    assert_eq!(test.field0, 1);
    assert_eq!(test.field1, 2);
    assert_eq!(test.field2, 3);
    assert_eq!(test.field3, 4);
}
