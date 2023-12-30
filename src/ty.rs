pub fn wrapped_in<'a>(
    ty: &'a syn::Type,
    wrapper: Option<&str>,
) -> Option<&'a syn::AngleBracketedGenericArguments> {
    let syn::Type::Path(syn::TypePath { path, .. }) = ty else {
        return None;
    };

    if let Some(last_segment) = path.segments.last() {
        if wrapper
            .map(|name| last_segment.ident == name)
            .unwrap_or(true)
        {
            if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                return Some(args);
            }
        }
    }

    return None;
}

pub fn wrapped_in_option<'a>(ty: &'a syn::Type) -> Option<&'a syn::Type> {
    let inner = wrapped_in(ty, Some("Option"))?;

    let inner_ty = &inner.args[0];

    if let syn::GenericArgument::Type(ty) = inner_ty {
        Some(ty)
    } else {
        None
    }
}
