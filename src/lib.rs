mod attr;
mod ty;

use std::collections::HashMap;

use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use ty::wrapped_in;

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
    let lifetime_names: Vec<_> = lifetime_params.iter().map(|p| p.lifetime.clone()).collect();
    let const_params: Vec<_> = ast.generics.const_params().collect();
    let const_names: Vec<_> = const_params.iter().map(|p| p.ident.clone()).collect();
    let type_params: Vec<_> = ast.generics.type_params().collect();
    let type_names: Vec<_> = type_params.iter().map(|p| p.ident.clone()).collect();

    let mut builder_fields = Vec::with_capacity(named.len());
    let mut builder_inits = Vec::with_capacity(named.len());
    let mut builder_moves = Vec::with_capacity(named.len());
    let mut builder_field_names = Vec::with_capacity(named.len());
    let mut builder_all_false = Vec::with_capacity(named.len());
    let mut builder_const_params = Vec::with_capacity(named.len());
    let mut builder_const_names = Vec::with_capacity(named.len());
    let mut builder_impls = Vec::with_capacity(named.len());
    let mut builder_setters = Vec::with_capacity(named.len());
    let mut builder_final_values = Vec::with_capacity(named.len());
    let mut builder_guard_traits = Vec::with_capacity(named.len());
    let mut builder_guard_trait_idents = Vec::with_capacity(named.len());

    /*
        Indicates whether the builder() function should be async or not.
        If a default value is provided for a field that gets resolved in an async context,
        this flag will be set to true.
    */
    let mut is_builder_async = false;

    /*
        Indicates whether the build() function should be async or not.
        If a lazy value is provided for a field that gets resolved in an async context,
        this flag will be set to true.
    */
    let mut is_build_async = false;

    let mut req_index: usize = 0;
    for field in named.iter() {
        let ident = &field.ident;
        let ty = &field.ty;
        let attrs = &field_to_attrs[field];

        let wrapped_in_option = ty::wrapped_in_option(ty);

        if wrapped_in_option.is_some() && attrs.value.is_some() {
            return syn::Error::new(ty.span(), "Option cannot have a default value.")
                .into_compile_error()
                .into();
        }

        let ty = if let Some(wrapped) = wrapped_in_option {
            wrapped
        } else {
            ty
        };

        let required = wrapped_in_option.is_none() && !attrs.has_value();

        let lazy_field = format_ident!("lazy_{}", ident.as_ref().unwrap());

        /* Generate builder fields */
        builder_fields.push(if attrs.value.is_none() {
            quote! { #ident: ::std::option::Option<#ty> }
        } else {
            quote! { #ident: #ty }
        });

        if let Some(lazy) = &attrs.lazy {
            let lazy_ty = if lazy.is_async {
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
                attr::Value::Default(_) => quote! { #ident: ::std::default::Default::default() },
                attr::Value::Lit(lit) => quote! { #ident: #lit },
                attr::Value::Callable(callable) => {
                    if !callable.is_async {
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

        /* Generate builder moves and inits */
        if let Some(lazy) = &attrs.lazy {
            builder_moves.push(quote! { #lazy_field: self.#lazy_field });

            if let Some(callable) = &lazy.callable {
                builder_inits.push(quote! { #lazy_field: Box::new(#callable) });
            } else {
                builder_inits.push(quote! { #lazy_field: None });
            }
        }

        builder_moves.push(quote! { #ident: self.#ident });

        /* Generate builder field name */
        builder_field_names.push(quote! { #ident });

        /* Generate builder final values */
        let final_value = if required {
            quote! { unsafe { self.#ident.unwrap_unchecked() } }
        } else {
            quote! { self.#ident }
        };

        let mut do_override = quote! {};
        if let Some(lazy) = &attrs.lazy {
            let is_await = if lazy.is_async {
                quote! { .await }
            } else {
                quote! {}
            };

            let override_expr = if wrapped_in_option.is_some() {
                quote! { Some(self.#lazy_field()#is_await) }
            } else {
                quote! { self.#lazy_field()#is_await }
            };

            do_override = if lazy.do_override.is_some() {
                quote! { let #ident = #override_expr; }
            } else {
                quote! {
                    let #ident = if self.#ident.is_none() {
                        #override_expr
                    } else {
                        #final_value;
                    };
                }
            };
        }

        builder_final_values.push(quote! {
           let #ident = #final_value;

            #do_override
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

        let wrapped_in_option = ty::wrapped_in_option(ty);
        let ty = if let Some(wrapped) = wrapped_in_option {
            wrapped
        } else {
            ty
        };

        let required = wrapped_in_option.is_none() && !attrs.has_value();

        let setter_name = if let Some(name) = &attrs.name {
            name
        } else {
            ident.as_ref().unwrap()
        };

        let input_ty = if attrs.props.into {
            quote! { impl Into<#ty> }
        } else {
            quote! { #ty }
        };

        let visibility = (!attrs.props.hide).then_some(Some(quote! { pub }));

        let check = attrs.check.as_ref().map(|check| {
            let check_ty = quote! { &dyn Fn(&#ty) -> bool };

            quote! {
                let check: #check_ty = &#check;

                if !(check)(&#ident) {
                    return Err("Provided value is not valid".into());
                }
            }
        });

        let mut return_val = if required {
            quote! {
                #builder {
                    #(#builder_moves,)*
                }
            }
        } else {
            quote! { self }
        };
        if check.is_some() {
            return_val = quote! { Ok(#return_val) };
        }

        let before_names = builder_const_names.iter().take(req_index);
        let after_names = builder_const_names.iter().skip(req_index + 1);

        let mut return_ty = if required {
            quote! {
                #builder<#(#lifetime_names,)* #(#const_names,)* #(#before_names,)* true, #(#after_names,)* #(#type_names,)*>
            }
        } else {
            quote! {
                #builder<#(#lifetime_names,)* #(#const_names,)* #(#builder_const_names,)* #(#type_names,)*>
            }
        };
        if check.is_some() {
            return_ty = quote! { ::std::result::Result<#return_ty, ::std::boxed::Box<dyn ::std::error::Error>> };
        };

        let value = if attrs.props.into {
            quote! { let #ident = #ident.into(); }
        } else {
            quote! { let #ident = #ident; }
        };

        let assignment = if attrs.value.is_none() {
            quote! { self.#ident = Some(#ident); }
        } else {
            quote! { self.#ident = #ident; }
        };

        let setter = quote! {
            #visibility fn #setter_name(mut self, #ident: #input_ty) -> #return_ty {
                #value

                #check

                #assignment

                #return_val
            }
        };

        if required && attrs.props.once {
            let before_names = builder_const_names.iter().take(req_index);
            let after_names = builder_const_names.iter().skip(req_index + 1);
            let before_params = builder_const_params.iter().take(req_index);
            let after_params = builder_const_params.iter().skip(req_index + 1);

            builder_impls.push(quote! {
                impl<#(#lifetime_params,)* #(#const_params,)* #(#before_params,)*  #(#after_params,)* #(#type_params,)*>
                #builder<#(#lifetime_names,)* #(#const_names,)* #(#before_names,)* false, #(#after_names,)* #(#type_names,)*>
                    #where_clause
                {
                    #setter
                }
            });
        } else {
            if required || !attrs.props.skip {
                builder_setters.push(setter);
            }
        }

        /* Generate each setters */
        if let Some(each) = &attrs.each {
            let each_ident = &each.ident;

            let inner_ty = wrapped_in(ty, None).unwrap();
            let inner_args = &inner_ty.args;

            let container_ident = if let syn::Type::Path(type_path) = ty {
                &type_path.path.segments.last().unwrap().ident
            } else {
                return syn::Error::new(ty.span(), "").into_compile_error().into();
            };

            let update_container = if attrs.has_value() {
                quote! { self.#ident.extend(Some(#each_ident)); }
            } else {
                quote! {
                    match self.#ident.as_mut() {
                        Some(c) => c.extend(Some(#each_ident)),

                        None => {
                            let mut c = #container_ident::new();
                            c.extend(Some(#each_ident));
                            self.#ident = Some(c);
                        }
                    }
                }
            };

            let each_check = if let Some(callable) = &each.callable {
                let check_ty = quote! { &dyn Fn(&(#inner_args)) -> bool };

                Some(quote! {
                    let check: #check_ty = &#callable;

                    if !check(&#each_ident) {
                        return Err("Provided value is not valid".into());
                    }
                })
            } else {
                None
            };

            let return_ty = if each_check.is_some() && check.is_none() {
                quote! { ::std::result::Result<#return_ty, ::std::boxed::Box<dyn ::std::error::Error>> }
            } else {
                return_ty
            };

            let return_val = if each_check.is_some() && check.is_none() {
                quote! { Ok(#return_val) }
            } else {
                return_val
            };

            builder_setters.push(quote! {
                pub fn #each_ident(mut self, #each_ident: (#inner_args)) -> #return_ty {
                    #each_check

                    #update_container

                    #return_val
                }
            });
        }

        /* Generate guard traits */
        if required {
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
            let before_names = builder_const_names.iter().take(req_index);
            let after_names = builder_const_names.iter().skip(req_index + 1);
            let before_params = builder_const_params.iter().take(req_index);
            let after_params = builder_const_params.iter().skip(req_index + 1);

            builder_guard_traits.push(quote! {
                #error_message
                pub trait #trait_ident {}
                impl<#(#lifetime_params,)* #(#const_params,)* #(#before_params,)* #(#after_params,)* #(#type_params,)* >
                    #trait_ident for
                    #builder<#(#lifetime_names,)* #(#const_names,)* #(#before_names,)* true, #(#after_names,)* #(#type_names,)* >
                    #where_clause { }
            });

            builder_guard_trait_idents.push(trait_ident);
        }

        if required {
            req_index += 1;
        }
    }

    quote! {
        pub struct #builder<
            #(#lifetime_params,)*
            #(#const_params,)*
            #(#builder_const_params,)*
            #(#type_params,)*
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

            pub #is_build_async fn build(self) -> #ident #ty_generics
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
