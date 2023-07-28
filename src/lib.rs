/* TODO: apply check to initial values and values returned by lazy functions */

mod attr;
mod ty;

use std::collections::HashMap;

use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use syn::spanned::Spanned;

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
        let attrs = match attr::parse_attrs(field) {
            Ok(attrs) => attrs,
            Err(error) => {
                return error.into_compile_error().into();
            }
        };

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
        let attrs = &field_to_attrs[field];

        let optional = ty::wrapped_in_option(&field.ty);

        if optional.is_some() && attrs.value.is_some() {
            return syn::Error::new(field.ty.span(), "Option cannot have a value property.")
                .into_compile_error()
                .into();
        }

        let ty = optional.unwrap_or(&field.ty);

        let required = optional.is_none() && !attrs.has_value();

        let lazy_field = format_ident!("lazy_{}", ident.as_ref().unwrap());

        /* Generate builder fields */
        builder_fields.push(quote! { #ident: ::std::option::Option<#ty> });

        if let Some(lazy) = &attrs.lazy {
            let lazy_ty = if lazy.is_async {
                is_build_async = true;

                quote! {
                    ::std::option::Option<::std::boxed::Box<dyn ::std::future::Future<Output = #ty>>>
                }
            } else {
                quote! {
                    ::std::option::Option<::std::boxed::Box<dyn Fn() -> #ty>>
                }
            };

            builder_fields.push(quote! { #lazy_field: #lazy_ty });
        }

        /* Generate initialization of each field of the builder */
        builder_inits.push(if let Some(value) = &attrs.value {
            match value {
                attr::Value::Default(_) => {
                    quote! { #ident: Some(::std::default::Default::default()) }
                }
                attr::Value::Lit(lit) => {
                    quote! { #ident: Some(#lit) }
                }
                attr::Value::Callable(callable) => {
                    if callable.is_async {
                        is_builder_async = true;

                        quote! { #ident: Some((#callable)().await) }
                    } else {
                        quote! { #ident: Some((#callable)()) }
                    }
                }
            }
        } else {
            quote! { #ident: None }
        });

        if let Some(lazy) = &attrs.lazy {
            builder_moves.push(quote! { #lazy_field: self.#lazy_field });

            if let Some(callable) = &lazy.callable {
                builder_inits.push(quote! { #lazy_field: Some(Box::new(#callable)) });
            } else {
                builder_inits.push(quote! { #lazy_field: None });
            }
        }

        builder_moves.push(quote! { #ident: self.#ident });

        /* Generate builder field names */
        builder_field_names.push(quote! { #ident });

        /* Generate builder final values */
        let initial_value = quote! {
            let #ident = self.#ident;
        };

        let mut override_stmt = quote! {};
        if let Some(lazy) = &attrs.lazy {
            let is_await = lazy.is_async.then_some(quote! { .await });

            let none_none = if optional.is_some() {
                quote! { None }
            } else {
                quote! { unreachable!() }
            };

            let some_some = if lazy.do_override.is_some() {
                quote! { (lazy)()#is_await }
            } else {
                quote! { value }
            };

            override_stmt = quote! {
                let #ident = match (self.#lazy_field, self.#ident) {
                    (Some(lazy), Some(value)) => Some(#some_some),
                    (Some(lazy), None) => Some((lazy)()#is_await),
                    (None, Some(value)) => Some(value),
                    (None, None) => #none_none,
                };
            };
        }

        let final_value = if optional.is_some() {
            quote! { let #ident = #ident; }
        } else {
            quote! { let #ident = unsafe { #ident.unwrap_unchecked() }; }
        };

        builder_final_values.push(quote! {
            #initial_value

            #override_stmt

            #final_value
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
        let attrs = &field_to_attrs[field];

        let optional = ty::wrapped_in_option(&field.ty);
        let required = optional.is_none() && !attrs.has_value();

        let ty = optional.unwrap_or(&field.ty);
        let setter_name = attrs.name.as_ref().or(ident.as_ref());

        let input_ty = if attrs.props.into {
            quote! { impl Into<#ty> }
        } else {
            quote! { #ty }
        };

        let visibility = (!attrs.props.hide).then_some(Some(quote! { pub }));

        let check_stmt = attrs.check.as_ref().map(|check| {
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
        if check_stmt.is_some() {
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
        if check_stmt.is_some() {
            return_ty = quote! { ::std::result::Result<#return_ty, ::std::boxed::Box<dyn ::std::error::Error>> };
        };

        let value = attrs
            .props
            .into
            .then_some(Some(quote! { let #ident = #ident.into(); }));

        let assignment = quote! { self.#ident = Some(#ident); };

        let setter = quote! {
            #visibility fn #setter_name(mut self, #ident: #input_ty) -> #return_ty {
                #value

                #check_stmt

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
        } else if required || !attrs.props.skip {
            builder_setters.push(setter);
        }

        /* Generate lazy setter */
        if let Some(lazy) = &attrs.lazy {
            let lazy_ident = format_ident!("lazy_{}", ident.as_ref().unwrap());

            let input_ty = if lazy.is_async {
                quote! { ::std::boxed::Box<dyn ::std::future::Future<Output = #ty>> }
            } else {
                quote! { ::std::boxed::Box<dyn Fn() -> #ty> }
            };

            let setter = quote! {
                #visibility fn #lazy_ident(mut self, #lazy_ident: #input_ty) -> #return_ty {
                    self.#lazy_ident = Some(#lazy_ident);

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
            } else if required || !attrs.props.skip {
                builder_setters.push(setter);
            }
        }

        /* Generate each setters */
        if let Some(each) = &attrs.each {
            let each_ident = &each.ident;

            let inner_ty = match ty::wrapped_in(ty, None) {
                Some(inner_ty) => inner_ty,
                None => {
                    return syn::Error::new(
                        ty.span(),
                        "Expected a container with generic arguments",
                    )
                    .into_compile_error()
                    .into();
                }
            };

            let inner_args = &inner_ty.args;

            let container_ident = if let syn::Type::Path(type_path) = ty {
                &type_path.path.segments.last().unwrap().ident
            } else {
                return syn::Error::new(ty.span(), "Failed to get the container name")
                    .into_compile_error()
                    .into();
            };

            let update_stmt = quote! {
                match self.#ident.as_mut() {
                    Some(c) => c.extend(Some(#each_ident)),
                    None => {
                        let mut c = #container_ident::new();
                        c.extend(Some(#each_ident));
                        self.#ident = Some(c);
                    }
                }
            };

            let check_stmt = each.callable.as_ref().map(|callable| {
                let check_ty = quote! { &dyn Fn(&(#inner_args)) -> bool };

                quote! {
                    let check: #check_ty = &#callable;

                    if !check(&#each_ident) {
                        return Err("Provided value is not valid".into());
                    }
                }
            });

            if check_stmt.is_some() && check_stmt.is_none() {
                return_ty = quote! { ::std::result::Result<#return_ty, ::std::boxed::Box<dyn ::std::error::Error>> };
            }

            if check_stmt.is_some() && check_stmt.is_none() {
                return_val = quote! { Ok(#return_val) };
            }

            let setter = quote! {
                #visibility fn #each_ident(mut self, #each_ident: (#inner_args)) -> #return_ty {
                    #check_stmt

                    #update_stmt

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
            } else if required || !attrs.props.skip {
                builder_setters.push(setter);
            }
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
