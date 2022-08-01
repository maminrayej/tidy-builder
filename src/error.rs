pub enum BuilderError {
    Enum(syn::DataEnum),
    Union(syn::DataUnion),
    UnnamedFields(syn::Fields),
    UnitStruct(syn::Fields),
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
        }
    }
}
