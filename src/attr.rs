use crate::err::BuilderError;

pub enum FieldAttr {
    Repeat(String),
    Default(Option<syn::Lit>),
    Skip,
}

fn parse_attr(
    nested: &syn::punctuated::Punctuated<syn::NestedMeta, syn::token::Comma>,
) -> Result<FieldAttr, BuilderError> {
    match &nested[0] {
        syn::NestedMeta::Meta(meta) => match meta {
            syn::Meta::Path(path) => {
                if path.segments[0].ident == "default" {
                    Ok(FieldAttr::Default(None))
                } else if path.segments[0].ident == "skip" {
                    Ok(FieldAttr::Skip)
                } else {
                    Err(BuilderError::UnknownAttr(meta.clone()))
                }
            }
            syn::Meta::NameValue(name_value) => {
                if name_value.path.segments[0].ident == "each" {
                    let each = parse_name_value(name_value)?;

                    Ok(FieldAttr::Repeat(each))
                } else if name_value.path.segments[0].ident == "default" {
                    Ok(FieldAttr::Default(Some(name_value.lit.clone())))
                } else {
                    Err(BuilderError::UnknownAttr(meta.clone()))
                }
            }
            syn::Meta::List(list) => Err(BuilderError::NestedMetaList(list.clone())),
        },
        syn::NestedMeta::Lit(lit) => Err(BuilderError::NotExpectedLit(lit.clone())),
    }
}

fn parse_name_value(name_value: &syn::MetaNameValue) -> Result<String, BuilderError> {
    if let syn::Lit::Str(lit_str) = &name_value.lit {
        Ok(lit_str.value())
    } else {
        Err(BuilderError::NotStrValue(name_value.lit.clone()))
    }
}

// Parses and returns the attributes of the `field`.
pub fn parse_attrs(field: &syn::Field) -> Result<FieldAttrs, BuilderError> {
    let mut parsed_attrs = vec![];

    for raw_attr in &field.attrs {
        if let Ok(syn::Meta::List(syn::MetaList { nested, .. })) = raw_attr.parse_meta() {
            parsed_attrs.push(parse_attr(&nested)?);
        } else {
            return Err(BuilderError::NotMetaList(raw_attr.clone()));
        }
    }

    Ok(FieldAttrs(parsed_attrs))
}

pub struct FieldAttrs(Vec<FieldAttr>);

impl FieldAttrs {
    pub fn skip(&self) -> bool {
        self.0.iter().any(|attr| matches!(&attr, FieldAttr::Skip))
    }

    pub fn is_default(&self) -> Option<Option<syn::Lit>> {
        self.0.iter().find_map(|attr| {
            if let FieldAttr::Default(default) = attr {
                Some(default.clone())
            } else {
                None
            }
        })
    }

    pub fn repeated(&self) -> Option<&String> {
        self.0.iter().find_map(|attr| {
            if let FieldAttr::Repeat(each) = attr {
                Some(each)
            } else {
                None
            }
        })
    }
}
