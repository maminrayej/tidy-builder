use quote::quote;
use syn::spanned::Spanned;

use super::Generator;

impl<'a> Generator<'a> {
    pub fn req_init(&mut self) {
        for (index, field) in self.req_fields.iter().enumerate() {
            let field_ident = &field.ident;
            let field_ty = &field.ty;
            let ct_param_ident = syn::Ident::new(&format!("P{}", index), field.span());

            self.b_fields
                .push(quote! { #field_ident: ::std::option::Option<#field_ty> });
            self.b_inits.push(quote! { #field_ident: None });

            self.b_ct_p.push(quote! { const #ct_param_ident: bool });
            self.b_ct_pn.push(quote! { #ct_param_ident });

            self.all_false.push(quote! { false });
            self.all_true.push(quote! { true });

            self.req_moves
                .push(quote! { #field_ident: self.#field_ident });
            self.req_unwraps
                .push(quote! { #field_ident: self.#field_ident.unwrap_unchecked() });
        }
    }

    pub fn opt_init(&mut self) {
        for opt_field in &self.opt_fields {
            let field_ident = &opt_field.ident;
            let field_ty = &opt_field.ty;

            self.b_fields.push(quote! { #field_ident: #field_ty });
            self.b_inits.push(quote! { #field_ident: None });

            self.opt_moves
                .push(quote! { #field_ident: self.#field_ident });
        }
    }

    pub fn def_init(&mut self) {
        for field in &self.def_fields {
            let field_ident = &field.ident;
            let field_ty = &field.ty;

            let default_value = match self.f_attrs[field].is_default().unwrap() {
                Some(value) => quote! { #value },
                None => quote! { Default::default() },
            };

            self.b_fields.push(quote! { #field_ident: #field_ty });
            self.b_inits.push(quote! { #field_ident: #default_value });

            self.def_moves
                .push(quote! { #field_ident: self.#field_ident });
        }
    }
}
