use crate::err::BuilderError;

pub enum FieldAttrKind {
    Repeat,
    Default,
}

pub enum FieldAttr {
    // #[builder(each = "each")
    Repeat(String),

    // #[builder(default)]
    Default,
}

// #[builder(each = "each")]
//           -------------
//            nested meta
//
// # Arguments
// `nested`: List of comma seperated nested metas.
//
// # Returns
// The attribute kind based on the first nested meta.
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

// Tries to parse the provided `nested` as a name-value meta.
// It returns the value only if the name is equal to the `expected_name`.
fn parse_name_value(nested: &syn::NestedMeta, expected_name: &str) -> Result<String, BuilderError> {
    if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. })) =
        nested
    {
        // Make sure the name is what it's expected.
        let name = &path.segments[0].ident;
        if name != expected_name {
            return Err(BuilderError::UnexpectedName(
                name.clone(),
                expected_name.to_string(),
            ));
        }

        // Compute the string value.
        let value = if let syn::Lit::Str(lit_str) = lit {
            lit_str.value()
        } else {
            return Err(BuilderError::NotStrValue(lit.clone()));
        };

        Ok(value)
    } else {
        Err(BuilderError::NotNameValue(nested.clone()))
    }
}
// Returns the parsed #[builder(each = "each")] attribute.
fn parse_repeated(
    nested: &syn::punctuated::Punctuated<syn::NestedMeta, syn::token::Comma>,
) -> Result<FieldAttr, BuilderError> {
    let each = parse_name_value(&nested[0], "each")?;

    Ok(FieldAttr::Repeat(each))
}

// Parses and returns the attributes of the `field`.
pub fn parse_attrs(field: &syn::Field) -> Result<FieldAttrs, BuilderError> {
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

    Ok(FieldAttrs(parsed_attrs))
}

pub struct FieldAttrs(Vec<FieldAttr>);

impl FieldAttrs {
    pub fn is_default(&self) -> bool {
        self.0.iter().any(|attr| matches!(attr, FieldAttr::Default))
    }

    pub fn repeated(&self) -> Option<&String> {
        self.0.iter().find_map(|attr| match attr {
            FieldAttr::Repeat(each) => Some(each),
            FieldAttr::Default => None,
        })
    }
}
