mod attr;

use std::collections::HashMap;

use convert_case::{Case, Casing};
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
    let mut builder_moves = vec![];
    let mut builder_all_false = vec![];
    let mut builder_const_params = vec![];
    let mut builder_const_param_names = vec![];
    let mut builder_impls = vec![];
    let mut builder_setters = vec![];
    let mut builder_final_values = vec![];
    let mut builder_guard_traits = vec![];
    let mut builder_guard_trait_idents = vec![];
    let mut builder_field_names = vec![];

    let mut is_new_async = false;
    let mut is_build_async = false;

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
                is_build_async = true;
                
                if let Some(inner_ty) = wrapped_in_option {
                    let ty = &inner_ty.args;

                    if has_default {
                        quote! { #lazy_field: ::std::boxed::Box<dyn ::std::future::Future<Output = #ty>> }
                    } else {
                        quote! { #lazy_field: std::option::Option<::std::boxed::Box<dyn ::std::future::Future<Output = #ty>>> }
                    }
                } else {
                    if has_default {
                        quote! { #lazy_field: ::std::boxed::Box<dyn ::std::future::Future<Output = #ty>> }
                    } else {
                        quote! { #lazy_field: ::std::option::Option<::std::boxed::Box<dyn ::std::future::Future<Output = #ty>>> }
                    }
                }
            } else {
                if let Some(inner_ty) = wrapped_in_option {
                    let ty = &inner_ty.args;

                    if has_default {
                        quote! { #lazy_field: ::std::boxed::Box<dyn Fn() -> #ty> }
                    } else {
                        quote! { #lazy_field: ::std::option::Option<::std::boxed::Box<dyn Fn() -> #ty>> }
                    }
                } else {
                    if has_default {
                        quote! { #lazy_field: ::std::boxed::Box<dyn Fn() -> #ty> }
                    } else {
                        quote! { #lazy_field: ::std::option::Option<::std::boxed::Box<dyn Fn() -> #ty>> }
                    }
                }
            });
        }
        if attrs.check.is_some() {
            builder_fields.push(if let Some(inner_ty) = wrapped_in_option {
                let ty = &inner_ty.args;

                quote! { #check_field: ::std::boxed::Box<dyn Fn(&#ty) -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error>>> }
            } else {
                quote! { #check_field: ::std::boxed::Box<dyn Fn(&#ty) -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error>>> }
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
                        is_new_async = true;

                        quote! { #ident: (#closure)().await }
                    }
                }
            }
        } else {
            quote! { #ident: None }
        });
        if let Some(lazy) = &attrs.lazy {
            builder_moves.push(quote! { #lazy_field: self.#lazy_field });

            if let Some(closure) = &lazy.closure {
                builder_inits.push(quote! { #lazy_field: Box::new(#closure) });
            } else {
                builder_inits.push(quote! { #lazy_field: None });
            }
        }
        if let Some(check) = &attrs.check {
            builder_inits.push(quote! { #check_field: Box::new(#check) });
            builder_moves.push(quote! { #check_field: self.#check_field});
        }

        /* Generate builder moves */
        builder_moves.push(quote! { #ident: self.#ident });

        /* Generate builder field names */
        builder_field_names.push(quote! { #ident });

        /* Generate builer build moves */
        builder_final_values.push(if required {
            if let Some(lazy) = &attrs.lazy {
                let is_await = lazy.is_async.then_some(Some(quote! { .await }));

                if lazy.override_value {
                    quote! { let #ident = self.#lazy_field()#is_await; }
                } else {
                    quote! {
                        let #ident = if self.#ident.is_none() {
                            self.#lazy_field()#is_await
                        } else {
                            unsafe { self.#ident.unwrap_unchecked() }
                        };
                    }
                }
            } else {
                quote! { let #ident = unsafe { self.#ident.unwrap_unchecked() }; }
            }
        } else if let Some(_) = wrapped_in_option {
            if let Some(lazy) = &attrs.lazy {
                let is_await = lazy.is_async.then_some(Some(quote! { .await }));

                if lazy.override_value {
                    quote! { let #ident = Some(self.#lazy_field()#is_await); }
                } else {
                    quote! {
                        let #ident = if self.#ident.is_none() {
                            Some(self.#lazy_field()#is_await)
                        } else {
                            self.#ident
                        };
                    }
                }
            } else {
                quote! { let #ident = self.#ident; }
            }
        } else {
            if let Some(lazy) = &attrs.lazy {
                let is_await = lazy.is_async.then_some(Some(quote! { .await }));

                if lazy.override_value {
                    quote! { let #ident = self.#lazy_field()#is_await; }
                } else {
                    quote! {
                        let #ident = if self.#ident.is_none() {
                            self.#lazy_field()#is_await
                        } else {
                            self.#ident
                        };
                    }
                }
            } else {
                quote! { let #ident = self.#ident; }
            }
        });

        /* Generate builder generic params */
        if required {
            let required_param_name = format_ident!("REQ{}", required_idx);
            required_idx += 1;

            builder_const_param_names.push(quote! { #required_param_name });
            builder_const_params.push(quote! { const #required_param_name: bool });
            builder_all_false.push(quote! { false });
        }
    }

    /* Generate setters, impls, and guards */
    let mut required_idx: usize = 0;

    for field in named.iter() {
        let ident = &field.ident;
        let ty = &field.ty;
        let attrs = &field_to_attrs[field];

        let wrapped_in_option = wrapped_in_option(ty);
        let required = wrapped_in_option.is_none() && !attrs.has_value();

        let check_field = format_ident!("check_{}", ident.as_ref().unwrap());

        let setter_name = if let Some(name) = &attrs.name {
            quote! { #name }
        } else {
            quote! { #ident }
        };

        let input_ty = if let Some(wrapped) = wrapped_in_option {
            let ty = &wrapped.args;

            if attrs.into {
                quote! { impl Into<#ty> }
            } else {
                quote! { #ty }
            }
        } else {
            if attrs.into {
                quote! { impl Into<#ty> }
            } else {
                quote! { #ty }
            }
        };

        let visibility = (!attrs.hide).then_some(Some(quote! { pub }));

        let check = attrs
            .check
            .as_ref()
            .map(|_| quote! {  self.#check_field(&#ident)?; });

        if required {
            if attrs.skip {
                return syn::Error::new(
                    field.span(),
                    "You cannot skip a field that has no default and/or lazy value",
                )
                .into_compile_error()
                .into();
            }

            let before_param_names = &builder_const_param_names[..required_idx];
            let after_param_names = &builder_const_param_names[required_idx + 1..];

            let before_params = &builder_const_params[..required_idx];
            let after_params = &builder_const_params[required_idx + 1..];

            required_idx += 1;

            let mut return_ty = quote! {
                #builder<#(#lifetime_param_names,)* #(#const_param_names,)* #(#before_param_names,)* true, #(#after_param_names,)* #(#type_param_names,)*>
            };

            if check.is_some() {
                return_ty = quote! { Result<#return_ty, Box<dyn ::std::error::Error>> };
            }

            let setter = quote! {
                #visibility fn #setter_name(mut self, #ident: #input_ty) -> #return_ty {
                    #check

                    self.#ident = Some(#ident);

                    #builder {
                        #(#builder_moves,)*
                    }
                }
            };

            if attrs.once {
                builder_impls.push(
                    quote! {
                        impl<#(#lifetime_params,)* #(#const_params,)* #(#before_params,)*  #(#after_params,)* #(#type_params,)*>
                        #builder<#(#lifetime_param_names,)* #(#const_param_names,)* #(#before_param_names,)* false, #(#after_param_names,)* #(#type_param_names,)*>
                            #where_clause
                        {
                            #setter
                        }
                    }
                )
            } else {
                builder_setters.push(setter);
            };

            let ident_string = ident.as_ref().unwrap().to_string();
            let ident_camel = ident_string.to_case(Case::UpperCamel);
            let trait_ident = format_ident!("Has{}", ident_camel);

            // This feature uses `#[rustc_on_unimplemented]` which is only available
            // in a nightly compiler.
            let mut error_message = None;
            if cfg!(feature = "better_error") {
                let message = format!("missing `{}`", &ident_string);
                let label = format!("provide `{}` before calling `.build()`", &ident_string);
                error_message = Some(quote! {
                    #[rustc_on_unimplemented(
                        message=#message,
                        label=#label,
                    )]
                });
            }

            builder_guard_traits.push(quote! {
                #error_message
                trait #trait_ident {}
                impl<#(#lifetime_params,)* #(#const_params,)* #(#before_params,)* #(#after_params,)* #(#type_params,)* >
                    #trait_ident for
                    #builder<#(#lifetime_param_names,)* #(#const_param_names,)* #(#before_param_names,)* true, #(#after_param_names,)* #(#type_param_names,)* >
                    #where_clause { }
            });

            builder_guard_trait_idents.push(trait_ident);
        } else if let Some(_) = wrapped_in_option {
            if attrs.skip {
                continue;
            }

            let mut return_ty = quote! {
                #builder<#(#lifetime_param_names,)* #(#const_param_names,)* #(#builder_const_param_names,)* #(#type_param_names,)*>
            };

            if check.is_some() {
                return_ty = quote! { Result<#return_ty, Box<dyn ::std::error::Error>> };
            }

            builder_setters.push(quote! {
                #visibility fn #setter_name(mut self, #ident: #input_ty) -> #return_ty {
                    #check

                    self.#ident = Some(#ident);
                    self
                }
            });
        } else {
            if attrs.skip {
                continue;
            }

            let mut return_ty = quote! {
                #builder<#(#lifetime_param_names,)* #(#const_param_names,)* #(#builder_const_param_names,)* #(#type_param_names,)*>
            };

            if check.is_some() {
                return_ty = quote! { Result<#return_ty, Box<dyn ::std::error::Error>> };
            }

            builder_setters.push(quote! {
                #visibility fn #setter_name(mut self, #ident: #input_ty) -> #return_ty {
                    #check

                    self.#ident = #ident;
                    self
                }
            });
        }
    }

    let is_new_async = is_new_async.then_some(Some(quote! { async }));
    let is_build_async =  is_build_async.then_some(Some(quote! { async }));

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
            pub #is_new_async fn builder() -> #builder<
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

        #(#builder_impls)*

        impl<#(#lifetime_params,)* #(#const_params,)* #(#builder_const_params,)* #(#type_params,)*>
            #builder<#(#lifetime_param_names,)* #(#const_param_names,)* #(#builder_const_param_names,)* #(#type_param_names,)* >
            #where_clause
        {
            #(#builder_setters)*

            pub fn #is_build_async build(self) -> #ident #ty_generics
                where Self: #(#builder_guard_trait_idents)+*
            {
                #(#builder_final_values)*

                #ident {
                    #(#builder_field_names,)*
                }
            }
        }

        #(#builder_guard_traits)*
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
