#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/nightly_ui/**/*.rs");
}
