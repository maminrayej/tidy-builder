use quote::quote;
use syn::spanned::Spanned;

use super::Generator;

impl<'a> Generator<'a> {
    // Iterates over required fields and initializes the generator.
    pub fn req_init(&mut self) {
        for (index, field) in self.req_fields.iter().enumerate() {
            let field_ident = &field.ident;
            let field_ty = &field.ty;
            let ct_param_ident = syn::Ident::new(&format!("P{}", index), field.span());

            // Wrap the type of the field in an `Option` to be able to set it to `None` at the beginning.
            self.b_fields
                .push(quote! { #field_ident: ::std::option::Option<#field_ty> });
            self.b_inits.push(quote! { #field_ident: None });

            // Create a const generic parameter for each required field in order to track whether it's been initialized or not.
            self.b_ct_p.push(quote! { const #ct_param_ident: bool });
            self.b_ct_pn.push(quote! { #ct_param_ident });

            self.all_false.push(quote! { false });

            self.req_moves
                .push(quote! { #field_ident: self.#field_ident });
            self.req_unwraps
                .push(quote! { #field_ident: self.#field_ident.unwrap_unchecked() });
        }
    }

    // Iterates over optional fields and initializes the generator.
    pub fn opt_init(&mut self) {
        for opt_field in &self.opt_fields {
            let field_ident = &opt_field.ident;
            let field_ty = &opt_field.ty;

            // Since this field is already `Option`, we don't wrap it in `Option` again.
            self.b_fields.push(quote! { #field_ident: #field_ty });
            self.b_inits.push(quote! { #field_ident: None });

            self.opt_moves
                .push(quote! { #field_ident: self.#field_ident });
        }
    }

    // Iterates over default fields and initializes the generator.
    pub fn def_init(&mut self) {
        for field in &self.def_fields {
            let field_ident = &field.ident;
            let field_ty = &field.ty;

            let default_value = match self.f_attrs[field].is_default().unwrap() {
                Some(value) => quote! { #value },
                None => quote! { ::std::default::Default::default() },
            };

            // No need to wrap a default field in an `Option` since we have its initialization value.
            self.b_fields.push(quote! { #field_ident: #field_ty });
            self.b_inits.push(quote! { #field_ident: #default_value });

            self.def_moves
                .push(quote! { #field_ident: self.#field_ident });
        }
    }
}
