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
        .value_2(&"value_2")
        .optional("optional".to_string())
        .value(&"value_old")
        .value(&"value_new")
        .optional_2("optional_2".to_string())
        .build();

    assert_eq!(config.value, &"value_new");
    assert_eq!(config.value_2, &"value_2");
    assert_eq!(config.optional.unwrap(), "optional".to_string());
    assert_eq!(config.optional_2.unwrap(), "optional_2".to_string());
}
