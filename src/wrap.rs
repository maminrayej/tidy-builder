#[rustfmt::skip]
pub fn wrapped_in<'a>(wrapper: &'a syn::Type, wrapper_name: Option<&str>) -> Option<&'a syn::Type> {
    if let syn::Type::Path(syn::TypePath { path ,.. }) = wrapper {
        if let Some(wrapper_name) = wrapper_name {
            if path.segments[0].ident != wrapper_name {
                return None;
            }
        }

        return match &path.segments[0].arguments {
            syn::PathArguments::None => None,
            syn::PathArguments::Parenthesized(_) => None,
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

// Only `Type::Path` are supported here. These types have the form: segment0::segment1::segment2.
// Currently this method only detects whether the type is an `Option` if it's written as `Option<_>`.
//
// TODO: We could also support:
//      * ::std::option::Option
//      * std::option::Option
//
// # Arguments
// * `ty`: The type to check whether it's an `Option` or not.
//
// # Returns
// * `Some`: Containing the type inside `Option`. For example calling this function
//           on `Option<T>` returns `Some(T)`.
// * `None`: If the type is not option.
#[rustfmt::skip]
pub fn is_option(ty: &syn::Type) -> Option<&syn::Type> {
    wrapped_in(ty, Some("Option"))
}
