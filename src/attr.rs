use syn::spanned::Spanned;

#[derive(Clone)]
pub enum Value {
    Default,
    Lit(syn::Lit),
    Closure(syn::ExprClosure),
}

#[derive(Clone)]
pub struct Lazy {
    pub closure: Option<syn::ExprClosure>,
    pub override_value: bool,
    pub is_async: bool,
}

/*
    #[skip]

    #[hide]

    #[once]

    #[into]

    #[each = <string>]

    #[name = <string>]

    #[check = <closure>]

    #[value = default, <lit>, <closure>]

    #[lazy]
    #[lazy = Override]
    #[lazy = Async]
    #[lazy = closure]
    #[lazy = Async + Override]
    #[lazy = Override + closure]

    #[async]
*/
pub enum FieldAttr {
    Skip,
    Hide,
    Once,
    Into,
    Value(Value),
    Each(String),
    Name(String),
    Lazy(Lazy),
    Async,
    Check(syn::ExprClosure),
}

fn as_add(expr: &syn::Expr) -> Result<(&syn::Expr, &syn::Expr), syn::Error> {
    if let syn::Expr::Binary(syn::ExprBinary {
        left, op, right, ..
    }) = expr
    {
        if matches!(op, syn::BinOp::Add(_)) {
            Ok((left, right))
        } else {
            Err(syn::Error::new(op.span(), "Expected the '+' operator"))
        }
    } else {
        Err(syn::Error::new(expr.span(), "Expected a binary operation"))
    }
}

fn as_lazy(expr: &syn::Expr) -> Result<Lazy, syn::Error> {
    if let Ok(path) = as_path(expr) {
        if path.is_ident("Override") {
            Ok(Lazy {
                closure: None,
                override_value: true,
                is_async: false,
            })
        } else if path.is_ident("Async") {
            Ok(Lazy {
                closure: None,
                override_value: false,
                is_async: true,
            })
        } else {
            Err(syn::Error::new(path.span(), "Unknown attribute"))
        }
    } else if let Ok(closure) = as_closure(expr) {
        Ok(Lazy {
            closure: Some(closure.clone()),
            override_value: false,
            is_async: closure.asyncness.is_some(),
        })
    } else if let Ok((left, right)) = as_add(expr) {
        if let Ok(path) = as_path(left) {
            if path.is_ident("Async") {
                if let Ok(path) = as_path(right) {
                    if path.is_ident("Override") {
                        Ok(Lazy {
                            closure: None,
                            override_value: true,
                            is_async: true,
                        })
                    } else {
                        Err(syn::Error::new(right.span(), "Expected Override"))
                    }
                } else {
                    Err(syn::Error::new(right.span(), "Expected an identifier"))
                }
            } else if path.is_ident("Override") {
                if let Ok(closure) = as_closure(right) {
                    Ok(Lazy {
                        closure: Some(closure.clone()),
                        override_value: true,
                        is_async: closure.asyncness.is_some(),
                    })
                } else {
                    Err(syn::Error::new(right.span(), "Expected a closure"))
                }
            } else {
                Err(syn::Error::new(path.span(), "Expected Override or async"))
            }
        } else {
            Err(syn::Error::new(left.span(), "Expected an identifier"))
        }
    } else {
        Err(syn::Error::new(expr.span(), "Malformed attribute"))
    }
}

fn as_value(expr: &syn::Expr) -> Result<Value, syn::Error> {
    if let Ok(path) = as_path(expr) {
        if path.is_ident("default") {
            Ok(Value::Default)
        } else {
            Err(syn::Error::new(path.span(), "Expected default"))
        }
    } else if let Ok(lit) = as_lit(expr) {
        Ok(Value::Lit(lit.clone()))
    } else if let Ok(closure) = as_closure(expr) {
        Ok(Value::Closure(closure.clone()))
    } else {
        Err(syn::Error::new(
            expr.span(),
            "Expected a literal, path, or a callable",
        ))
    }
}

fn as_path(expr: &syn::Expr) -> Result<&syn::Path, syn::Error> {
    if let syn::Expr::Path(syn::ExprPath { path, .. }) = expr {
        Ok(path)
    } else {
        Err(syn::Error::new(expr.span(), "Expected a path"))
    }
}

fn as_closure(expr: &syn::Expr) -> Result<&syn::ExprClosure, syn::Error> {
    if let syn::Expr::Closure(closure) = expr {
        Ok(closure)
    } else {
        Err(syn::Error::new(expr.span(), "Expected a path"))
    }
}

fn as_lit(expr: &syn::Expr) -> Result<&syn::Lit, syn::Error> {
    if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = expr {
        Ok(lit)
    } else {
        Err(syn::Error::new(expr.span(), "Expected a literal"))
    }
}

fn as_string(lit: &syn::Lit) -> Result<String, syn::Error> {
    if let syn::Lit::Str(str) = lit {
        Ok(str.value())
    } else {
        Err(syn::Error::new(lit.span(), "Expected a string literal"))
    }
}

fn parse_attr(attr: &syn::Attribute) -> Result<FieldAttr, syn::Error> {
    let result = if let Ok(path) = attr.parse_args::<syn::Path>() {
        if path.is_ident("skip") {
            Ok(FieldAttr::Skip)
        } else if path.is_ident("hide") {
            Ok(FieldAttr::Hide)
        } else if path.is_ident("once") {
            Ok(FieldAttr::Once)
        } else if path.is_ident("into") {
            Ok(FieldAttr::Into)
        } else if path.is_ident("lazy") {
            Ok(FieldAttr::Lazy(Lazy {
                closure: None,
                override_value: false,
                is_async: false,
            }))
        } else if path.is_ident("async") {
            Ok(FieldAttr::Async)
        } else {
            Err(syn::Error::new(path.span(), "Unknown attribute"))
        }
    } else if let Ok(assign) = attr.parse_args::<syn::ExprAssign>() {
        let syn::Expr::Path(left) = assign.left.as_ref() else {
            return Err(syn::Error::new(assign.left.span(), "Expected an identifier"));
        };

        if left.path.is_ident("each") {
            let string = as_string(as_lit(assign.right.as_ref())?)?;

            Ok(FieldAttr::Each(string))
        } else if left.path.is_ident("name") {
            let string = as_string(as_lit(assign.right.as_ref())?)?;

            Ok(FieldAttr::Name(string))
        } else if left.path.is_ident("value") {
            let value = as_value(&assign.right)?;

            Ok(FieldAttr::Value(value))
        } else if left.path.is_ident("check") {
            let closure = as_closure(&assign.right)?;

            Ok(FieldAttr::Check(closure.clone()))
        } else if left.path.is_ident("lazy") {
            let lazy = as_lazy(&assign.right)?;

            Ok(FieldAttr::Lazy(lazy))
        } else {
            Err(syn::Error::new(assign.left.span(), "Unknown attribute"))
        }
    } else {
        Err(syn::Error::new(
            attr.bracket_token.span.span(),
            format!("Attribute is malformed: {attr:?}"),
        ))
    };

    result
}

pub fn parse_attrs(field: &syn::Field) -> Result<Attributes, syn::Error> {
    let attrs = field
        .attrs
        .iter()
        .map(parse_attr)
        .collect::<Result<Vec<FieldAttr>, syn::Error>>()?;

    let mut attributes = Attributes {
        skip: false,
        hide: false,
        once: false,
        into: false,
        async_setter: false,
        value: None,
        each: None,
        name: None,
        lazy: None,
        check: None,
    };

    for attr in attrs {
        match attr {
            FieldAttr::Skip => attributes.skip = true,
            FieldAttr::Hide => attributes.hide = true,
            FieldAttr::Once => attributes.once = true,
            FieldAttr::Into => attributes.into = true,
            FieldAttr::Async => attributes.async_setter = true,
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
    pub skip: bool,
    pub hide: bool,
    pub once: bool,
    pub into: bool,
    pub async_setter: bool,
    pub value: Option<Value>,
    pub each: Option<String>,
    pub name: Option<String>,
    pub lazy: Option<Lazy>,
    pub check: Option<syn::ExprClosure>,
}

impl Attributes {
    pub fn has_value(&self) -> bool {
        let has_lazy_value = self.lazy.is_some() && self.lazy.as_ref().unwrap().closure.is_some();

        self.value.is_some() || has_lazy_value
    }
}
