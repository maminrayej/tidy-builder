use crate::err::Error;

// Some types wrap around another type(their inner type). For example `Vec` wraps around `T` so does `Option`.
// This function returns the inner type of a wrapper type, if its name is equal to the provided `wrapper_name`.
//
// For example calling:
//      wrapped_in(Vec<T>, Some("Vec"))
// will return `T`. But, calling
//      wrapped_in(Vec<T>, Some("Option"))
// will return `None` since the name does not match.
//
// If we only care about the inner type, we can set `wrapper_name` to `None`.
#[rustfmt::skip]
pub fn wrapped_in<'a>(wrapper: &'a syn::Type, wrapper_name: Option<&str>) -> Option<&'a syn::Type> {
    if let syn::Type::Path(syn::TypePath { path ,.. }) = wrapper {
        if let Some(wrapper_name) = wrapper_name {
            if path.segments[0].ident != wrapper_name {
                return None;
            }
        }

        return match &path.segments[0].arguments {
            syn::PathArguments::None | syn::PathArguments::Parenthesized(_) => None,
            syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }) => {
                if let syn::GenericArgument::Type(inner_ty) = &args[0] {
                    Some(inner_ty)
                } else {
                    None
                }
            }
        };
    }

    None
}

pub fn type_ident(wrapper: &syn::Type) -> Result<&syn::Ident, Error> {
    if let syn::Type::Path(type_path) = wrapper {
        Ok(&type_path.path.segments[0].ident)
    } else {
        Err(Error::UnsupportedType(wrapper.clone()))
    }
}

// Returns inner type of an `Option` and `None` if type is not an `Option`.
#[rustfmt::skip]
pub fn is_option(ty: &syn::Type) -> Option<&syn::Type> {
    wrapped_in(ty, Some("Option"))
}
