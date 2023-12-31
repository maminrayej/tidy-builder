mod attrs;
mod ty;

use std::collections::HashMap;

use quote::quote;
use syn::spanned::Spanned;

macro_rules! error {
    ($src: expr, $msg: expr) => {
        return Err(syn::Error::new($src.span(), $msg));
    };
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let result = match &ast.data {
        syn::Data::Struct(s) => for_struct(&ast, s),
        syn::Data::Enum(e) => for_enum(&ast, e),
        syn::Data::Union(_) => for_union(&ast),
    };

    let token_stream = match result {
        Ok(output) => output,
        Err(error) => error.into_compile_error(),
    };

    token_stream.into()
}

fn for_struct(
    ast: &syn::DeriveInput,
    struct_data: &syn::DataStruct,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let fields_cnt = struct_data.fields.len();
    let mut attr_map = HashMap::with_capacity(fields_cnt);
    for field in &struct_data.fields {
        let attrs = attrs::parse_attrs(field)?;

        attr_map.insert(field, attrs);
    }

    let struct_ident = &ast.ident;
    let builder_ident = quote::format_ident!("{}Builder", struct_ident);

    /* generic parameters of the struct */
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let lifetime_params: Vec<_> = ast.generics.lifetimes().collect();
    let lifetime_names: Vec<_> = lifetime_params.iter().map(|p| p.lifetime.clone()).collect();
    let const_params: Vec<_> = ast.generics.const_params().collect();
    let const_names: Vec<_> = const_params.iter().map(|p| p.ident.clone()).collect();
    let type_params: Vec<_> = ast.generics.type_params().collect();
    let type_names: Vec<_> = type_params.iter().map(|p| p.ident.clone()).collect();

    let mut builder_moves = Vec::with_capacity(fields_cnt);
    let mut builder_fields = Vec::with_capacity(fields_cnt);
    let mut builder_const_names = Vec::with_capacity(fields_cnt);
    let mut builder_const_params = Vec::with_capacity(fields_cnt);
    let mut builder_all_false = Vec::with_capacity(fields_cnt);

    let mut builder_inits = Vec::with_capacity(fields_cnt);
    let mut builder_setters = Vec::with_capacity(fields_cnt);
    let mut builder_impls = Vec::with_capacity(fields_cnt);

    let mut is_builder_async = false;

    for field in &struct_data.fields {
        let ident = field.ident.as_ref().unwrap();
        let attrs = &attr_map[&field];

        let optional = ty::wrapped_in_option(&field.ty);
        let required = optional.is_none() && !attrs.has_value();

        builder_moves.push(quote! { #ident: self.#ident });

        if required {
            let req_param_name = quote::format_ident!("REQ_{}", ident.to_string().to_uppercase());

            builder_const_names.push(quote! { #req_param_name });
            builder_const_params.push(quote! { const #req_param_name: bool });
            builder_all_false.push(quote! { false });
        }
    }

    /* generate the builder struct definition */
    let mut req_index = 0;
    for field in &struct_data.fields {
        let ident = field.ident.as_ref().unwrap();
        let attrs = &attr_map[&field];

        let optional = ty::wrapped_in_option(&field.ty);
        let required = optional.is_none() && !attrs.has_value();

        /* validate the specified attributes */
        if optional.is_some() {
            if attrs.value.is_some() {
                error!(field.ty, "optional field cannot have default value");
            }
            if attrs.setter.once {
                error!(field.ty, "once cannot be applied to optional fields");
            }
        } else if required {
            if attrs.setter.skip {
                error!(field.span(), "cannot skip a required field");
            }
        } else {
            if attrs.setter.once {
                error!(field.span(), "once cannot be applied to fields with values");
            }
        }

        let ty = optional.unwrap_or(&field.ty);

        builder_fields.push(quote! { #ident: ::std::option::Option<#ty> });
        builder_inits.push({
            let init = attrs
                .value()
                .map(|value| {
                    let check_stmt = attrs.check().map(|func| {
                        let (func_name, is_await) = func.to_token_parts();

                        if is_await.is_some() {
                            is_builder_async = true;
                        }

                        quote! { (#func_name)(#ident)#is_await?; }
                    });

                    quote! {
                        {
                            let #ident = #value;
                            #check_stmt
                            Some(#ident)
                        }
                    }
                })
                .unwrap_or(quote! { None });

            quote! { #ident: #init }
        });

        /* generate setters and impl blocks */
        let input_ty = if attrs.setter.into {
            quote! { impl Into<#ty> }
        } else {
            quote! { #ty }
        };

        let before_names = builder_const_names.iter().take(req_index);
        let after_names = builder_const_names.iter().skip(req_index + 1);
        let mut return_ty = if required {
            quote! { #builder_ident<#(#lifetime_names,)* #(#type_names,)* #(#const_names,)* #(#before_names,)* true, #(#after_names,)*> }
        } else {
            quote! { Self }
        };

        let mut return_val = if required {
            quote! {
                #builder_ident {
                    #(#builder_moves,)*
                }
            }
        } else {
            quote! { self }
        };
        
        let init_stmt = attrs
            .setter
            .into
            .then_some(Some(quote! { let #ident = #ident.into(); }));

        let mut is_setter_async = false;
        let check_stmt = attrs.check().map(|func| {
            let (func_name, is_await) = func.to_token_parts();

            if is_await.is_some() {
                is_setter_async = true;
            }
            
            quote! { (#func_name)(#ident)#is_await?; }
        });
        if check_stmt.is_some() {
            return_val = quote! { Ok(#return_val) };
            return_ty = quote! { ::std::result::Result<#return_ty, ::std::boxed::Box<dyn ::std::error::Error>> };
        }

        let assign_stmt = quote! { self.#ident = Some(#ident); };

        let is_setter_async = is_setter_async.then_some(quote! {async});

        let setter = quote! {
            #is_setter_async fn #ident(mut self, #ident: #input_ty) -> #return_ty {
                #init_stmt
                #check_stmt
                #assign_stmt
                #return_val
            }
        };
        
        if required && attrs.setter.once {
            let before_names = builder_const_names.iter().take(req_index);
            let after_names = builder_const_names.iter().skip(req_index + 1);
            let before_params = builder_const_params.iter().take(req_index);
            let after_params = builder_const_params.iter().skip(req_index + 1);

            builder_impls.push(quote! {
                impl<#(#lifetime_params,)* #(#type_params,)* #(#const_params,)* #(#before_params,)* #(#after_params,)*>
                #builder_ident<#(#lifetime_names,)* #(#type_names,)* #(#const_names,)* #(#before_names,)* false, #(#after_names,)*>
                    #where_clause
                {
                    #setter
                }
            });
        } else if required || !attrs.setter.skip {
            builder_setters.push(setter);
        }

        if required {
            req_index += 1;
        }
    }

    let is_builder_async = is_builder_async.then_some(quote! { async });

    Ok(quote! {
        struct #builder_ident<
            #(#lifetime_params,)*
            #(#type_params,)*
            #(#const_params,)*
            #(#builder_const_params,)*
        > {
            #(#builder_fields),*
        }

        impl #impl_generics #struct_ident #ty_generics  #where_clause {
            pub #is_builder_async fn builder() -> #builder_ident<#(#lifetime_names,)*
                                                                #(#type_names,)*
                                                                #(#const_names,)*
                                                                #(#builder_all_false,)*>
            {
                #builder_ident {
                    #(#builder_inits,)*
                }
            }
        }

        impl<#(#lifetime_params,)*
            #(#type_params,)* 
            #(#const_params,)* 
            #(#builder_const_params,)*>
        #builder_ident<#(#lifetime_names,)*
                    #(#type_names,)* 
                    #(#const_names,)* 
                    #(#builder_const_names,)*>
        #where_clause
        {
            #(#builder_setters)*
        }
    })
}

fn for_enum(
    ast: &syn::DeriveInput,
    enum_data: &syn::DataEnum,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    todo!()
}

fn for_union(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    error!(ast, "Unions are not supported");
}
