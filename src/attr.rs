use crate::err::BuilderError;

pub enum FieldAttrKind {
    Repeat,
    Default,
}

pub enum FieldAttr {
    // #[builder(each = "each", new = "new", ext = "single|tuple")
    Repeat {
        each: String,
        new: String,
        ext: String,
    },
    // #[builder(default)]
    Default,
}

fn attr_kind(
    nested: &syn::punctuated::Punctuated<syn::NestedMeta, syn::token::Comma>,
) -> Result<FieldAttrKind, BuilderError> {
    match &nested[0] {
        syn::NestedMeta::Meta(meta) => match meta {
            syn::Meta::Path(path) => {
                if path.segments[0].ident == "default" {
                    Ok(FieldAttrKind::Default)
                } else {
                    Err(BuilderError::UnknownAttr(meta.clone()))
                }
            }
            syn::Meta::NameValue(name_value) => {
                if name_value.path.segments[0].ident == "each" {
                    Ok(FieldAttrKind::Repeat)
                } else {
                    Err(BuilderError::UnknownAttr(meta.clone()))
                }
            }
            syn::Meta::List(list) => Err(BuilderError::NestedMetaList(list.clone())),
        },
        syn::NestedMeta::Lit(lit) => Err(BuilderError::NotExpectedLit(lit.clone())),
    }
}

fn parse_name_value(nested: &syn::NestedMeta, expected_name: &str) -> Result<String, BuilderError> {
    if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. })) =
        nested
    {
        // Make sure the name is what it's expected.
        let name = path.segments[0].ident.to_string();
        if name != "each" {
            return Err(BuilderError::UnexpectedName(
                path.segments[0].ident.clone(),
                expected_name.to_string(),
            ));
        }

        // Compute the string value.
        let value = if let syn::Lit::Str(lit_str) = lit {
            lit_str.value()
        } else {
            return Err(BuilderError::NonStrValue(lit.clone()));
        };

        Ok(value)
    } else {
        Err(BuilderError::NotNameValue(nested.clone()))
    }
}

fn parse_repeated(
    nested: &syn::punctuated::Punctuated<syn::NestedMeta, syn::token::Comma>,
) -> Result<FieldAttr, BuilderError> {
    let each = parse_name_value(&nested[0], "each")?;

    let new = if nested.len() >= 2 {
        parse_name_value(&nested[1], "new")?
    } else {
        "new".to_string()
    };

    let ext = if nested.len() == 3 {
        parse_name_value(&nested[2], "ext")?
    } else {
        "single".to_string()
    };

    Ok(FieldAttr::Repeat { each, new, ext })
}

pub fn parse_attrs(field: &syn::Field) -> Result<Vec<FieldAttr>, BuilderError> {
    let mut parsed_attrs = vec![];

    for raw_attr in &field.attrs {
        if let Ok(syn::Meta::List(syn::MetaList { nested, .. })) = raw_attr.parse_meta() {
            let parsed_attr = match attr_kind(&nested)? {
                FieldAttrKind::Repeat => parse_repeated(&nested)?,
                FieldAttrKind::Default => FieldAttr::Default,
            };

            parsed_attrs.push(parsed_attr);
        } else {
            return Err(BuilderError::NotMetaList(raw_attr.clone()));
        }
    }

    Ok(parsed_attrs)
}
