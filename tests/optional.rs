use std::collections::HashMap;

fn is_even(n: &usize) -> bool {
    *n % 2 == 0
}

struct MyUsize(usize);

impl From<MyUsize> for usize {
    fn from(value: MyUsize) -> Self {
        value.0
    }
}

#[derive(tidy_builder::Builder)]
pub struct Test<'a, 'b: 'a, T, const B: bool>
where
    T: std::fmt::Debug,
{
    #[builder(props = into)]
    #[builder(check = |n: &usize| n % 2 == 0)]
    #[builder(name  = set_foo)]
    #[builder(lazy)]
    foo: Option<usize>,

    #[builder(props = into)]
    #[builder(check = is_even)]
    #[builder(name  = set_bar)]
    #[builder(lazy  = override)]
    bar: Option<usize>,

    #[builder(props = into)]
    #[builder(name  = set_baz)]
    #[builder(lazy  = override)]
    baz: Option<T>,

    #[builder(name  = set_qux)]
    #[builder(each  = arg, |n: &usize| n % 2 == 0)]
    qux: Option<Vec<&'a usize>>,

    #[builder(name  = set_quxx)]
    #[builder(each  = kv, |&(k, _)| k % 2 == 0)]
    quxx: Option<HashMap<usize, &'b str>>,
}

#[test]
fn optional() {
    let arg0 = 0;
    let arg2 = 2;

    let test = Test::<bool, false>::builder()
        .set_foo(MyUsize(0))
        .unwrap()
        .lazy_foo(Box::new(|| 2))
        .set_bar(0usize)
        .unwrap()
        .lazy_bar(Box::new(|| 2))
        .set_baz(true)
        .arg(&arg0)
        .unwrap()
        .arg(&arg2)
        .unwrap()
        .kv((0, "ferris"))
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(test.foo, Some(0));
    assert_eq!(test.bar, Some(2));
    assert_eq!(test.baz, Some(true));
    assert_eq!(test.qux, Some(vec![&arg0, &arg2]));
    assert_eq!(test.quxx, Some(HashMap::from_iter(Some((0, "ferris")))));
}
