#[derive(Debug)]
pub enum Error {
    Enum(syn::DataEnum),
    Union(syn::DataUnion),
    UnnamedFields(syn::Fields),
    UnitStruct(syn::Fields),
    NotMetaList(syn::Attribute),
    NotStrValue(syn::Lit),
    NotNameValue(syn::NestedMeta),
    UnexpectedLit(syn::Lit),
    NestedMetaList(syn::MetaList),
    UnknownAttr(syn::Meta),
    UnsupportedType(syn::Type),
    SkipRequired(syn::Field),
}

impl From<Error> for proc_macro::TokenStream {
    fn from(error: Error) -> proc_macro::TokenStream {
        match error {
            Error::Enum(enum_t) => {
                syn::Error::new_spanned(enum_t.enum_token, "Builder does not support enums")
                    .into_compile_error()
                    .into()
            }
            Error::Union(union_t) => {
                syn::Error::new_spanned(union_t.union_token, "Builder does not support unions")
                    .into_compile_error()
                    .into()
            }
            Error::UnnamedFields(fields) => {
                syn::Error::new_spanned(fields, "Builder does not support unnamed fields")
                    .into_compile_error()
                    .into()
            }
            Error::UnitStruct(fields) => {
                syn::Error::new_spanned(fields, "Builder does not support unit structs")
                    .into_compile_error()
                    .into()
            }
            Error::NotMetaList(attr) => {
                syn::Error::new_spanned(attr, "Provided attribute cannot be parsed as a meta list")
                    .into_compile_error()
                    .into()
            }
            Error::NotStrValue(lit) => syn::Error::new_spanned(lit, "Literal must be a string")
                .into_compile_error()
                .into(),
            Error::NotNameValue(nested_meta) => {
                syn::Error::new_spanned(nested_meta, "Provided nested meta is not a key value")
                    .into_compile_error()
                    .into()
            }
            Error::UnexpectedLit(lit) => {
                syn::Error::new_spanned(lit, "Not expected a literal inner meta")
                    .into_compile_error()
                    .into()
            }
            Error::NestedMetaList(meta_list) => {
                syn::Error::new_spanned(meta_list, "Nested meta list is not supported")
                    .into_compile_error()
                    .into()
            }
            Error::UnknownAttr(name_value) => {
                syn::Error::new_spanned(name_value, "Unknown attribute")
                    .into_compile_error()
                    .into()
            }
            Error::UnsupportedType(ty) => {
                syn::Error::new_spanned(ty, "Only segmented paths are supported")
                    .into_compile_error()
                    .into()
            }
            Error::SkipRequired(field) => {
                syn::Error::new_spanned(field, "Cannot skip a required field")
                    .into_compile_error()
                    .into()
            }
        }
    }
}
