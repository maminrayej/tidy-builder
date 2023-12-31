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

    let mut builder_fields = Vec::with_capacity(fields_cnt);
    let mut builder_const_names = Vec::with_capacity(fields_cnt);
    let mut builder_const_params = Vec::with_capacity(fields_cnt);
    let mut builder_all_false = Vec::with_capacity(fields_cnt);

    let mut builder_inits = Vec::with_capacity(fields_cnt);

    let mut is_builder_async = false;

    /* generate the builder struct definition */
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

        if required {
            let req_param_name = quote::format_ident!("REQ_{}", ident.to_string().to_uppercase());

            builder_const_names.push(quote! { #req_param_name });
            builder_const_params.push(quote! { const #req_param_name: bool });
            builder_all_false.push(quote! { false });
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
