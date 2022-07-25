mod error;

use error::BuilderError::*;

use quote::quote;
use syn::*;

fn is_wrapped_in(ty: Type, wrapper: &str) -> Option<Type> {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if segments[0].ident == wrapper {
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

                let mut builder_fields = vec![];
                let mut builder_inits = vec![];

                // TODO: Generate const generics for the builder.

                for field in fields {
                    let field_ident = field.ident.clone();
                    let field_type = field.ty.clone();
                    let inner_type = is_wrapped_in(field_type.clone(), "Option");
                    let is_optional = inner_type.is_some();

                    // --- Prepare fields of the builder struct:
                    // For each field that is not wrapped in `Option` in the original struct,
                    // we wrap it in `Option`, and any field that is already wrapped in `Option`,
                    // we leave it as is. For example:
                    // name: String        --> name: Option<String>
                    // age : Option<usize> --> age : Option<usize>
                    // We do this so we don't double wrap a field in `Option`.
                    let builder_field = if is_optional {
                        quote! { #field_ident: #field_type }
                    } else {
                        quote! { #field_ident: std::option::Option<#field_type> }
                    };
                    builder_fields.push(builder_field);

                    // --- Prepare builder initializers:
                    // For each field in the original struct, we will create a `builder_field`
                    // which we know in wrapped in `Option`. So we need to initialize it with
                    // `None`.
                    builder_inits.push(quote! { #field_ident: None });
                }

                quote! {
                    pub struct #builder_ident #ty_generics #where_clause {
                        #(#builder_fields),*
                    }

                    impl #impl_generics #struct_ident #ty_generics #where_clause {
                        pub fn builder() -> #builder_ident #ty_generics {
                            #builder_ident {
                                #(#builder_inits),*
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

