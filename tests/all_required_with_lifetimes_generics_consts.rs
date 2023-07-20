#[derive(tidy_builder::Builder)]
pub struct Test<'a, 'b: 'a, 'c, T0: std::fmt::Debug, T1, const T2: usize, const T3: bool>
where
    T1: Default,
    'c: 'a + 'b,
{
    field0: &'a u8,
    field1: &'b u16,
    field2: &'c T0,
    field3: T1,
}

#[test]
fn all_required_with_lifetimes_generics_consts() {
    let field0 = 0;
    let field1 = 1;
    let field2 = "Ferris";

    let test: Test<_, _, 0, false> = Test::builder()
        .field0(&field0)
        .field1(&field1)
        .field2(&field2)
        .field3(3)
        .build();

    assert_eq!(*test.field0, 0);
    assert_eq!(*test.field1, 1);
    assert_eq!(test.field2, &"Ferris");
    assert_eq!(test.field3, 3);
}
