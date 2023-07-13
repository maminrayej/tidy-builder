mod attr;

use quote::quote;
use syn::spanned::Spanned;

macro_rules! return_on_error {
    ($e: expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => {
                return err.to_compile_error().into();
            }
        }
    };
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let syn::Data::Struct(
        syn::DataStruct {
            fields: syn::Fields::Named(
                syn::FieldsNamed {
                    named,
                    ..
                }
            ),
            ..
        }
    ) = &ast.data
    else {
        return syn::Error::new(
            ast.span(),
            "Only structs with named fields are supported by the Builder macro"
        )
        .into_compile_error()
        .into();
    };

    quote! {}.into()
}
