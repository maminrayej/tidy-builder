use crate::err::Error;

// Different attributes that a field can have.
pub enum FieldAttr {
    // Represents the repeated setter attribute: `#[builder(each = "name")]`
    // `String` will be the name specified by the user.
    Repeat(String),

    // Represents the default attribute:
    // If the `Option<syn::Lit>` is `None`: `#[builder(default)]`
    // If the `Option<syn::Lit>` is `Some`: `#[builder(default = lit)]`
    Default(Option<syn::Lit>),

    // Represents the `#[builder(skip)]` attribute.
    Skip,
}

fn parse_attr(
    nested: &syn::punctuated::Punctuated<syn::NestedMeta, syn::token::Comma>,
) -> Result<FieldAttr, Error> {
    match &nested[0] {
        syn::NestedMeta::Meta(meta) => match meta {
            // Single word attributes:
            // * `#[builder(default)]`
            // * `#[builder(skip)]`
            syn::Meta::Path(path) => {
                let name = &path.segments[0].ident;

                match name.to_string().as_str() {
                    "default" => Ok(FieldAttr::Default(None)),
                    "skip" => Ok(FieldAttr::Skip),
                    _ => Err(Error::UnknownAttr(meta.clone())),
                }
            }
            // Name value attributes:
            // * `#[builder(each = "name")]`
            // * `#[builder(default = lit)]`
            syn::Meta::NameValue(name_value) => {
                let name = &name_value.path.segments[0].ident;

                match name.to_string().as_str() {
                    "each" => {
                        let each = extract_value(name_value)?;

                        Ok(FieldAttr::Repeat(each))
                    }
                    "default" => Ok(FieldAttr::Default(Some(name_value.lit.clone()))),
                    _ => Err(Error::UnknownAttr(meta.clone())),
                }
            }
            syn::Meta::List(list) => Err(Error::NestedMetaList(list.clone())),
        },
        syn::NestedMeta::Lit(lit) => Err(Error::UnexpectedLit(lit.clone())),
    }
}

fn extract_value(name_value: &syn::MetaNameValue) -> Result<String, Error> {
    if let syn::Lit::Str(lit_str) = &name_value.lit {
        Ok(lit_str.value())
    } else {
        Err(Error::NotStrValue(name_value.lit.clone()))
    }
}

// Parses and returns the attributes of the `field`.
pub fn parse_attrs(field: &syn::Field) -> Result<FieldAttrs, Error> {
    let mut parsed_attrs = vec![];

    for raw_attr in &field.attrs {
        if let Ok(syn::Meta::List(syn::MetaList { nested, .. })) = raw_attr.parse_meta() {
            parsed_attrs.push(parse_attr(&nested)?);
        } else {
            return Err(Error::NotMetaList(raw_attr.clone()));
        }
    }

    Ok(FieldAttrs(parsed_attrs))
}

pub struct FieldAttrs(Vec<FieldAttr>);

impl FieldAttrs {
    pub fn should_skip(&self) -> bool {
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
