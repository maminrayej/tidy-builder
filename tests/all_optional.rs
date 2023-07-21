fn is_even(num: &usize) -> bool {
    num % 2 == 0
}

#[derive(tidy_builder::Builder)]
pub struct Test {
    #[builder(props = skip)]
    field0: Option<usize>,

    #[builder(props = into)]
    field1: Option<usize>,

    #[builder(props = once)]
    field2: Option<usize>,

    #[builder(props = into, once)]
    field3: Option<usize>,

    #[builder(check = |num| num % 2 == 0)]
    field4: Option<usize>,

    #[builder(props = into)]
    #[builder(check = |num| num % 2 == 0)]
    field5: Option<usize>,

    #[builder(props = into, once)]
    #[builder(check = is_even)]
    field6: Option<usize>,

    #[builder(name = new_field7)]
    #[builder(props = into, once)]
    #[builder(check = is_even)]
    field7: Option<usize>,

    #[builder(props = into, once)]
    #[builder(check = |args| args.iter().all(is_even))]
    #[builder(each = arg, |num| is_even(num))]
    args: Option<Vec<usize>>,
}

#[test]
fn all_optional() {
    let test = Test::builder()
        .field1(1usize)
        .field2(2)
        .field3(3usize)
        .field4(4)
        .unwrap()
        .field5(6usize)
        .unwrap()
        .field6(8usize)
        .unwrap()
        .new_field7(10usize)
        .unwrap()
        .build();

    assert_eq!(test.field0, None);
    assert_eq!(test.field1, Some(1));
    assert_eq!(test.field2, Some(2));
    assert_eq!(test.field3, Some(3));
    assert_eq!(test.field4, Some(4));
    assert_eq!(test.field5, Some(6));
    assert_eq!(test.field6, Some(8));
    assert_eq!(test.field7, Some(10));
}
