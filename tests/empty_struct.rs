#[derive(tidy_builder::Builder)]
pub struct Test {}

#[test]
fn empty_struct() {
    let _test = Test::builder().build();
}
