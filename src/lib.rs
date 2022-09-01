mod attr;
mod err;
mod gen;
mod wrap;

use std::collections::HashMap;

use attr::*;
use err::*;
use gen::*;
use wrap::*;

use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use syn::spanned::Spanned;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match ast.data {
        syn::Data::Struct(struct_t) => match struct_t.fields {
            syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
                let fields = named;
                let struct_ident = ast.ident.clone();

                // Map each field to its parsed attributes.
                let mut f_attrs: HashMap<&syn::Field, FieldAttrs> =
                    HashMap::with_capacity(fields.len());

                for field in &fields {
                    let attrs = match parse_attrs(field) {
                        Ok(attrs) => attrs,
                        Err(err) => {
                            return err.into();
                        }
                    };

                    f_attrs.insert(field, attrs);
                }

                // In the definition below, the boundary of each value is depicted.
                //
                // impl<T: std::fmt::Debug> Foo<T> where T: std::fmt::Display
                //     --------------------    --- --------------------------
                //              0               1               2
                //
                //  0: impl_generics
                //  1: ty_generics
                //  2: where_clause
                let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

                let builder_ident =
                    syn::Ident::new(&format!("{struct_ident}Builder"), struct_ident.span());

                //--- Struct generic Parameters ---//
                let st_param_names = param_to_name(&ast.generics);
                // st_lt_pn: struct lifetime param names
                // st_ct_pn: struct const param names
                // st_ty_pn: struct type param names
                let (st_lt_pn, st_ct_pn, st_ty_pn) = split_param_names(st_param_names);

                let st_params: Vec<_> = ast.generics.params.iter().cloned().collect();
                // st_lt_p: struct lifetime params
                // st_ct_p: struct const params
                // st_ty_p: struct type params
                let (st_lt_p, st_ct_p, st_ty_p) = split_params(st_params);

                //--- Builder generic parameters ---//
                let mut required_fields = vec![];
                let mut optional_fields = vec![];
                let mut default_fields = vec![];
                for field in &fields {
                    let is_default = f_attrs[field].is_default().is_some();
                    let is_option = is_option(&field.ty).is_some();

                    if is_option {
                        optional_fields.push(field);
                    } else if is_default {
                        default_fields.push(field);
                    } else {
                        required_fields.push(field);
                    }
                }

                // Contains all the builder parameters as `false`.
                // So it helps to create:
                //      `Builder<false, false, false>`.
                let mut all_false = vec![];

                // Contains all the builder parameters as `true`.
                // So it helps to create:
                //      `Builder<true, true, true>`.
                let mut all_true = vec![];

                // Contains the names of all builder parameters
                // So it helps to create:
                //      `Builder<P0, P1, P2>`.
                let mut b_ct_pn = vec![];

                // Contains all builder parameters
                // So it helps to create:
                //      `Builder<const P0: bool, const P1: bool, const P2: bool>`.
                let mut b_ct_p = vec![];

                // Contains all the fields of the builder.
                // For example if the struct is:
                //      struct MyStruct {
                //          foo: Option<String>,
                //          bar: usize
                //      }
                // The fields of the builder gonna be:
                //      struct MyStructBuilder {
                //          foo: Option<String>,
                //          bar: Option<usize>
                //      }
                let mut b_fields = vec![];

                // Contains all the initializers of the builder struct.
                // For example for the builder on the comment above it's going to be:
                //      MyStructBuilder {
                //          foo: None,
                //          bar: None
                //      }
                let mut b_inits = vec![];

                // When we set the value of a required field, we must create the next state in the
                // state machine. For that matter, we need to move the fields from the previous state to the new one.
                // This field contains the moves of required fields.
                let mut req_moves = vec![];

                // When we reach the final state of the state machine and want to build the struct,
                // we will call `unwrap` on the required fields because we know they are not `None`.
                // For example:
                //      fn builder(self) -> MyStruct {
                //          MyStruct {
                //              foo: self.foo,
                //              bar: self.bar.unwrap()
                //          }
                //      }
                // This variable contains the unwraps of required fields.
                let mut req_unwraps = vec![];

                for (index, field) in required_fields.iter().enumerate() {
                    let field_ident = &field.ident;
                    let field_ty = &field.ty;
                    let ct_param_ident = syn::Ident::new(&format!("P{}", index), field.span());

                    b_fields.push(quote! { #field_ident: ::std::option::Option<#field_ty> });
                    b_inits.push(quote! { #field_ident: None });

                    req_moves.push(quote! { #field_ident: self.#field_ident });
                    req_unwraps.push(quote! { #field_ident: self.#field_ident.unwrap_unchecked() });

                    all_false.push(quote! { false });
                    all_true.push(quote! { true });
                    b_ct_pn.push(quote! { #ct_param_ident });
                    b_ct_p.push(quote! { const #ct_param_ident: bool });
                }

                let mut def_moves = vec![];
                for field in &default_fields {
                    let field_ident = &field.ident;
                    let field_ty = &field.ty;
                    let default_value = match f_attrs[field].is_default().unwrap() {
                        Some(value) => quote! { #value },
                        None => quote! { Default::default() },
                    };

                    b_fields.push(quote! { #field_ident: #field_ty });
                    b_inits.push(quote! { #field_ident: #default_value });

                    def_moves.push(quote! { #field_ident: self.#field_ident });
                }

                // When we set the value of an optional field, we must create the current state in the
                // state machine but set the optional field. For that matter,
                // we need to move the fields from the previous state to the new one.
                // This field contains the moves of optional fields.
                let mut opt_moves = vec![];

                for opt_field in &optional_fields {
                    let field_ident = &opt_field.ident;
                    let field_ty = &opt_field.ty;

                    opt_moves.push(quote! { #field_ident: self.#field_ident });

                    b_fields.push(quote! { #field_ident: #field_ty });
                    b_inits.push(quote! { #field_ident: None });
                }

                //--- State machine actions: Setters ---//

                // Setting the value of an optional field:
                let mut def_setters = vec![];
                for def_field in &default_fields {
                    let field_ident = &def_field.ident;
                    let field_ty = &def_field.ty;

                    let skip = f_attrs[def_field].skip();
                    if skip {
                        continue;
                    }

                    def_setters.push(
                        quote! {
                            pub fn #field_ident(mut self, #field_ident: #field_ty) ->
                                #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)*>
                            {
                                self.#field_ident = #field_ident;
                                self
                            }
                        }
                    );
                }

                let mut opt_setters = vec![];
                for opt_field in &optional_fields {
                    let field_ident = &opt_field.ident;
                    let field_ty = &opt_field.ty;
                    let inner_ty = is_option(field_ty).unwrap();

                    let attrs = match parse_attrs(opt_field) {
                        Ok(attrs) => attrs,
                        Err(err) => {
                            return err.into();
                        }
                    };

                    let skip = f_attrs[opt_field].skip();
                    if skip {
                        continue;
                    }

                    let repeated_attr = attrs.repeated();

                    // When we set an optional field, we stay in the same state.
                    // Therefore, we just need to set the value of the optional field.
                    let opt_setter = quote! {
                        pub fn #field_ident(mut self, #field_ident: #inner_ty) ->
                            #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)*>
                        {
                            self.#field_ident = Some(#field_ident);
                            self
                        }
                    };

                    if let Some(each) = repeated_attr {
                        let raw_ty = if let syn::Type::Path(type_path) = inner_ty {
                            &type_path.path.segments[0].ident
                        } else {
                            return BuilderError::UnsupportedType(field_ty.clone()).into();
                        };
                        let inner_inner_ty = wrapped_in(inner_ty, None);
                        let each_ident = syn::Ident::new(each.as_str(), opt_field.span());

                        opt_setters.push(
                            quote! {
                                pub fn #each_ident(mut self, #each_ident: #inner_inner_ty) ->
                                    #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)*>
                                {
                                    match self.#field_ident.as_mut() {
                                        Some(c) => c.extend(Some(#each_ident)),
                                        None => {
                                            let mut c = #raw_ty::new();
                                            c.extend(Some(#each_ident));
                                            self.#field_ident = Some(c);
                                        }
                                    }

                                    #builder_ident {
                                        #(#req_moves,)*
                                        #(#opt_moves,)*
                                    }
                                }
                            }
                        );

                        if field_ident.clone().unwrap() != each {
                            opt_setters.push(opt_setter);
                        }
                    } else {
                        opt_setters.push(opt_setter);
                    }
                }

                // Setting the value of a required field.
                let mut req_setters = vec![];
                for (index, req_field) in required_fields.iter().enumerate() {
                    let field_ident = &req_field.ident;
                    let field_ty = &req_field.ty;

                    let attrs = match parse_attrs(req_field) {
                        Ok(attrs) => attrs,
                        Err(err) => {
                            return err.into();
                        }
                    };

                    let skip = f_attrs[req_field].skip();
                    if skip {
                        return BuilderError::SkipRequired((*req_field).clone()).into();
                    }

                    let repeated_attr = attrs.repeated();

                    // When setting a required field, we need to move the other required fields
                    // into the new state. So we pick the moves before and after this field.
                    let before_req_moves = &req_moves[..index];
                    let after_req_moves = &req_moves[index + 1..];

                    // When setting a parameter to `true`, we need to copy the other parameter
                    // names. So we pick the parameter names before and after the parameter that
                    // corresponds to this required field.
                    let before_pn = &b_ct_pn[..index];
                    let after_pn = &b_ct_pn[index + 1..];

                    // When we set the value of a required field, we must change to a state in
                    // which the parameter corresponding to that field is set to `true`.
                    let req_setter = quote! {
                        pub fn #field_ident(self, #field_ident: #field_ty) ->
                            #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#before_pn,)* true, #(#after_pn,)* #(#st_ty_pn,)*>
                        {
                            #builder_ident {
                                #(#before_req_moves,)*
                                #field_ident: Some(#field_ident),
                                #(#after_req_moves,)*
                                #(#opt_moves,)*
                                #(#def_moves,)*
                            }
                        }
                    };

                    if let Some(each) = repeated_attr {
                        let raw_ty = if let syn::Type::Path(type_path) = field_ty {
                            &type_path.path.segments[0].ident
                        } else {
                            return BuilderError::UnsupportedType(field_ty.clone()).into();
                        };
                        let inner_ty = wrapped_in(field_ty, None);
                        let each_ident = syn::Ident::new(each.as_str(), req_field.span());

                        req_setters.push(
                            quote! {
                                pub fn #each_ident(mut self, #each_ident: #inner_ty) ->
                                    #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#before_pn,)* true, #(#after_pn,)* #(#st_ty_pn,)*>
                                {
                                    match self.#field_ident.as_mut() {
                                        Some(c) => c.extend(Some(#each_ident)),
                                        None => {
                                            let mut c = #raw_ty::new();
                                            c.extend(Some(#each_ident));
                                            self.#field_ident = Some(c);
                                        }
                                    }

                                    #builder_ident {
                                        #(#req_moves,)*
                                        #(#opt_moves,)*
                                    }
                                }
                            }
                        );

                        if field_ident.clone().unwrap() != each {
                            req_setters.push(req_setter);
                        }
                    } else {
                        req_setters.push(req_setter);
                    }
                }

                // HasProp1, HasProp2 trait definitions and their implementation
                // for the builder
                let mut constraint_traits = vec![];
                let mut constraint_traits_idents = vec![];
                for (field_idx, field) in required_fields.iter().enumerate() {
                    let field_name = field.ident.as_ref().unwrap().to_string();
                    let field_camel = field_name.to_case(Case::UpperCamel);
                    let trait_name = format_ident!("Has{}", field_camel);

                    // we need all the generic parameters except for the field's const bool one

                    // impl <...> #trait_name for Builder<...>
                    //        ^--left               right--^
                    let mut generic_const_pars_left = vec![];
                    let mut generic_const_pars_right = vec![];

                    for (const_par_idx, const_param_name) in b_ct_pn.iter().enumerate() {
                        if field_idx == const_par_idx {
                            generic_const_pars_right.push(quote! { true });
                        } else {
                            generic_const_pars_left.push(quote! { const #const_param_name : bool });
                            generic_const_pars_right.push(quote! { #const_param_name });
                        }
                    }

                    let mut error_message = None;
                    // this feature uses `#[rustc_on_unimplemented]` which is only available
                    // in a nightly compiler.
                    if cfg!(feature = "better_error") {
                        let message = format!("missing `{}`", &field_name);
                        let label = format!("provide `{}` before calling `.build()`", &field_name);
                        error_message = Some(quote! {
                            #[rustc_on_unimplemented(
                                message=#message,
                                label=#label,
                            )]
                        });
                    }

                    constraint_traits.push(quote! {
                        #error_message
                        trait #trait_name {}

                        impl<#(#st_lt_p,)* #(#st_ct_p,)* #(#generic_const_pars_left,)* #(#st_ty_p,)*>
                            #trait_name for
                            #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#generic_const_pars_right,)* #(#st_ty_pn,)* >
                            #where_clause { }
                    });

                    constraint_traits_idents.push(trait_name);
                }

                //--- Generating the builder ---//
                quote! {
                    // Definition of the builder struct.
                    pub struct #builder_ident<#(#st_lt_p,)* #(#st_ct_p,)* #(#b_ct_p,)* #(#st_ty_p,)*> #where_clause {
                        #(#b_fields),*
                    }

                    // An impl on the given struct to add the `builder` method to initialize the
                    // builder.
                    impl #impl_generics #struct_ident #ty_generics #where_clause {
                        pub fn builder() -> #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#all_false,)* #(#st_ty_pn,)*> {
                            #builder_ident {
                                #(#b_inits),*
                            }
                        }
                    }

                    // impl on the builder containing the setter methods.
                    impl<#(#st_lt_p,)* #(#st_ct_p,)* #(#b_ct_p,)* #(#st_ty_p,)*>
                        #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)* >
                        #where_clause
                    {
                        #(#opt_setters)*
                        #(#def_setters)*
                        #(#req_setters)*

                        fn build(self) -> #struct_ident #ty_generics
                            where Self: #(#constraint_traits_idents)+*
                        {
                            unsafe {
                                #struct_ident {
                                    #(#opt_moves,)*
                                    #(#def_moves,)*
                                    #(#req_unwraps,)*
                                }
                            }
                        }
                    }

                    #(#constraint_traits)*

                }
                .into()
            }
            syn::Fields::Unnamed(_) => BuilderError::UnnamedFields(struct_t.fields).into(),
            syn::Fields::Unit => BuilderError::UnitStruct(struct_t.fields).into(),
        },
        syn::Data::Enum(enum_t) => BuilderError::Enum(enum_t).into(),
        syn::Data::Union(union_t) => BuilderError::Union(union_t).into(),
    }
}
