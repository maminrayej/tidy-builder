use syn::spanned::Spanned;

pub enum Callable {
    Path(syn::Path),
    Closure(syn::ExprClosure),
}

pub enum Default {
    Lit(syn::Lit),
    Callable(Callable),
}

/*
    #[skip]
    #[hide]
    #[once]
    #[into]
    #[default] #[default = <lit>, <path>, <closure>]
    #[each = <string>]
    #[name = <string>]
    #[lazy = <path>, <closure>]
    #[async = <path>, <closure>]
    #[check = <path>, <closure>]
*/
pub enum FieldAttr {
    Skip,
    Hide,
    Once,
    Into,
    Default(Option<Default>),
    Each(String),
    Name(String),
    Lazy(Callable),
    Async(Callable),
    Check(Callable),
}

fn as_default(expr: &syn::Expr) -> Result<Default, syn::Error> {
    if let Ok(lit) = as_lit(expr) {
        Ok(Default::Lit(lit.clone()))
    } else if let Ok(callable) = as_callable(expr) {
        Ok(Default::Callable(callable))
    } else {
        Err(syn::Error::new(
            expr.span(),
            "Expected a literal, path, or a callable",
        ))
    }
}

fn as_callable(expr: &syn::Expr) -> Result<Callable, syn::Error> {
    if let Ok(path) = as_path(expr) {
        Ok(Callable::Path(path.clone()))
    } else if let Ok(closure) = as_closure(expr) {
        Ok(Callable::Closure(closure.clone()))
    } else {
        Err(syn::Error::new(expr.span(), "Expected a path or a closure"))
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
        } else if path.is_ident("default") {
            Ok(FieldAttr::Default(None))
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
        } else if left.path.is_ident("default") {
            let default = as_default(assign.right.as_ref())?;

            Ok(FieldAttr::Default(Some(default)))
        } else if left.path.is_ident("check") {
            let callable = as_callable(assign.right.as_ref())?;

            Ok(FieldAttr::Check(callable))
        } else if left.path.is_ident("lazy") {
            let callable = as_callable(assign.right.as_ref())?;

            Ok(FieldAttr::Lazy(callable))
        } else if left.path.is_ident("async") {
            let callable = as_callable(assign.right.as_ref())?;

            Ok(FieldAttr::Async(callable))
        } else {
            Err(syn::Error::new(assign.left.span(), "Unknown attribute"))
        }
    } else {
        Err(syn::Error::new(attr.span(), "Attribute is malformed"))
    };

    result
}
