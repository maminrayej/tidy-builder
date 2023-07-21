use std::collections::HashMap;

fn is_even(num: &usize) -> bool {
    num % 2 == 0
}

#[derive(tidy_builder::Builder)]
pub struct Test {
    field0: usize,

    #[builder(props = into)]
    field1: usize,

    #[builder(props = once)]
    field2: usize,

    #[builder(props = into, once)]
    field3: usize,

    #[builder(check = |num| num % 2 == 0)]
    field4: usize,

    #[builder(props = into)]
    #[builder(check = |num| num % 2 == 0)]
    field5: usize,

    #[builder(props = into, once)]
    #[builder(check = is_even)]
    field6: usize,

    #[builder(name = new_field7)]
    #[builder(props = into, once)]
    #[builder(check = is_even)]
    field7: usize,

    #[builder(props = into, once)]
    #[builder(check = |args| args.iter().all(is_even))]
    #[builder(each = arg, is_even)]
    args: Vec<usize>,

    #[builder(each = kv)]
    kvs: HashMap<usize, usize>,
}

#[test]
fn all_required() {
    let test = Test::builder()
        .field0(0)
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
        .arg(8)
        .unwrap()
        .kv((2, 2))
        .kv((4, 4))
        .build();

    assert_eq!(test.field0, 0);
    assert_eq!(test.field1, 1);
    assert_eq!(test.field2, 2);
    assert_eq!(test.field3, 3);
    assert_eq!(test.field4, 4);
    assert_eq!(test.field5, 6);
    assert_eq!(test.field6, 8);
    assert_eq!(test.field7, 10);
}
