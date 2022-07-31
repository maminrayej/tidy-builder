#[derive(tidy_builder::Builder)]
pub struct Config<'a, const N: usize, T: std::fmt::Display>
where
    T: std::fmt::Debug,
{
    value: &'a T,
    value_2: &'a T,
    optional: Option<String>,
    optional_2: Option<String>,
}

fn main() {
    let config: Config<'static, 0, &str> = Config::builder()
        .value(&"amin")
        .value(&"sahar")
        .optional("opt1".to_string())
        .optional_2("opt2".to_string())
        .value_2(&"faezeh")
        .build();

    assert_eq!(config.value, &"sahar");
    assert_eq!(config.value_2, &"faezeh");
    assert_eq!(config.optional.unwrap(), "opt1".to_string());
    assert_eq!(config.optional_2.unwrap(), "opt2".to_string());
    // Config::builder().value(&"amin").value_2(&"faezeh").build();
    // Config::builder().value(&"amin").value_2(&"faezeh").build();
    // Config::builder().value(&"amin").value_2(&"faezeh").build();
}
