use convert_case::{Case, Casing};
use quote::{format_ident, quote};

use super::Generator;

impl<'a> Generator<'a> {
    // Returns the traits guarding the `build` function.
    pub fn guards(&self) -> (Vec<proc_macro2::TokenStream>, Vec<syn::Ident>) {
        let mut guard_traits = vec![];
        let mut guard_trait_idents = vec![];

        // Generate a trait guard for each required field.
        for (field_idx, field) in self.req_fields.iter().enumerate() {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let field_camel = field_name.to_case(Case::UpperCamel);
            let trait_ident = format_ident!("Has{}", field_camel);

            let before_ct_pn = &self.b_ct_pn[0..field_idx];
            let after_ct_pn = &self.b_ct_pn[field_idx + 1..];

            let before_ct_p = &self.b_ct_p[0..field_idx];
            let after_ct_p = &self.b_ct_p[field_idx + 1..];

            // This feature uses `#[rustc_on_unimplemented]` which is only available
            // in a nightly compiler.
            let mut error_message = None;
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

            guard_traits.push(quote! {
                #error_message
                trait #trait_ident {}
                impl<#(#st_lt_p,)* #(#st_ct_p,)* #(#before_ct_p,)* #(#after_ct_p,)* #(#st_ty_p,)* >
                    #trait_ident for
                    #b_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#before_ct_pn,)* true, #(#after_ct_pn,)* #(#st_ty_pn,)* >
                    #where_clause { }
            });

            guard_trait_idents.push(trait_ident);
        }

        (guard_traits, guard_trait_idents)
    }
}
