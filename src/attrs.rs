/*
    #[setter     = skip, once, into]
    #[vis        = <visibility>]
    #[each       = <ident>, async? <func>?]
    #[value      = default | <lit> | async? <func>]
    #[check      = async? <func>]
    #[lazy       = override? async? <func>]
*/

#[derive(Debug)]
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
pub struct Attrs {
    setter: Option<Setter>,
    vis: Option<syn::Visibility>,
    each: Option<Each>,
    value: Option<Value>,
    check: Option<Function>,
    lazy: Option<Lazy>,
}