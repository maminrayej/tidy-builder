fn is_even(num: &usize) -> bool {
    num % 2 == 0
}

#[derive(tidy_builder::Builder)]
pub struct Test {
    #[builder(props = skip)]
    field0: Option<usize>,

    #[builder(props = into)]
    field1: Option<usize>,

    #[builder(props = once)]
    field2: Option<usize>,

    #[builder(props = into, once)]
    field3: Option<usize>,

    #[builder(check = |num| num % 2 == 0)]
    field4: Option<usize>,

    #[builder(props = into)]
    #[builder(check = |num| num % 2 == 0)]
    field5: Option<usize>,

    #[builder(props = into, once)]
    #[builder(check = is_even)]
    field6: Option<usize>,

    #[builder(name = new_field7)]
    #[builder(props = into, once)]
    #[builder(check = is_even)]
    field7: Option<usize>,

    #[builder(props = into, once)]
    #[builder(check = |nums| nums.iter().all(is_even))]
    #[builder(each = arg, is_even)]
    args: Option<Vec<usize>>,
}

fn main() {}
