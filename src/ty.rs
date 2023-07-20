pub fn wrapped_in<'a>(
    ty: &'a syn::Type,
    wrapper: Option<&str>,
) -> Option<&'a syn::AngleBracketedGenericArguments> {
    let syn::Type::Path(syn::TypePath { path, .. }) = ty else { return None; };

    let args = (&path.segments)
        .last()
        .filter(|seg| wrapper.is_none() || wrapper.is_some_and(|wrapper| seg.ident == wrapper))
        .map(|seg| &seg.arguments);

    if let Some(syn::PathArguments::AngleBracketed(args)) = args {
        Some(args)
    } else {
        None
    }
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
