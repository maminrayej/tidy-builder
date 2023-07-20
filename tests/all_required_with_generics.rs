#[derive(tidy_builder::Builder)]
pub struct Test<T0, T1, T2, T3> {
    field0: T0,
    field1: T1,
    field2: T2,
    field3: T3,
}

#[test]
fn all_required_with_generics() {
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
