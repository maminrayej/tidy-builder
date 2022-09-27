use quote::quote;
use syn::spanned::Spanned;

use super::Generator;
use crate::err::Error;
use crate::wrap::{is_option, wrapped_in};

impl<'a> Generator<'a> {
    pub fn req_setters(&self) -> Result<Vec<proc_macro2::TokenStream>, Error> {
        let mut req_setters = vec![];

        for (index, req_field) in self.req_fields.iter().enumerate() {
            let field_ident = &req_field.ident;
            let field_ty = &req_field.ty;

            if self.f_attrs[req_field].should_skip() {
                return Err(Error::SkipRequired((*req_field).clone()));
            }

            let repeated_attr = self.f_attrs[req_field].repeated();

            // When setting a required field, we need to move the other required fields
            // into the new state. So we pick the moves before and after this field.
            let before_req_moves = &self.req_moves[..index];
            let after_req_moves = &self.req_moves[index + 1..];

            // When setting a parameter to `true`, we need to copy the other parameter
            // names. So we pick the parameter names before and after the parameter that
            // corresponds to this required field.
            let before_pn = &self.b_ct_pn[..index];
            let after_pn = &self.b_ct_pn[index + 1..];

            // Define these to be able to interpolate in quote.
            let b_ident = &self.b_ident;

            let st_lt_pn = &self.st_lt_pn;
            let st_ct_pn = &self.st_ct_pn;
            let st_ty_pn = &self.st_ty_pn;

            let req_moves = &self.req_moves;
            let opt_moves = &self.opt_moves;
            let def_moves = &self.def_moves;

            // When we set the value of a required field, we must change to a state in
            // which the parameter corresponding to that field is set to `true`.
            let req_setter = quote! {
                pub fn #field_ident(self, #field_ident: #field_ty) ->
                    #b_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#before_pn,)* true, #(#after_pn,)* #(#st_ty_pn,)*>
                {
                    #b_ident {
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
                    // return Error::UnsupportedType(field_ty.clone()).into();
                    panic!("Unsupported type")
                };
                let inner_ty = wrapped_in(field_ty, None);
                let each_ident = syn::Ident::new(each.as_str(), req_field.span());

                req_setters.push(
                    quote! {
                        pub fn #each_ident(mut self, #each_ident: #inner_ty) ->
                            #b_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#before_pn,)* true, #(#after_pn,)* #(#st_ty_pn,)*>
                        {
                            match self.#field_ident.as_mut() {
                                Some(c) => c.extend(Some(#each_ident)),
                                None => {
                                    let mut c = #raw_ty::new();
                                    c.extend(Some(#each_ident));
                                    self.#field_ident = Some(c);
                                }
                            }
                            #b_ident {
                                #(#req_moves,)*
                                #(#opt_moves,)*
                                #(#def_moves,)*
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

        Ok(req_setters)
    }

    pub fn opt_setters(&self) -> Result<Vec<proc_macro2::TokenStream>, Error> {
        let mut opt_setters = vec![];

        for opt_field in &self.opt_fields {
            let field_ident = &opt_field.ident;
            let field_ty = &opt_field.ty;
            let inner_ty = is_option(field_ty).unwrap();

            if self.f_attrs[opt_field].should_skip() {
                continue;
            }

            let repeated_attr = self.f_attrs[opt_field].repeated();

            // Define these to be able to interpolate in quote.
            let b_ident = &self.b_ident;

            let b_ct_pn = &self.b_ct_pn;

            let st_lt_pn = &self.st_lt_pn;
            let st_ct_pn = &self.st_ct_pn;
            let st_ty_pn = &self.st_ty_pn;

            let opt_setter = quote! {
                pub fn #field_ident(mut self, #field_ident: #inner_ty) ->
                    #b_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)*>
                {
                    self.#field_ident = Some(#field_ident);
                    self
                }
            };

            if let Some(each) = repeated_attr {
                let raw_ty = if let syn::Type::Path(type_path) = inner_ty {
                    &type_path.path.segments[0].ident
                } else {
                    return Err(Error::UnsupportedType(field_ty.clone()));
                };
                let inner_inner_ty = wrapped_in(inner_ty, Some("Vec"));
                let each_ident = syn::Ident::new(each.as_str(), opt_field.span());

                opt_setters.push(quote! {
                    pub fn #each_ident(mut self, #each_ident: #inner_inner_ty) ->
                        #b_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)*>
                    {
                        match self.#field_ident.as_mut() {
                            Some(c) => c.extend(Some(#each_ident)),
                            None => {
                                let mut c = #raw_ty::new();
                                c.extend(Some(#each_ident));
                                self.#field_ident = Some(c);
                            }
                        }

                        self
                    }
                });

                if field_ident.clone().unwrap() != each {
                    opt_setters.push(opt_setter);
                }
            } else {
                opt_setters.push(opt_setter);
            }
        }

        Ok(opt_setters)
    }

    pub fn def_setters(&self) -> Result<Vec<proc_macro2::TokenStream>, Error> {
        let mut def_setters = vec![];

        for def_field in &self.def_fields {
            let field_ident = &def_field.ident;
            let field_ty = &def_field.ty;

            if self.f_attrs[def_field].should_skip() {
                continue;
            }

            // Define these to be able to interpolate in quote.
            let b_ident = &self.b_ident;

            let b_ct_pn = &self.b_ct_pn;

            let st_lt_pn = &self.st_lt_pn;
            let st_ct_pn = &self.st_ct_pn;
            let st_ty_pn = &self.st_ty_pn;

            def_setters.push(quote! {
                pub fn #field_ident(mut self, #field_ident: #field_ty) ->
                    #b_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)*>
                {
                    self.#field_ident = #field_ident;
                    self
                }
            });
        }

        Ok(def_setters)
    }
}
