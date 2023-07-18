/*
    #[props = skip, hide, once, into]
    #[each = <ident>, <callable>]
    #[name = <ident>]
    #[value = default, <lit>, <callable>]
    #[check = <callable>]
    #[lazy = override, async, <callable>]
*/

#[derive(Debug, Clone)]
pub struct Lazy {
    pub do_override: Option<syn::token::Override>,
    pub asyncness: Option<syn::token::Async>,
    pub callable: Option<CallableItem>,
}

impl syn::parse::Parse for Lazy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut do_override = None;
        let mut asyncness = None;
        let mut callable = None;

        if input.lookahead1().peek(syn::Token![override]) {
            do_override = Some(input.parse::<syn::token::Override>().unwrap());

            if input.lookahead1().peek(syn::Token![,]) {
                let _ = input.parse::<syn::token::Comma>().unwrap();

                if input.lookahead1().peek(syn::Token![async]) {
                    asyncness = Some(input.parse::<syn::token::Async>().unwrap());

                    if input.lookahead1().peek(syn::Token![,]) {
                        let _ = input.parse::<syn::token::Comma>().unwrap();

                        callable = Some(input.parse::<CallableItem>()?);
                    }
                } else {
                    callable = Some(input.parse::<CallableItem>()?);
                }
            }
        } else {
            if !input.is_empty() {
                callable = Some(input.parse::<CallableItem>()?)
            }
        }

        Ok(Lazy {
            do_override,
            asyncness,
            callable,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Default(syn::token::Default),
    Lit(syn::Lit),
    Callable(CallableItem),
}

impl syn::parse::Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::Token![default]) {
            input.parse::<syn::token::Default>().map(Value::Default)
        } else if lookahead.peek(syn::Lit) {
            input.parse::<syn::Lit>().map(Value::Lit)
        } else {
            input.parse::<CallableItem>().map(Value::Callable)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Each {
    pub ident: syn::Ident,
    pub callable: Option<CallableItem>,
}

impl syn::parse::Parse for Each {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let mut callable = None;

        if input.lookahead1().peek(syn::Token![,]) {
            let _ = input.parse::<syn::token::Comma>().unwrap();

            callable = Some(input.parse::<CallableItem>()?);
        }

        Ok(Each { ident, callable })
    }
}

#[derive(Debug, Clone)]
pub struct Props {
    pub skip: bool,
    pub hide: bool,
    pub once: bool,
    pub into: bool,
}

impl syn::parse::Parse for Props {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let idents = input.parse_terminated(syn::Ident::parse, syn::Token![,])?;

        let mut props = Props {
            skip: false,
            hide: false,
            once: false,
            into: false,
        };

        for ident in idents {
            if ident == "skip" {
                props.skip = true;
            } else if ident == "hide" {
                props.hide = true;
            } else if ident == "once" {
                props.once = true;
            } else if ident == "into" {
                props.into = true;
            }
        }

        Ok(props)
    }
}

#[derive(Debug, Clone)]
pub enum Callable {
    Function(syn::Path),
    Closure(syn::ExprClosure),
}

/* <async>? <path> | <closure> */
#[derive(Debug, Clone)]
pub struct CallableItem {
    pub asyncness: Option<syn::token::Async>,
    pub callable: Callable,
}

impl syn::parse::Parse for CallableItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        let callable = if lookahead.peek(syn::Token![|]) {
            let closure = input.parse::<syn::ExprClosure>()?;

            CallableItem {
                asyncness: closure.asyncness,
                callable: Callable::Closure(closure),
            }
        } else {
            let asyncness = lookahead
                .peek(syn::Token![async])
                .then(|| input.parse::<syn::token::Async>().unwrap());

            let callable = input.parse().map(Callable::Function)?;

            CallableItem {
                asyncness,
                callable,
            }
        };

        Ok(callable)
    }
}

impl quote::ToTokens for CallableItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.callable {
            Callable::Function(path) => path.to_tokens(tokens),
            Callable::Closure(closure) => closure.to_tokens(tokens),
        }
    }
}

pub enum FieldAttr {
    Props(Props),
    Value(Value),
    Each(Each),
    Name(syn::Ident),
    Lazy(Lazy),
    Check(CallableItem),
}

impl syn::parse::Parse for FieldAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let _ = input.parse::<syn::token::Eq>()?;

        if ident == "props" {
            input.parse::<Props>().map(FieldAttr::Props)
        } else if ident == "value" {
            input.parse::<Value>().map(FieldAttr::Value)
        } else if ident == "each" {
            input.parse::<Each>().map(FieldAttr::Each)
        } else if ident == "name" {
            input.parse::<syn::Ident>().map(FieldAttr::Name)
        } else if ident == "lazy" {
            input.parse::<Lazy>().map(FieldAttr::Lazy)
        } else if ident == "check" {
            input.parse::<CallableItem>().map(FieldAttr::Check)
        } else {
            Err(syn::Error::new(ident.span(), "Unknown attribute"))
        }
    }
}

pub fn parse_attrs(field: &syn::Field) -> Result<Attributes, syn::Error> {
    let mut attributes = Attributes {
        props: Props {
            skip: false,
            hide: false,
            once: false,
            into: false,
        },
        value: None,
        each: None,
        name: None,
        lazy: None,
        check: None,
    };

    for attr in field
        .attrs
        .iter()
        .map(|attr| attr.parse_args::<FieldAttr>())
    {
        match attr? {
            FieldAttr::Props(props) => attributes.props = props,
            FieldAttr::Value(value) => attributes.value = Some(value),
            FieldAttr::Each(each) => attributes.each = Some(each),
            FieldAttr::Name(name) => attributes.name = Some(name),
            FieldAttr::Lazy(lazy) => attributes.lazy = Some(lazy),
            FieldAttr::Check(check) => attributes.check = Some(check),
        }
    }

    Ok(attributes)
}

pub struct Attributes {
    pub props: Props,
    pub value: Option<Value>,
    pub each: Option<Each>,
    pub name: Option<syn::Ident>,
    pub lazy: Option<Lazy>,
    pub check: Option<CallableItem>,
}

impl Attributes {
    pub fn has_value(&self) -> bool {
        let has_lazy_value = self.lazy.is_some() && self.lazy.as_ref().unwrap().callable.is_some();

        self.value.is_some() || has_lazy_value
    }
}
