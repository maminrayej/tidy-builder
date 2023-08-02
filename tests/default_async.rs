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

async fn give_eight() -> usize {
    8
}

async fn give_false() -> bool {
    false
}

#[derive(tidy_builder::Builder)]
pub struct Test<'a, 'b: 'a, const B: bool> {
    #[builder(value = async give_eight)]
    #[builder(props = into)]
    #[builder(check = |n| n % 2 == 0)]
    #[builder(name  = set_foo)]
    #[builder(lazy)]
    foo: usize,

    #[builder(value = async || async { 0 })]
    #[builder(props = into)]
    #[builder(check = is_even)]
    #[builder(name  = set_bar)]
    #[builder(lazy  = override)]
    bar: usize,

    #[builder(value = async || give_false())]
    #[builder(props = into)]
    #[builder(name  = set_baz)]
    #[builder(lazy  = override)]
    baz: bool,

    #[builder(value = default)]
    #[builder(name  = set_qux)]
    #[builder(each  = arg, |n: &usize| n % 2 == 0)]
    qux: Vec<&'a usize>,

    #[builder(value = default)]
    #[builder(name  = set_quxx)]
    #[builder(each  = kv, |&(k, _)| k % 2 == 0)]
    quxx: HashMap<usize, &'b str>,
}

#[tokio::test]
async fn default_async() {
    let arg0 = 0;
    let arg2 = 2;

    let test = Test::<false>::builder()
        .await
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
