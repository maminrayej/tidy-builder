use std::process::Command;

#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
    drop(t); // dropping the `TestCases`, runs the tests

    assert!(std::env::set_current_dir("./tests/nightly_ui").is_ok());

    let result = Command::new("cargo")
        .args(["+nightly", "test"])
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap()
        .status;
    assert!(result.success());
}
