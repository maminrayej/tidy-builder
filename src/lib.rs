mod attrs;
mod ty;

use std::collections::HashMap;

use quote::quote;
use syn::spanned::Spanned;

macro_rules! error {
    ($src: expr, $msg: expr) => {
        return Err(syn::Error::new($src.span(), $msg));
    };
}

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
    let fields_cnt = struct_data.fields.len();
    let mut attr_map = HashMap::with_capacity(fields_cnt);
    for field in &struct_data.fields {
        let attrs = attrs::parse_attrs(field)?;

        attr_map.insert(field, attrs);
    }

    let builder_ident = quote::format_ident!("{}Builder", ast.ident);

    let mut builder_fields = Vec::with_capacity(fields_cnt);

    /* generate the builder struct definition */
    for field in &struct_data.fields {
        let ident = field.ident.as_ref().unwrap();
        let attrs = &attr_map[&field];

        let optional = ty::wrapped_in_option(&field.ty);
        let required = optional.is_none() && !attrs.has_value();

        /* validate the specified attributes */
        if optional.is_some() {
            if attrs.value.is_some() {
                error!(field.ty, "optional field cannot have default value");
            }
            if attrs.setter.once {
                error!(field.ty, "once cannot be applied to optional fields");
            }
        } else if required {
            if attrs.setter.skip {
                error!(field.span(), "cannot skip a required field");
            }
        } else {
            if attrs.setter.once {
                error!(field.span(), "once cannot be applied to fields with values");
            }
        }

        let ty = optional.unwrap_or(&field.ty);

        builder_fields.push(quote! { #ident: ::std::option::Option<#ty> });
    }

    Ok(quote! {
        struct #builder_ident {
            #(#builder_fields),*
        }
    })
}

fn for_enum(
    ast: &syn::DeriveInput,
    enum_data: &syn::DataEnum,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    todo!()
}

fn for_union(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    error!(ast, "Unions are not supported");
}
