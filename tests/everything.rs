fn field7_default() -> u64 {
    3 + 4
}

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

    #[builder(value = default)]
    field4: u8,
    #[builder(value = 5)]
    field5: u16,
    #[builder(value = || 3 + 3)]
    field6: u32,
    #[builder(value = field7_default)]
    field7: u64,

    #[builder(props = skip)]
    field8: Option<usize>,

    #[builder(name = renamed_field9)]
    field9: usize,

    #[builder(name = renamed_field10)]
    field10: Option<usize>,

    #[builder(name = renamed_field11)]
    #[builder(value = 0)]
    field11: usize,
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
        .renamed_field9(9)
        .renamed_field10(10)
        .renamed_field11(11)
        .build();

    assert_eq!(*test.field0, 0);
    assert_eq!(*test.field1, 1);
    assert_eq!(test.field2, &"Ferris");
    assert_eq!(test.field3, 3);
    assert_eq!(test.field4, 0);
    assert_eq!(test.field5, 5);
    assert_eq!(test.field6, 6);
    assert_eq!(test.field7, 7);
    assert_eq!(test.field8, None);
    assert_eq!(test.field9, 9);
    assert_eq!(test.field10, Some(10));
    assert_eq!(test.field11, 11);
}
