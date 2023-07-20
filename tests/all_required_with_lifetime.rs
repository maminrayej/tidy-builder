#[derive(tidy_builder::Builder)]
pub struct Test<'a, 'b, 'c, 'd, T0, T1, T2, T3> {
    field0: &'a T0,
    field1: &'b T1,
    field2: &'c T2,
    field3: &'d T3,
}

#[test]
fn all_required_with_lifetime() {
    let field0 = 0;
    let field1 = 1;
    let field2 = 2;
    let field3 = 3;

    let test = Test::builder()
        .field0(&field0)
        .field1(&field1)
        .field2(&field2)
        .field3(&field3)
        .build();

    assert_eq!(*test.field0, 0);
    assert_eq!(*test.field1, 1);
    assert_eq!(*test.field2, 2);
    assert_eq!(*test.field3, 3);
}
