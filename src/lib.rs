mod attr;

use std::collections::HashMap;

use quote::{format_ident, quote};
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

    let ident = &ast.ident;
    let builder = format_ident!("{}Builder", ast.ident);

    let mut field_to_attrs = HashMap::with_capacity(named.len());
    for field in named.iter() {
        let attrs = return_on_error!(attr::parse_attrs(field));

        field_to_attrs.insert(field, attrs);
    }

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let lifetime_params: Vec<_> = ast.generics.lifetimes().collect();
    let lifetime_param_names: Vec<_> = lifetime_params
        .iter()
        .map(|lifetime| lifetime.lifetime.clone())
        .collect();
    let const_params: Vec<_> = ast.generics.const_params().collect();
    let const_param_names: Vec<_> = const_params
        .iter()
        .map(|param| param.ident.clone())
        .collect();
    let type_params: Vec<_> = ast.generics.type_params().collect();
    let type_param_names: Vec<_> = type_params
        .iter()
        .map(|param| param.ident.clone())
        .collect();

    let mut builder_fields = vec![];
    let mut builder_inits = vec![];
    let mut builder_all_false = vec![];
    let mut builder_const_params = vec![];
    let mut builder_const_param_names = vec![];

    let mut required_idx: usize = 0;

    for field in named.iter() {
        let ident = &field.ident;
        let ty = &field.ty;
        let attrs = &field_to_attrs[field];

        let wrapped_in_option = wrapped_in_option(ty);

        let required = wrapped_in_option.is_none() && !attrs.has_value();

        let lazy_field = format_ident!("lazy_{}", ident.as_ref().unwrap());

        let check_field = format_ident!("check_{}", ident.as_ref().unwrap());

        /* Generate builder fields */
        builder_fields.push(if attrs.value.is_none() {
            quote! { #ident: ::std::option::Option<#ty> }
        } else {
            quote! { #ident: #ty }
        });
        if let Some(lazy) = &attrs.lazy {
            let has_default = lazy.closure.is_some();

            builder_fields.push(if lazy.is_async {
                if let Some(inner_ty) = wrapped_in_option {
                    let ty = &inner_ty.args;

                    if has_default {
                        quote! { #lazy_field: Box<dyn ::std::future::Future<Output = #ty>> }
                    } else {
                        quote! { #lazy_field: Option<Box<dyn ::std::future::Future<Output = #ty>>> }
                    }
                } else {
                    if has_default {
                        quote! { #lazy_field: Box<dyn ::std::future::Future<Output = #ty>> }
                    } else {
                        quote! { #lazy_field: Option<Box<dyn ::std::future::Future<Output = #ty>>> }
                    }
                }
            } else {
                if let Some(inner_ty) = wrapped_in_option {
                    let ty = &inner_ty.args;

                    if has_default {
                        quote! { #lazy_field: Box<dyn Fn() -> #ty> }
                    } else {
                        quote! { #lazy_field: Option<Box<dyn Fn() -> #ty>> }
                    }
                } else {
                    if has_default {
                        quote! { #lazy_field: Box<dyn Fn() -> #ty> }
                    } else {
                        quote! { #lazy_field: Option<Box<dyn Fn() -> #ty>> }
                    }
                }
            });
        }
        if attrs.check.is_some() {
            builder_fields.push(if let Some(inner_ty) = wrapped_in_option {
                let ty = &inner_ty.args;

                quote! { #check_field: Box<dyn Fn(#ty) -> bool> }
            } else {
                quote! { #check_field: Box<dyn Fn(#ty) -> bool> }
            });
        }

        /* Generate initialization of each field of the builder */
        builder_inits.push(if let Some(value) = &attrs.value {
            match value {
                attr::Value::Default => quote! { #ident: ::std::default::Default() },
                attr::Value::Lit(lit) => quote! { #ident: #lit },
                attr::Value::Closure(closure) => {
                    if closure.asyncness.is_none() {
                        quote! { #ident: (#closure)() }
                    } else {
                        quote! { #ident: (#closure)().await }
                    }
                }
            }
        } else {
            quote! { #ident: None }
        });
        if let Some(lazy) = &attrs.lazy {
            if let Some(closure) = &lazy.closure {
                builder_inits.push(quote! { #lazy_field: Box::new(#closure) });
            } else {
                builder_inits.push(quote! { #lazy_field: None });
            }
        }
        if let Some(check) = &attrs.check {
            builder_inits.push(quote! { #check_field: Box::new(#check) })
        }

        /* Generate builder generic params */
        if required {
            let required_param_name = format_ident!("REQ{}", required_idx);
            required_idx += 1;

            builder_const_param_names.push(quote! { #required_param_name });
            builder_const_params.push(quote! { const #required_param_name: bool });
            builder_all_false.push(quote! { false });
        }

        /* Generate setters */
    }

    quote! {
        pub struct #builder<
            #(#lifetime_param_names,)*
            #(#const_param_names,)*
            #(#builder_const_params,)*
            #(#type_param_names,)*
        > {
            #(#builder_fields,)*
        }

        impl #impl_generics #ident #ty_generics  #where_clause {
            pub fn builder() -> #builder<
                #(#lifetime_param_names,)*
                #(#const_param_names,)*
                #(#builder_all_false,)*
                #(#type_param_names,)*
            > {
                #builder {
                    #(#builder_inits,)*
                }
            }
        }
    }
    .into()
}

fn wrapped_in<'a>(
    ty: &'a syn::Type,
    wrapper: &str,
) -> Option<&'a syn::AngleBracketedGenericArguments> {
    let syn::Type::Path(syn::TypePath { path, .. }) = ty else { return None; };

    let args = (&path.segments)
        .last()
        .filter(|seg| seg.ident == wrapper)
        .map(|seg| &seg.arguments);

    if let Some(syn::PathArguments::AngleBracketed(args)) = args {
        Some(args)
    } else {
        None
    }
}

fn wrapped_in_option(ty: &syn::Type) -> Option<&syn::AngleBracketedGenericArguments> {
    wrapped_in(ty, "Option")
}
