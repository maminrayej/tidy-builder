mod attrs;

use syn::spanned::Spanned;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let result = match &ast.data {
        syn::Data::Struct(s) => for_struct(&ast, s),
        syn::Data::Enum(e) => for_enum(&ast, e),
        syn::Data::Union(_) => for_union(&ast),
    };

    let token_stream = match result {
        Ok(output) => output,
        Err(error) => error.into_compile_error(),
    };

    token_stream.into()
}

fn for_struct(
    ast: &syn::DeriveInput,
    struct_data: &syn::DataStruct,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    todo!()
}

fn for_enum(
    ast: &syn::DeriveInput,
    enum_data: &syn::DataEnum,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    todo!()
}

fn for_union(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    Err(syn::Error::new(ast.span(), "Unions are not supported"))
}
