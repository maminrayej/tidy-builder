// Sometimes we only need the name of a generic parameter.
// For example in `T: std::fmt::Display`, the whole thing is
// a generic parameter but we want to extract the `T` from it.
// Since we have three types of generic parameters, we need to
// distinguish between their names too.
//  * A `Type` is like `T: std::fmt::Display` from which we want the `T` which is the `Ident`.
//  * A `Lifetime` is like `'a: 'b` from which we want the `'a` which is the `Lifetime`.
//  * A `Const` is like `const N: usize` from which we want the `N` which is the `Ident`.
#[derive(Clone)]
pub enum GenericParamName {
    Type(syn::Ident),
    Lifetime(syn::Lifetime),
    Const(syn::Ident),
}

// We need this trait to be able to interpolate on a vector of `GenericParamName`.
impl quote::ToTokens for GenericParamName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            GenericParamName::Type(ty) => ty.to_tokens(tokens),
            GenericParamName::Lifetime(lt) => lt.to_tokens(tokens),
            GenericParamName::Const(ct) => ct.to_tokens(tokens),
        }
    }
}

// Extracts the name of each generic parameter in `generics`.
pub fn param_to_name(generics: &syn::Generics) -> Vec<GenericParamName> {
    generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(ty) => GenericParamName::Type(ty.ident.clone()),
            syn::GenericParam::Lifetime(lt) => GenericParamName::Lifetime(lt.lifetime.clone()),
            syn::GenericParam::Const(c) => GenericParamName::Const(c.ident.clone()),
        })
        .collect()
}

// Splits the generic parameter names into three categories.
pub fn split_param_names(
    param_names: Vec<GenericParamName>,
) -> (
    Vec<GenericParamName>, // Lifetime generic parameters
    Vec<GenericParamName>, // Const generic parameters
    Vec<GenericParamName>, // Type generic parameters
) {
    let mut lifetimes = vec![];
    let mut consts = vec![];
    let mut types = vec![];

    for param_name in param_names {
        match param_name {
            GenericParamName::Lifetime(_) => lifetimes.push(param_name.clone()),
            GenericParamName::Const(_) => consts.push(param_name.clone()),
            GenericParamName::Type(_) => types.push(param_name.clone()),
        }
    }

    (lifetimes, consts, types)
}

// Splits generic parameters into three categories.
pub fn split_params<'a>(
    params: impl Iterator<Item = &'a syn::GenericParam>,
) -> (
    Vec<syn::GenericParam>, // Lifetime generic parameters
    Vec<syn::GenericParam>, // Const generic parameters
    Vec<syn::GenericParam>, // Type generic parameters
) {
    let mut lifetimes = vec![];
    let mut consts = vec![];
    let mut types = vec![];

    for param in params {
        match param {
            syn::GenericParam::Lifetime(_) => lifetimes.push(param.clone()),
            syn::GenericParam::Const(_) => consts.push(param.clone()),
            syn::GenericParam::Type(_) => types.push(param.clone()),
        }
    }

    (lifetimes, consts, types)
}
