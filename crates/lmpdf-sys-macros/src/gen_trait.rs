use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::FfiFunction;
use crate::version::version_cfg_tokens_opt;

pub fn generate_trait(fns: &[FfiFunction]) -> TokenStream {
    let methods: Vec<TokenStream> = fns.iter().map(generate_method).collect();
    quote! {
        #[allow(clippy::too_many_arguments)]
        pub trait PdfiumBindings: Send + Sync {
            #(#methods)*
        }
    }
}

fn generate_method(f: &FfiFunction) -> TokenStream {
    let cfg = version_cfg_tokens_opt(f.since);
    let name = &f.name;
    let args: Vec<TokenStream> = f
        .args
        .iter()
        .map(|a| {
            let n = &a.name;
            let t = &a.ty;
            quote! { #n: #t }
        })
        .collect();

    match &f.ret {
        Some(ret) => quote! {
            #cfg
            unsafe fn #name(&self, #(#args),*) -> #ret;
        },
        None => quote! {
            #cfg
            unsafe fn #name(&self, #(#args),*);
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FfiArg, FfiFunction};

    fn make_fn(
        name: &str,
        args: Vec<(&str, &str)>,
        ret: Option<&str>,
        since: Option<u32>,
    ) -> FfiFunction {
        FfiFunction {
            name: syn::Ident::new(name, proc_macro2::Span::call_site()),
            args: args
                .into_iter()
                .map(|(n, t)| FfiArg {
                    name: syn::Ident::new(n, proc_macro2::Span::call_site()),
                    ty: syn::parse_str(t).unwrap(),
                })
                .collect(),
            ret: ret.map(|r| syn::parse_str(r).unwrap()),
            since,
        }
    }

    #[test]
    fn single_void_fn() {
        let fns = vec![make_fn("FPDF_InitLibrary", vec![], None, None)];
        let tokens = generate_trait(&fns);
        let s = tokens.to_string();
        assert!(s.contains("FPDF_InitLibrary"));
        assert!(s.contains("PdfiumBindings"));
        assert!(s.contains("unsafe fn"));
    }

    #[test]
    fn fn_with_args_and_return() {
        let fns = vec![make_fn(
            "FPDF_GetPageCount",
            vec![("document", "FPDF_DOCUMENT")],
            Some("c_int"),
            None,
        )];
        let tokens = generate_trait(&fns);
        let s = tokens.to_string();
        assert!(s.contains("document : FPDF_DOCUMENT") || s.contains("document: FPDF_DOCUMENT"));
        assert!(s.contains("c_int"));
    }

    #[test]
    fn version_gated_fn() {
        let fns = vec![make_fn("FPDF_New", vec![], None, Some(7543))];
        let tokens = generate_trait(&fns);
        let s = tokens.to_string();
        assert!(s.contains("cfg"));
        assert!(s.contains("pdfium_7543"));
    }

    #[test]
    fn multiple_fns() {
        let fns = vec![
            make_fn("A", vec![], None, None),
            make_fn("B", vec![], Some("c_int"), None),
        ];
        let tokens = generate_trait(&fns);
        let s = tokens.to_string();
        assert!(s.contains("fn A"));
        assert!(s.contains("fn B"));
    }

    #[test]
    fn output_parseable_as_trait() {
        let fns = vec![
            make_fn("FPDF_Init", vec![], None, None),
            make_fn("FPDF_Get", vec![("doc", "i32")], Some("i32"), Some(7811)),
        ];
        let tokens = generate_trait(&fns);
        syn::parse2::<syn::ItemTrait>(tokens).expect("should parse as a valid trait");
    }
}
