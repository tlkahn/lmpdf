pub struct FfiArg {
    pub name: syn::Ident,
    pub ty: syn::Type,
}

pub struct FfiFunction {
    pub name: syn::Ident,
    pub args: Vec<FfiArg>,
    pub ret: Option<syn::Type>,
    pub since: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn make_fn(name: &str, since: Option<u32>, ret: Option<syn::Type>) -> FfiFunction {
        FfiFunction {
            name: syn::Ident::new(name, proc_macro2::Span::call_site()),
            args: vec![],
            ret,
            since,
        }
    }

    #[test]
    fn no_version_gate() {
        let f = make_fn("FPDF_Init", None, None);
        assert!(f.since.is_none());
    }

    #[test]
    fn has_version_gate() {
        let f = make_fn("FPDF_Init", Some(7543), None);
        assert!(f.since.is_some());
    }

    #[test]
    fn no_return_type() {
        let f = make_fn("FPDF_Init", None, None);
        assert!(f.ret.is_none());
    }

    #[test]
    fn has_return_type() {
        let ty: syn::Type = parse_quote!(c_int);
        let f = make_fn("FPDF_Init", None, Some(ty));
        assert!(f.ret.is_some());
    }

    #[test]
    fn args_stored() {
        let f = FfiFunction {
            name: syn::Ident::new("Foo", proc_macro2::Span::call_site()),
            args: vec![FfiArg {
                name: syn::Ident::new("x", proc_macro2::Span::call_site()),
                ty: parse_quote!(c_int),
            }],
            ret: None,
            since: None,
        };
        assert_eq!(f.args.len(), 1);
        assert_eq!(f.args[0].name, "x");
    }
}
