pub enum BuilderError {
    EnumErr(syn::DataEnum),
    UnionErr(syn::DataUnion),
    UnnamedFieldsErr(syn::Fields),
    UnitStructErr(syn::Fields),
}

impl Into<proc_macro::TokenStream> for BuilderError {
    fn into(self) -> proc_macro::TokenStream {
        match self {
            BuilderError::EnumErr(enum_t) => {
                syn::Error::new_spanned(enum_t.enum_token, "Builder does not support enums")
                    .into_compile_error()
                    .into()
            }
            BuilderError::UnionErr(union_t) => {
                syn::Error::new_spanned(union_t.union_token, "Builder does not support unions")
                    .into_compile_error()
                    .into()
            }
            BuilderError::UnnamedFieldsErr(fields) => {
                syn::Error::new_spanned(fields, "Builder does not support unnamed fields")
                    .into_compile_error()
                    .into()
            }
            BuilderError::UnitStructErr(fields) => {
                syn::Error::new_spanned(fields, "Builder does not support unit structs")
                    .into_compile_error()
                    .into()
            }
        }
    }
}
