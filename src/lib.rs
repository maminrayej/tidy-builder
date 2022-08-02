//! The [Builder](`crate::Builder`) derive macro creates a compile-time correct builder.
//! It means that it only allows you to build the given struct as long as you provide a
//! value for all of its required fields.
//!
//! A field is interpreted as required if it's not wrapped in an `Option`.
//! Any field inside of an `Option` is not considered required in order to
//! build the given struct. For example in:
//! ```rust
//! pub struct MyStruct {
//!     foo: String,
//!     bar: Option<usize>,
//! }
//! ```
//! The `foo` field is required and `bar` is optional. **Note** that although
//! `std::option::Option` also referes to the same type, for now this macro doesn't
//! recongnize anything other than `Option`.
//!
//! The builder generated using the [Builder](`crate::Builder`) macro guarantees correctness
//! by encoding the initialized set using const generics. An example makes it clear. Let's assume
//! we have a struct that has two required fields and an optional one:
//! ```rust
//! pub struct MyStruct {
//!     req1: String,
//!     req2: String,
//!     opt1: Option<String>
//! }
//! ```
//! The generated builder will be:
//! ```rust
//! pub struct MyStructBuilder<const P0: bool, const P1: bool> {
//!     req1: Option<String>,
//!     req2: Option<String>,
//!     opt1: Option<String>,
//! }
//! ```
//! The `P0` indicates whether the first required parameter is initialized or not. And similarly,
//! the `P1` does the same thing for the second required parameter. The initial state of the
//! builder will be `MyStructBuilder<false, false>` and the first time a required field is
//! initialized, its corresponding const generic parameter will be set to true which indicates a
//! different state. Setting an optional value does not change the state and consequently keeps the
//! same const generic parameters. When the builder reaches the `MyStructBuilder<true, true>` and
//! only then you can call the `build` function on the builder.
//!
//! So the complete generated code for the given example struct is:
//! ```rust
//! pub struct MyStruct {
//!     req1: String,
//!     req2: String,
//!     opt1: Option<String>
//! }
//!
//! pub struct MyStructBuilder<const P0: bool, const P1: bool> {
//!     req1: Option<String>,
//!     req2: Option<String>,
//!     opt1: Option<String>,
//! }
//!
//! impl MyStruct {
//!     pub fn builder() -> MyStructBuilder<false, false> {
//!         MyStructBuilder {
//!             req1: None,
//!             req2: None,
//!             opt1: None,
//!         }
//!     }
//! }
//!
//! impl<const P0: bool, const P1: bool> MyStructBuilder<P0, P1> {
//!     pub fn req1(self, req1: String) -> MyStructBuilder<true, P1> {
//!         MyStructBuilder {
//!             req1: Some(req1),
//!             req2: self.req2,
//!             opt1: self.opt1,
//!         }
//!     }
//!
//!     pub fn req2(self, req2: String) -> MyStructBuilder<P0, true> {
//!         MyStructBuilder {
//!             req1: self.req1,
//!             req2: Some(req2),
//!             opt1: self.opt1,
//!         }
//!     }
//!
//!     pub fn opt1(self, opt1: String) -> MyStructBuilder<P0, P1> {
//!         MyStructBuilder {
//!             req1: self.req1,
//!             req2: self.req2,
//!             opt1: Some(opt1),
//!         }
//!     }
//! }
//!
//! impl MyStructBuilder<true, true> {
//!     pub fn build(self) -> MyStruct {
//!         unsafe {
//!             MyStruct {
//!                 req1: self.req1.unwrap_unchecked(),
//!                 req2: self.req2.unwrap_unchecked(),
//!                 opt1: self.opt1,
//!             }
//!         }
//!     }
//! }
//! ```

mod error;

use error::BuilderError::*;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::*;

// Only `Type::Path` are supported here. These types have the form: segment0::segment1::segment2.
// Currently this method only detects whether the type is an `Option` if it's written as `Option<_>`.
//
// TODO: We could also support:
//      * ::std::option::Option
//      * std::option::Option
//
// # Arguments
// * `ty`: The type to check whether it's an `Option` or not.
//
// # Returns
// * `Some`: Containing the type inside `Option`. For example calling this function
//           on `Option<T>` returns `Some(T)`.
// * `None`: If the type is not option.
#[rustfmt::skip]
fn is_option(ty: &Type) -> Option<Type> {
    // If `ty` is a `Type::Path`, it will contain one or more segments.
    // For example:
    //      std::option::Option
    //      ---  ------  ------
    //       s0    s1      s2
    // has three segments.
    if let Type::Path(TypePath { path: Path { segments, .. }, .. }) = ty {
        // Becuase we only look for a type like `Option<_>`, we only check the first segment.
        if segments[0].ident == "Option" {
            // A type can have zero or more arguments. In case of `Option<_>`, we expect
            // to see `AngleBracketed` arguments. So anything else cannot be an `Option`.
            return match &segments[0].arguments {
                PathArguments::None => None,
                PathArguments::Parenthesized(_) => None,
                PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
                    // We expect the argument to be a type. For example in `Option<String>`,
                    // The argument is a type and its `String`.
                    if let GenericArgument::Type(inner_ty) = &args[0] {
                        Some(inner_ty.clone())
                    } else {
                        None
                    }
                }
            };
        }
    }

    None
}

// Sometimes we only need the name of a generic parameter.
// For example in `T: std::fmt::Display`, the whole thing is
// a generic parameter but we want to extract the `T` from it.
// Since we have three types of generic parameters, we need to
// distinguish between their names too.
//  * A `Type` is like `T: std::fmt::Display` from which we want the `T` which is the `Ident`.
//  * A `Lifetime` is like `'a: 'b` from which we want the `'a` which is the `Lifetime`.
//  * A `Const` is like `const N: usize` from which we want the `N` which is the `Ident`.
#[derive(Clone)]
enum GenericParamName {
    Type(Ident),
    Lifetime(Lifetime),
    Const(Ident),
}

// We need this trait to be able to interpolate on a vector of `GenericParamName`.
impl ToTokens for GenericParamName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            GenericParamName::Type(ty) => ty.to_tokens(tokens),
            GenericParamName::Lifetime(lt) => lt.to_tokens(tokens),
            GenericParamName::Const(ct) => ct.to_tokens(tokens),
        }
    }
}

// Extracts the name of each generic parameter in `generics`.
fn param_to_name(generics: &Generics) -> Vec<GenericParamName> {
    generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(ty) => GenericParamName::Type(ty.ident.clone()),
            GenericParam::Lifetime(lt) => GenericParamName::Lifetime(lt.lifetime.clone()),
            GenericParam::Const(c) => GenericParamName::Const(c.ident.clone()),
        })
        .collect()
}

// Splits the generic parameter names into three categories.
fn split_param_names(
    param_names: Vec<GenericParamName>,
) -> (
    Vec<GenericParamName>, // Lifetime generic parameters
    Vec<GenericParamName>, // Const generic parameters
    Vec<GenericParamName>, // Type generic parameters
) {
    let mut lifetimes = vec![];
    let mut consts = vec![];
    let mut types = vec![];

    for param_name in param_names {
        match param_name {
            GenericParamName::Lifetime(_) => lifetimes.push(param_name.clone()),
            GenericParamName::Const(_) => consts.push(param_name.clone()),
            GenericParamName::Type(_) => types.push(param_name.clone()),
        }
    }

    (lifetimes, consts, types)
}

// Splits generic parameters into three categories.
fn split_params(
    params: Vec<GenericParam>,
) -> (
    Vec<GenericParam>, // Lifetime generic parameters
    Vec<GenericParam>, // Const generic parameters
    Vec<GenericParam>, // Type generic parameters
) {
    let mut lifetimes = vec![];
    let mut consts = vec![];
    let mut types = vec![];

    for param in params {
        match param {
            GenericParam::Lifetime(_) => lifetimes.push(param.clone()),
            GenericParam::Const(_) => consts.push(param.clone()),
            GenericParam::Type(_) => types.push(param.clone()),
        }
    }

    (lifetimes, consts, types)
}

#[proc_macro_derive(Builder)]
pub fn builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match ast.data {
        Data::Struct(struct_t) => match struct_t.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let fields = named;
                let struct_ident = ast.ident.clone();

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
                    Ident::new(&format!("{struct_ident}Builder"), struct_ident.span());

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
                let (optional_fields, required_fields): (Vec<_>, Vec<_>) = fields
                    .iter()
                    .partition(|field| is_option(&field.ty).is_some());

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
                    let ct_param_ident = Ident::new(&format!("P{}", index), field.span());

                    b_fields.push(quote! { #field_ident: ::std::option::Option<#field_ty> });
                    b_inits.push(quote! { #field_ident: None });

                    req_moves.push(quote! { #field_ident: self.#field_ident });
                    req_unwraps.push(quote! { #field_ident: self.#field_ident.unwrap_unchecked() });

                    all_false.push(quote! { false });
                    all_true.push(quote! { true });
                    b_ct_pn.push(quote! { #ct_param_ident });
                    b_ct_p.push(quote! { const #ct_param_ident: bool });
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
                let mut opt_setters = vec![];
                for opt_field in &optional_fields {
                    let field_ident = &opt_field.ident;
                    let field_ty = &opt_field.ty;
                    let inner_ty = is_option(field_ty).unwrap();

                    // When we set an optional field, we stay in the same state.
                    // Therefore, we just need to set the value of the optional field.
                    opt_setters.push(
                        quote! {
                            pub fn #field_ident(mut self, #field_ident: #inner_ty) ->
                                #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)*>
                            {
                                self.#field_ident = Some(#field_ident);
                                self
                            }
                        }
                    );
                }

                // Setting the value of a required field.
                let mut req_setters = vec![];
                for (index, req_field) in required_fields.iter().enumerate() {
                    let field_ident = &req_field.ident;
                    let field_ty = &req_field.ty;

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
                    req_setters.push(
                        quote! {
                            pub fn #field_ident(self, #field_ident: #field_ty) ->
                                #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#before_pn,)* true, #(#after_pn,)* #(#st_ty_pn,)*>
                            {
                                #builder_ident {
                                    #(#before_req_moves,)*
                                    #field_ident: Some(#field_ident),
                                    #(#after_req_moves,)*
                                    #(#opt_moves,)*
                                }
                            }
                        }
                    );
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
                        #(#req_setters)*
                    }

                    // impl block on a builder with all of its parameters set to true.
                    // Meaning it's in the final state and can actually build the given struct.
                    impl<#(#st_lt_p,)* #(#st_ct_p,)* #(#st_ty_p,)*>
                        #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#all_true,)* #(#st_ty_pn,)* >
                        #where_clause
                    {
                        fn build(self) -> #struct_ident #ty_generics {
                            unsafe {
                                #struct_ident {
                                    #(#opt_moves,)*
                                    #(#req_unwraps,)*
                                }
                            }
                        }
                    }

                }
                .into()
            }
            Fields::Unnamed(_) => UnnamedFields(struct_t.fields).into(),
            Fields::Unit => UnitStruct(struct_t.fields).into(),
        },
        Data::Enum(enum_t) => Enum(enum_t).into(),
        Data::Union(union_t) => Union(union_t).into(),
    }
}
