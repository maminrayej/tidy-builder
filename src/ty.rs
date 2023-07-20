pub fn wrapped_in_option<'a>(ty: &'a syn::Type) -> Option<&'a syn::AngleBracketedGenericArguments> {
    let syn::Type::Path(syn::TypePath { path, .. }) = ty else { return None; };

    let args = (&path.segments)
        .last()
        .filter(|seg| seg.ident == "Option")
        .map(|seg| &seg.arguments);

    if let Some(syn::PathArguments::AngleBracketed(args)) = args {
        Some(args)
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub enum FieldType<'a> {
    Type(&'a syn::Type),
    Args(&'a syn::AngleBracketedGenericArguments),
}

impl<'a> quote::ToTokens for FieldType<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldType::Type(ty) => ty.to_tokens(tokens),
            FieldType::Args(args) => args.args.to_tokens(tokens),
        }
    }
}
