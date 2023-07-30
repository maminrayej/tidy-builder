#[derive(tidy_builder::Builder)]
pub struct Test {
    #[builder(each = arg)]
    #[builder(lazy)]
    field0: Vec<usize>,
}

fn main() {
    let test = Test::builder()
        .arg(0)
        .arg(1)
        .lazy_field0(Box::new(|| vec![2, 3]))
        .build();

    assert_eq!(test.field0, vec![0, 1]);
}
