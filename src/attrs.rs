/*
    #[setter     = skip, once, into]
    #[vis        = <visibility>]
    #[each       = <ident>, async? <func>?]
    #[value      = default | <lit> | async? <func>]
    #[check      = async? <func>]
    #[lazy       = override? async? <func>]
*/

#[derive(Debug, Default)]
pub struct Setter {
    pub skip: bool,
    pub once: bool,
    pub into: bool,
}
impl syn::parse::Parse for Setter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut setter = Setter {
            skip: false,
            once: false,
            into: false,
        };

        let idents = input.parse_terminated(syn::Ident::parse, syn::Token!(,))?;

        for ident in idents {
            if ident == "skip" {
                setter.skip = true;
            } else if ident == "once" {
                setter.once = true;
            } else if ident == "into" {
                setter.into = true;
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "only skip, once, or into are valid",
                ));
            }
        }

        Ok(setter)
    }
}

#[derive(Debug)]
pub struct Function {
    pub is_async: bool,
    pub function: syn::Ident,
}
impl syn::parse::Parse for Function {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_async = input.parse::<syn::Token![async]>().is_ok();

        let function = input.parse::<syn::Ident>()?;

        Ok(Function { is_async, function })
    }
}

#[derive(Debug)]
pub struct Each {
    pub ident: syn::Ident,
    pub check: Option<Function>,
}
impl syn::parse::Parse for Each {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let mut check = None;

        if input.parse::<syn::Token![,]>().is_ok() {
            check = Some(input.parse::<Function>()?);
        }

        Ok(Each { ident, check })
    }
}

#[derive(Debug)]
pub enum Value {
    Default,
    Literal(syn::Lit),
    Function(Function),
}
impl syn::parse::Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let value = if let Ok(_) = input.parse::<syn::Token![default]>() {
            Value::Default
        } else if let Ok(lit) = input.parse::<syn::Lit>() {
            Value::Literal(lit)
        } else {
            let function = input.parse::<Function>()?;

            Value::Function(function)
        };

        Ok(value)
    }
}

#[derive(Debug)]
pub struct Lazy {
    pub is_override: bool,
    pub function: Function,
}
impl syn::parse::Parse for Lazy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_override = input.parse::<syn::Token![override]>().is_ok();
        let function = input.parse::<Function>()?;

        Ok(Lazy {
            is_override,
            function,
        })
    }
}

#[derive(Debug)]
enum Attr {
    Setter(Setter),
    Vis(syn::Visibility),
    Each(Each),
    Value(Value),
    Check(Function),
    Lazy(Lazy),
}

impl syn::parse::Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;

        let _ = input.parse::<syn::Token![=]>()?;

        if ident == "setter" {
            input.parse::<Setter>().map(Attr::Setter)
        } else if ident == "vis" {
            input.parse::<syn::Visibility>().map(Attr::Vis)
        } else if ident == "each" {
            input.parse::<Each>().map(Attr::Each)
        } else if ident == "value" {
            input.parse::<Value>().map(Attr::Value)
        } else if ident == "check" {
            input.parse::<Function>().map(Attr::Check)
        } else if ident == "lazy" {
            input.parse::<Lazy>().map(Attr::Lazy)
        } else {
            Err(syn::Error::new(ident.span(), "unexpected attribute identifier"))
        }
    }
}

#[derive(Debug, Default)]
pub struct Attrs {
    pub setter: Setter,
    pub vis: Option<syn::Visibility>,
    pub each: Option<Each>,
    pub value: Option<Value>,
    pub check: Option<Function>,
    pub lazy: Option<Lazy>,
}

impl Attrs {
    pub fn has_value(&self) -> bool {
        self.lazy.is_some() || self.value.is_some()
    }
}


pub fn parse_attrs(field: &syn::Field) -> syn::Result<Attrs> {
    let mut attrs: Attrs = Default::default();

    for attr in &field.attrs {
        let attr = attr.parse_args::<Attr>()?;

        match attr {
            Attr::Setter(setter) => attrs.setter = setter,
            Attr::Vis(vis) => attrs.vis = Some(vis),
            Attr::Each(each) => attrs.each = Some(each),
            Attr::Value(value) => attrs.value = Some(value),
            Attr::Check(check) => attrs.check = Some(check),
            Attr::Lazy(lazy) => attrs.lazy = Some(lazy),
        }
    }

    Ok(attrs)
}