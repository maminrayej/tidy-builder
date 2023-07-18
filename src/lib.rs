mod attr;

use std::collections::HashMap;

use convert_case::{Case, Casing};
// use convert_case::{Case, Casing};
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

#[derive(Debug, Clone)]
enum FieldType<'a> {
    Type(&'a syn::Type),
    Args(&'a syn::AngleBracketedGenericArguments),
}

impl<'a> quote::ToTokens for FieldType<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldType::Type(ty) => ty.to_tokens(tokens),
            FieldType::Args(args) => args.args.to_tokens(tokens),
        }
    }
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
    let lifetime_names: Vec<_> = lifetime_params.iter().map(|p| p.lifetime.clone()).collect();
    let const_params: Vec<_> = ast.generics.const_params().collect();
    let const_names: Vec<_> = const_params.iter().map(|p| p.ident.clone()).collect();
    let type_params: Vec<_> = ast.generics.type_params().collect();
    let type_names: Vec<_> = type_params.iter().map(|p| p.ident.clone()).collect();

    let mut builder_fields = vec![];
    let mut builder_inits = vec![];
    let mut builder_moves = vec![];
    let mut builder_field_names = vec![];
    let mut builder_all_false = vec![];
    let mut builder_const_params = vec![];
    let mut builder_const_names = vec![];
    let mut builder_impls = vec![];
    let mut builder_setters = vec![];
    let mut builder_final_values = vec![];
    let mut builder_guard_traits = vec![];
    let mut builder_guard_trait_idents = vec![];

    let mut is_builder_async = false;
    let mut is_build_async = false;

    let mut req_index: usize = 0;
    for field in named.iter() {
        let ident = &field.ident;
        let ty = &field.ty;
        let attrs = &field_to_attrs[field];

        let wrapped_in_option = wrapped_in_option(ty);

        let ty = if let Some(wrapped) = wrapped_in_option {
            FieldType::Args(wrapped)
        } else {
            FieldType::Type(ty)
        };

        let required = wrapped_in_option.is_none() && !attrs.has_value();

        let lazy_field = format_ident!("lazy_{}", ident.as_ref().unwrap());
        let check_field = format_ident!("check_{}", ident.as_ref().unwrap());

        /* Generate builder fields */
        builder_fields.push(if attrs.value.is_none() {
            quote! { #ident: ::std::option::Option<#ty> }
        } else {
            quote! { #ident: #ty }
        });

        if attrs.check.is_some() {
            builder_fields.push(
                quote! { #check_field: ::std::boxed::Box<dyn Fn(&#ty) -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error>>> }
            );
        }

        if let Some(lazy) = &attrs.lazy {
            let lazy_ty = if lazy.asyncness.is_some() {
                is_build_async = true;

                quote! { ::std::boxed::box<dyn ::std::future::Future<Output = #ty>> }
            } else {
                quote! { ::std::boxed::Box<dyn Fn() -> #ty> }
            };

            let lazy_ty = if lazy.callable.is_none() {
                quote! { ::std::option::Option<#lazy_ty> }
            } else {
                lazy_ty
            };

            builder_fields.push(quote! {
               #lazy_field: #lazy_ty
            });
        }

        /* Generate initialization of each field of the builder */
        builder_inits.push(if let Some(value) = &attrs.value {
            match value {
                attr::Value::Default(_) => quote! { #ident: ::std::default::Default() },
                attr::Value::Lit(lit) => quote! { #ident: #lit },
                attr::Value::Callable(callable) => {
                    if callable.asyncness.is_none() {
                        quote! { #ident: (#callable)() }
                    } else {
                        is_builder_async = true;

                        quote! { #ident: (#callable)().await }
                    }
                }
            }
        } else {
            quote! { #ident: None }
        });

        if let Some(lazy) = &attrs.lazy {
            builder_moves.push(quote! { #lazy_field: self.#lazy_field });

            if let Some(callable) = &lazy.callable {
                builder_inits.push(quote! { #lazy_field: Box::new(#callable) });
            } else {
                builder_inits.push(quote! { #lazy_field: None });
            }
        }
        if let Some(check) = &attrs.check {
            builder_inits.push(quote! { #check_field: Box::new(#check) });
            builder_moves.push(quote! { #check_field: self.#check_field});
        }

        builder_moves.push(quote! { #ident: self.#ident });

        builder_field_names.push(quote! { #ident });

        builder_final_values.push(if required {
            if let Some(lazy) = &attrs.lazy {
                let is_await = lazy.asyncness.map(|_| quote! { .await });

                if lazy.do_override.is_some() {
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
                let is_await = lazy.asyncness.map(|_| quote! { .await });

                if lazy.do_override.is_some() {
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
                let is_await = lazy.asyncness.map(|_| quote! { .await });

                if lazy.do_override.is_some() {
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
            let required_param_name = format_ident!("_REQ{}", req_index);
            req_index += 1;

            builder_const_names.push(quote! { #required_param_name });
            builder_const_params.push(quote! { const #required_param_name: bool });
            builder_all_false.push(quote! { false });
        }
    }

    let is_builder_async = is_builder_async.then_some(Some(quote! { async }));
    let is_build_async = is_build_async.then_some(Some(quote! { async }));

    /* Generate setters, impls, and guards */
    let mut req_index = 0;
    for field in named.iter() {
        let ident = &field.ident;
        let ty = &field.ty;
        let attrs = &field_to_attrs[field];

        let wrapped_in_option = wrapped_in_option(ty);
        let ty = if let Some(wrapped) = wrapped_in_option {
            FieldType::Args(wrapped)
        } else {
            FieldType::Type(ty)
        };

        let required = wrapped_in_option.is_none() && !attrs.has_value();

        let check_field = format_ident!("check_{}", ident.as_ref().unwrap());

        let setter_name = if let Some(name) = &attrs.name {
            name
        } else {
            ident.as_ref().unwrap()
        };

        let input_ty = if attrs.props.into {
            quote! { Into<#ty> }
        } else {
            quote! { #ty }
        };

        let visibility = (!attrs.props.hide).then_some(Some(quote! { pub }));

        let check = attrs.check.as_ref().map(|_| {
            quote! {
                if !self.#check_field(&#ident) {
                    return Err("Provided value is not valie".into());
                }
            }
        });

        if required {
            if attrs.props.skip {
                return syn::Error::new(
                    field.span(),
                    "You cannot skip a field that has no default and/or lazy value",
                )
                .into_compile_error()
                .into();
            }

            let before_names = &builder_const_names[..req_index];
            let after_names = &builder_const_names[req_index + 1..];

            let before_params = &builder_const_params[..req_index];
            let after_params = &builder_const_params[req_index + 1..];

            req_index += 1;

            let mut return_ty = quote! {
                #builder<#(#lifetime_names,)* #(#const_names,)* #(#before_names,)* true, #(#after_names,)* #(#type_names,)*>
            };
            let mut return_val = quote! {
                #builder {
                    #(#builder_moves,)*
                }
            };

            if check.is_some() {
                return_ty = quote! { ::std::result::Result<#return_ty, ::std::boxed::Box<dyn ::std::error::Error>> };
                return_val = quote! { Ok(#return_val) };
            }

            let setter = quote! {
                #visibility fn #setter_name(mut self, #ident: #input_ty) -> #return_ty {
                    #check

                    self.#ident = Some(#ident);

                    #return_val
                }
            };

            if attrs.props.once {
                builder_impls.push(quote! {
                    impl<#(#lifetime_params,)* #(#const_params,)* #(#before_params,)*  #(#after_params,)* #(#type_params,)*>
                    #builder<#(#lifetime_names,)* #(#const_names,)* #(#before_names,)* false, #(#after_names,)* #(#type_names,)*>
                        #where_clause
                    {
                        #setter
                    }
                });
            } else {
                builder_setters.push(setter);
            }

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
                pub trait #trait_ident {}
                impl<#(#lifetime_params,)* #(#const_params,)* #(#before_params,)* #(#after_params,)* #(#type_params,)* >
                    #trait_ident for
                    #builder<#(#lifetime_names,)* #(#const_names,)* #(#before_names,)* true, #(#after_names,)* #(#type_names,)* >
                    #where_clause { }
            });

            builder_guard_trait_idents.push(trait_ident);
        } else if let Some(_) = wrapped_in_option {
            if attrs.props.skip {
                continue;
            }

            let mut return_ty = quote! {
                #builder<#(#lifetime_names,)* #(#const_names,)* #(#builder_const_names,)* #(#type_names,)*>
            };

            let mut return_val = quote! {
                self
            };

            if check.is_some() {
                return_ty = quote! { ::std::result::Result<#return_ty, ::std::boxed::Box<dyn ::std::error::Error>> };
                return_val = quote! { Ok(self) };
            }

            builder_setters.push(quote! {
                #visibility fn #setter_name(mut self, #ident: #input_ty) -> #return_ty {
                    #check

                    self.#ident = Some(#ident);

                    #return_val
                }
            });
        } else {
            if attrs.props.skip {
                continue;
            }

            let mut return_ty = quote! {
                #builder<#(#lifetime_names,)* #(#const_names,)* #(#builder_const_names,)* #(#type_names,)*>
            };

            let mut return_val = quote! {
                self
            };

            if check.is_some() {
                return_ty = quote! { ::std::result::Result<#return_ty, ::std::boxed::Box<dyn ::std::error::Error>> };
                return_val = quote! { Ok(self) }
            }

            builder_setters.push(quote! {
                #visibility fn #setter_name(mut self, #ident: #input_ty) -> #return_ty {
                    #check

                    self.#ident = #ident;

                    #return_val
                }
            });
        }
    }

    quote! {
        pub struct #builder<
            #(#lifetime_names,)*
            #(#const_names,)*
            #(#builder_const_params,)*
            #(#type_names,)*
        > {
            #(#builder_fields,)*
        }

        impl #impl_generics #ident #ty_generics  #where_clause {
            pub #is_builder_async fn builder() -> #builder<
                #(#lifetime_names,)*
                #(#const_names,)*
                #(#builder_all_false,)*
                #(#type_names,)*
            > {
                #builder {
                    #(#builder_inits,)*
                }
            }
        }

        #(#builder_impls)*

        impl<#(#lifetime_params,)* #(#const_params,)* #(#builder_const_params,)* #(#type_params,)*>
            #builder<#(#lifetime_names,)* #(#const_names,)* #(#builder_const_names,)* #(#type_names,)* >
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
    wrapper: Option<&str>,
) -> Option<&'a syn::AngleBracketedGenericArguments> {
    let syn::Type::Path(syn::TypePath { path, .. }) = ty else { return None; };

    let args = (&path.segments)
        .last()
        .filter(|seg| wrapper.iter().any(|wrapper| seg.ident == wrapper))
        .map(|seg| &seg.arguments);

    if let Some(syn::PathArguments::AngleBracketed(args)) = args {
        Some(args)
    } else {
        None
    }
}

fn wrapped_in_option(ty: &syn::Type) -> Option<&syn::AngleBracketedGenericArguments> {
    wrapped_in(ty, Some("Option"))
}
