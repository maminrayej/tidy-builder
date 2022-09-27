mod attribute;
mod err;
mod generator;
mod generics;
mod wrap;

macro_rules! ret_on_err {
    ($e: expr) => {
        match $e {
            Ok(value) => value,
            Err(err) => {
                return err.into();
            }
        }
    };
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let generator = ret_on_err!(generator::Generator::new(&ast));

    ret_on_err!(generator.generate()).into()
}
