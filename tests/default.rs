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
    T: std::fmt::Debug + std::default::Default,
{
    #[builder(value = 8)]
    #[builder(props = into)]
    #[builder(check = |n| n % 2 == 0)]
    #[builder(name  = set_foo)]
    #[builder(lazy)]
    foo: usize,

    #[builder(value = 0)]
    #[builder(props = into)]
    #[builder(check = is_even)]
    #[builder(name  = set_bar)]
    #[builder(lazy  = override)]
    bar: usize,

    #[builder(value = default)]
    #[builder(props = once, into)]
    #[builder(name  = set_baz)]
    #[builder(lazy  = override)]
    baz: T,

    #[builder(value = default)]
    #[builder(name  = set_qux)]
    #[builder(each  = arg, |n: &usize| n % 2 == 0)]
    qux: Vec<&'a usize>,

    #[builder(value = default)]
    #[builder(props = once)]
    #[builder(name  = set_quxx)]
    #[builder(each  = kv, |&(k, _)| k % 2 == 0)]
    quxx: HashMap<usize, &'b str>,
}

#[test]
fn default() {
    let arg0 = 0;
    let arg2 = 2;

    let test = Test::<bool, false>::builder()
        .unwrap()
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

    assert_eq!(test.foo, 0);
    assert_eq!(test.bar, 2);
    assert_eq!(test.baz, true);
    assert_eq!(test.qux, vec![&arg0, &arg2]);
    assert_eq!(test.quxx, HashMap::from_iter(Some((0, "ferris"))));
}
