mod impl_constraint;
mod impl_init;
mod impl_setter;

use std::collections::HashMap;

use quote::{format_ident, quote};

use crate::attribute::{parse_attrs, FieldAttrs};
use crate::err::Error;
use crate::generics::{param_to_name, split_param_names, split_params, GenericParamName};
use crate::wrap::is_option;

pub struct Generator<'a> {
    f_attrs: HashMap<&'a syn::Field, FieldAttrs>,
    b_ident: syn::Ident,
    s_ident: syn::Ident,

    impl_generics: syn::ImplGenerics<'a>,
    ty_generics: syn::TypeGenerics<'a>,
    where_clause: Option<&'a syn::WhereClause>,

    st_lt_pn: Vec<GenericParamName>,
    st_ct_pn: Vec<GenericParamName>,
    st_ty_pn: Vec<GenericParamName>,

    st_lt_p: Vec<syn::GenericParam>,
    st_ct_p: Vec<syn::GenericParam>,
    st_ty_p: Vec<syn::GenericParam>,

    req_fields: Vec<&'a syn::Field>,
    opt_fields: Vec<&'a syn::Field>,
    def_fields: Vec<&'a syn::Field>,

    all_false: Vec<proc_macro2::TokenStream>,
    all_true: Vec<proc_macro2::TokenStream>,

    // b_ct_pn: builder const param names
    // b_ct_p:  builder const params
    b_ct_pn: Vec<proc_macro2::TokenStream>,
    b_ct_p: Vec<proc_macro2::TokenStream>,

    // b_fields: Contains fields of the builder
    // b_inits:  Contains initializtion code for fields of the builder
    b_fields: Vec<proc_macro2::TokenStream>,
    b_inits: Vec<proc_macro2::TokenStream>,

    // When we set the value of a required field, we must create the next state in the
    // state machine. For that matter, we need to move the fields from the previous state(previous struct) to the new one(new struct).
    // These variables contain the code to move the fields to the new state.
    req_moves: Vec<proc_macro2::TokenStream>,
    opt_moves: Vec<proc_macro2::TokenStream>,
    def_moves: Vec<proc_macro2::TokenStream>,

    // When we reach the final state of the state machine and want to build the struct,
    // we will call `unwrap` on the required fields because we know they are not `None`.
    // This variable contains the code to unwrap the required fields.
    req_unwraps: Vec<proc_macro2::TokenStream>,
}

impl<'a> Generator<'a> {
    pub fn new(ast: &'a syn::DeriveInput) -> Result<Self, Error> {
        match ast.data {
            syn::Data::Struct(ref struct_t) => match &struct_t.fields {
                syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
                    let fields = named;
                    let s_ident = ast.ident.clone();

                    // Map each field to its parsed attributes.
                    let mut f_attrs = HashMap::with_capacity(fields.len());
                    for field in fields {
                        let attrs = parse_attrs(field)?;

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

                    let b_ident = format_ident!("{}Builder", s_ident);

                    //--- Struct generic Parameters ---//
                    let st_param_names = param_to_name(&ast.generics);

                    // st_lt_pn: struct lifetime param names
                    // st_ct_pn: struct const param names
                    // st_ty_pn: struct type param names
                    let (st_lt_pn, st_ct_pn, st_ty_pn) = split_param_names(st_param_names);

                    // st_lt_p: struct lifetime params
                    // st_ct_p: struct const params
                    // st_ty_p: struct type params
                    let (st_lt_p, st_ct_p, st_ty_p) = split_params(ast.generics.params.iter());

                    // Split the struct fields since handling required, optional, and default fields is different.
                    let mut req_fields = vec![];
                    let mut opt_fields = vec![];
                    let mut def_fields = vec![];
                    for field in fields {
                        let is_default = f_attrs[field].is_default().is_some();
                        let is_option = is_option(&field.ty).is_some();

                        if is_option {
                            opt_fields.push(field);
                        } else if is_default {
                            def_fields.push(field);
                        } else {
                            req_fields.push(field);
                        }
                    }

                    let mut generator = Generator {
                        f_attrs,
                        b_ident,
                        s_ident,

                        impl_generics,
                        ty_generics,
                        where_clause,

                        st_lt_pn,
                        st_ct_pn,
                        st_ty_pn,
                        st_lt_p,
                        st_ct_p,
                        st_ty_p,

                        req_fields,
                        opt_fields,
                        def_fields,

                        all_false: vec![],
                        all_true: vec![],

                        b_ct_pn: vec![],
                        b_ct_p: vec![],
                        b_fields: vec![],
                        b_inits: vec![],

                        req_moves: vec![],
                        opt_moves: vec![],
                        def_moves: vec![],

                        req_unwraps: vec![],
                    };

                    generator.req_init();
                    generator.opt_init();
                    generator.def_init();

                    Ok(generator)
                }
                syn::Fields::Unnamed(_) => Err(Error::UnnamedFields(struct_t.fields.clone())),
                syn::Fields::Unit => Err(Error::UnitStruct(struct_t.fields.clone())),
            },
            syn::Data::Enum(ref enum_t) => Err(Error::Enum(enum_t.clone())),
            syn::Data::Union(ref union_t) => Err(Error::Union(union_t.clone())),
        }
    }

    pub fn generate(self) -> Result<proc_macro2::TokenStream, Error> {
        let req_setters = self.req_setters()?;
        let opt_setters = self.opt_setters()?;
        let def_setters = self.def_setters()?;

        let (constraint_traits, constraint_traits_idents) = self.constraints();

        let (
            b_ident,
            s_ident,
            all_false,
            impl_generics,
            ty_generics,
            where_clause,
            st_lt_pn,
            st_ct_pn,
            st_ty_pn,
            st_lt_p,
            st_ct_p,
            st_ty_p,
            _req_fields,
            _opt_fields,
            _def_fields,
            b_ct_pn,
            b_ct_p,
            b_fields,
            b_inits,
            _req_moves,
            opt_moves,
            def_moves,
            req_unwraps,
        ) = (
            self.b_ident,
            self.s_ident,
            self.all_false,
            self.impl_generics,
            self.ty_generics,
            self.where_clause,
            self.st_lt_pn,
            self.st_ct_pn,
            self.st_ty_pn,
            self.st_lt_p,
            self.st_ct_p,
            self.st_ty_p,
            self.req_fields,
            self.opt_fields,
            self.def_fields,
            self.b_ct_pn,
            self.b_ct_p,
            self.b_fields,
            self.b_inits,
            self.req_moves,
            self.opt_moves,
            self.def_moves,
            self.req_unwraps,
        );

        Ok(quote! {
            pub struct #b_ident<#(#st_lt_p,)* #(#st_ct_p,)* #(#b_ct_p,)* #(#st_ty_p,)*> #where_clause {
                #(#b_fields),*
            }

            impl #impl_generics #s_ident #ty_generics #where_clause {
                pub fn builder() -> #b_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#all_false,)* #(#st_ty_pn,)*> {
                    #b_ident {
                        #(#b_inits),*
                    }
                }
            }

            impl<#(#st_lt_p,)* #(#st_ct_p,)* #(#b_ct_p,)* #(#st_ty_p,)*>
                #b_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)* >
                #where_clause
            {
                #(#opt_setters)*
                #(#def_setters)*
                #(#req_setters)*

                fn build(self) -> #s_ident #ty_generics
                    where Self: #(#constraint_traits_idents)+*
                {
                    unsafe {
                        #s_ident {
                            #(#opt_moves,)*
                            #(#def_moves,)*
                            #(#req_unwraps,)*
                        }
                    }
                }
            }

            #(#constraint_traits)*
        })
    }
}
