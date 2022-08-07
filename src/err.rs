pub enum BuilderError {
    Enum(syn::DataEnum),
    Union(syn::DataUnion),
    UnnamedFields(syn::Fields),
    UnitStruct(syn::Fields),
    NotMetaList(syn::Attribute),
    UnexpectedName(syn::Ident, String),
    NonStrValue(syn::Lit),
    NotNameValue(syn::NestedMeta),
    EmptyInnerMeta(syn::NestedMeta),
    NotExpectedLit(syn::Lit),
    NestedMetaList(syn::MetaList),
    UnknownAttr(syn::Meta),
    UnsupportedType(syn::Type),
}

impl From<BuilderError> for proc_macro::TokenStream {
    fn from(error: BuilderError) -> proc_macro::TokenStream {
        match error {
            BuilderError::Enum(enum_t) => {
                syn::Error::new_spanned(enum_t.enum_token, "Builder does not support enums")
                    .into_compile_error()
                    .into()
            }
            BuilderError::Union(union_t) => {
                syn::Error::new_spanned(union_t.union_token, "Builder does not support unions")
                    .into_compile_error()
                    .into()
            }
            BuilderError::UnnamedFields(fields) => {
                syn::Error::new_spanned(fields, "Builder does not support unnamed fields")
                    .into_compile_error()
                    .into()
            }
            BuilderError::UnitStruct(fields) => {
                syn::Error::new_spanned(fields, "Builder does not support unit structs")
                    .into_compile_error()
                    .into()
            }
            BuilderError::NotMetaList(attr) => {
                syn::Error::new_spanned(attr, "Provided attribute cannot be parsed as a meta list")
                    .into_compile_error()
                    .into()
            }
            BuilderError::UnexpectedName(ident, expected) => {
                syn::Error::new_spanned(ident, format!("Expected: {}", expected).as_str())
                    .into_compile_error()
                    .into()
            }
            BuilderError::NonStrValue(lit) => {
                syn::Error::new_spanned(lit, "Literal must be a string")
                    .into_compile_error()
                    .into()
            }
            BuilderError::NotNameValue(nested_meta) => {
                syn::Error::new_spanned(nested_meta, "Provided nested meta is not a key value")
                    .into_compile_error()
                    .into()
            }
            BuilderError::EmptyInnerMeta(nested_meta) => syn::Error::new_spanned(
                nested_meta,
                "Nested meta is empty. It must at least contain one element.",
            )
            .into_compile_error()
            .into(),
            BuilderError::NotExpectedLit(lit) => {
                syn::Error::new_spanned(lit, "Not expected a literal inner meta")
                    .into_compile_error()
                    .into()
            }
            BuilderError::NestedMetaList(meta_list) => {
                syn::Error::new_spanned(meta_list, "Nested meta list is not supported")
                    .into_compile_error()
                    .into()
            }
            BuilderError::UnknownAttr(name_value) => {
                syn::Error::new_spanned(name_value, "Unknown attribute")
                    .into_compile_error()
                    .into()
            }
            BuilderError::UnsupportedType(ty) => {
                syn::Error::new_spanned(ty, "Only segmented paths are supported")
                    .into_compile_error()
                    .into()
            }
        }
    }
}
