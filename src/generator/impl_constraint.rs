use convert_case::{Case, Casing};
use quote::{format_ident, quote};

use super::Generator;

impl<'a> Generator<'a> {
    pub fn constraints(&self) -> (Vec<proc_macro2::TokenStream>, Vec<syn::Ident>) {
        let mut constraint_traits = vec![];
        let mut constraint_traits_idents = vec![];
        for (field_idx, field) in self.req_fields.iter().enumerate() {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let field_camel = field_name.to_case(Case::UpperCamel);
            let trait_name = format_ident!("Has{}", field_camel);

            // we need all the generic parameters except for the field's const bool one

            // impl <...> #trait_name for Builder<...>
            //        ^--left               right--^
            let mut generic_const_pars_left = vec![];
            let mut generic_const_pars_right = vec![];

            for (const_par_idx, const_param_name) in self.b_ct_pn.iter().enumerate() {
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

            // Define these to be able to interpolate in quote.
            let b_ident = &self.b_ident;

            let where_clause = &self.where_clause;

            let st_lt_pn = &self.st_lt_pn;
            let st_ct_pn = &self.st_ct_pn;
            let st_ty_pn = &self.st_ty_pn;

            let st_lt_p = &self.st_lt_p;
            let st_ct_p = &self.st_ct_p;
            let st_ty_p = &self.st_ty_p;

            constraint_traits.push(quote! {
                            #error_message
                            trait #trait_name {}
                            impl<#(#st_lt_p,)* #(#st_ct_p,)* #(#generic_const_pars_left,)* #(#st_ty_p,)*>
                                #trait_name for
                                #b_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#generic_const_pars_right,)* #(#st_ty_pn,)* >
                                #where_clause { }
                        });

            constraint_traits_idents.push(trait_name);
        }

        (constraint_traits, constraint_traits_idents)
    }
}
