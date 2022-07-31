mod error;

use error::BuilderError::*;

use proc_macro2::{Punct, Spacing, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::*;

// # Returns
// * `Some`: Containing the type inside `Option`. For example calling this function
//           on `Option<T>` returns `Some(T)`.
// * `None`: If the type is not option.
fn is_option(ty: &Type) -> Option<Type> {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if segments[0].ident == "Option" {
            return match &segments[0].arguments {
                PathArguments::None => None,
                PathArguments::Parenthesized(_) => None,
                PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
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

#[derive(Clone)]
enum GenParamName {
    Type(Ident),
    Lifetime(Lifetime),
    Const(Ident),
}

impl ToTokens for GenParamName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            GenParamName::Type(ty) => tokens.append(ty.clone()),
            GenParamName::Lifetime(lt) => {
                let mut apostrophe = Punct::new('\'', Spacing::Joint);
                apostrophe.set_span(lt.apostrophe);
                tokens.append(apostrophe);
                lt.ident.to_tokens(tokens);
            }
            GenParamName::Const(c) => tokens.append(c.clone()),
        }
    }
}

fn gen_params_into_names(generics: &Generics) -> Vec<GenParamName> {
    generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(ty) => GenParamName::Type(ty.ident.clone()),
            GenericParam::Lifetime(lt) => GenParamName::Lifetime(lt.lifetime.clone()),
            GenericParam::Const(c) => GenParamName::Const(c.ident.clone()),
        })
        .collect()
}

fn split_generic_params_names(
    generic_params: Vec<GenParamName>,
) -> (Vec<GenParamName>, Vec<GenParamName>, Vec<GenParamName>) {
    let mut lifetimes = vec![];
    let mut consts = vec![];
    let mut types = vec![];

    for param in generic_params {
        match param {
            GenParamName::Lifetime(_) => lifetimes.push(param.clone()),
            GenParamName::Const(_) => consts.push(param.clone()),
            GenParamName::Type(_) => types.push(param.clone()),
        }
    }

    (lifetimes, consts, types)
}

fn split_generic_params(
    generic_params: Vec<GenericParam>,
) -> (Vec<GenericParam>, Vec<GenericParam>, Vec<GenericParam>) {
    let mut lifetimes = vec![];
    let mut consts = vec![];
    let mut types = vec![];

    for param in generic_params {
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
                let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

                let builder_ident = Ident::new(
                    format!("{}Builder", struct_ident).as_str(),
                    struct_ident.span(),
                );

                let mut required_fields = vec![];
                let mut optional_fields = vec![];

                let mut b_fields = vec![];
                let mut b_inits = vec![];
                for field in fields.iter() {
                    let field_ident = &field.ident;
                    let field_ty = &field.ty;
                    let inner_ty = is_option(field_ty);
                    let is_option = inner_ty.is_some();

                    if is_option {
                        optional_fields.push(field);
                    } else {
                        required_fields.push(field);
                    }

                    b_fields.push(if is_option {
                        quote! { #field_ident: #field_ty }
                    } else {
                        quote! { #field_ident: ::std::option::Option<#field_ty> }
                    });

                    b_inits.push(quote! { #field_ident: None });
                }

                // Struct generic Parameters
                let struct_gen_param_names = gen_params_into_names(&ast.generics);
                let (st_lt_pn, st_ct_pn, st_ty_pn) =
                    split_generic_params_names(struct_gen_param_names);

                let struct_gen_params: Vec<_> = ast.generics.params.iter().cloned().collect();
                let (st_lt_p, st_ct_p, st_ty_p) = split_generic_params(struct_gen_params);

                // Builder generic parameters
                let mut all_false = vec![]; // <false, false>
                let mut all_true = vec![]; // <false, false>
                let mut b_ct_pn = vec![]; // <P0, P1>
                let mut b_ct_p = vec![]; // <const P0: bool, const P1: bool>
                for (index, field) in required_fields.iter().enumerate() {
                    // let field_ident = &field.ident;
                    // let field_ty = &field.ty;
                    let const_param_ident = Ident::new(&format!("P{}", index), field.span());

                    all_false.push(quote! { false });
                    all_true.push(quote! { true });
                    b_ct_pn.push(quote! { #const_param_ident });
                    b_ct_p.push(quote! { const #const_param_ident: bool });
                }

                // Self setters
                let mut req_self_setters = vec![];
                for req_fields in &required_fields {
                    let field_ident = &req_fields.ident;
                    req_self_setters.push(quote! { #field_ident: self.#field_ident });
                }

                let mut opt_self_setters = vec![];
                for opt_field in &optional_fields {
                    let field_ident = &opt_field.ident;
                    opt_self_setters.push(quote! { #field_ident: self.#field_ident });
                }

                // Setters
                // let mut req_setters = vec![];
                let mut opt_setters = vec![];

                for (index, opt_field) in optional_fields.iter().enumerate() {
                    let field_ident = &opt_field.ident;
                    let field_ty = &opt_field.ty;   
                    let inner_ty = is_option(field_ty).unwrap();

                    let before_self_setters = &opt_self_setters[..index];
                    let after_self_setters = &opt_self_setters[index + 1..];

                    opt_setters.push(
                        quote! {
                            pub fn #field_ident(self, #field_ident: #inner_ty) -> 
                                #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)*> 
                            {
                                #builder_ident {
                                    #(#before_self_setters,)*
                                    #field_ident: Some(#field_ident),
                                    #(#after_self_setters,)*
                                    #(#req_self_setters,)*
                                }
                            }
                        }
                    );
                }

                let mut req_setters = vec![];

                for (index, req_field) in required_fields.iter().enumerate() {
                    let field_ident = &req_field.ident;
                    let field_ty = &req_field.ty;   

                    let before_self_setters = &req_self_setters[..index];
                    let after_self_setters = &req_self_setters[index + 1..];

                    let before_pn = &b_ct_pn[..index];
                    let after_pn = &b_ct_pn[index + 1..];

                    req_setters.push(
                        quote! {
                            pub fn #field_ident(self, #field_ident: #field_ty) -> 
                                #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#before_pn,)* true, #(#after_pn,)* #(#st_ty_pn,)*> 
                            {
                                #builder_ident {
                                    #(#before_self_setters,)*
                                    #field_ident: Some(#field_ident),
                                    #(#after_self_setters,)*
                                    #(#opt_self_setters,)*
                                }
                            }
                        }
                    );
                }

                // Builders
                let mut req_field_builders = vec![];
                for req_field in required_fields {
                    let field_ident = &req_field.ident;

                    req_field_builders.push(quote! { #field_ident: self.#field_ident.unwrap_unchecked() });
                }

                let mut opt_field_builders = vec![];
                for opt_field in optional_fields {
                    let field_ident = &opt_field.ident;

                    opt_field_builders.push(quote! { #field_ident: self.#field_ident });
                }

                quote! {
                    pub struct #builder_ident<#(#st_lt_p,)* #(#st_ct_p,)* #(#b_ct_p,)* #(#st_ty_p,)*> #where_clause {
                        #(#b_fields),*
                    }

                    impl #impl_generics #struct_ident #ty_generics #where_clause {
                        pub fn builder() -> #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#all_false,)* #(#st_ty_pn,)*> {
                            #builder_ident {
                                #(#b_inits),*
                            }
                        }
                    }

                    impl<#(#st_lt_p,)* #(#st_ct_p,)* #(#b_ct_p,)* #(#st_ty_p,)*> 
                        #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#b_ct_pn,)* #(#st_ty_pn,)* > 
                        #where_clause 
                    {
                        #(#opt_setters)* 
                        #(#req_setters)* 
                    }

                    impl<#(#st_lt_p,)* #(#st_ct_p,)* #(#st_ty_p,)*> 
                        #builder_ident<#(#st_lt_pn,)* #(#st_ct_pn,)* #(#all_true,)* #(#st_ty_pn,)* > 
                        #where_clause 
                    {
                        fn build(self) -> #struct_ident #ty_generics {
                            unsafe {
                                #struct_ident {
                                    #(#opt_field_builders,)*
                                    #(#req_field_builders,)*
                                }
                            }
                        }
                    }

                }
                .into()
            }
            Fields::Unnamed(_) => UnnamedFieldsErr(struct_t.fields).into(),
            Fields::Unit => UnitStructErr(struct_t.fields).into(),
        },
        Data::Enum(enum_t) => EnumErr(enum_t).into(),
        Data::Union(union_t) => UnionErr(union_t).into(),
    }
}
