use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::FfiFunction;
use crate::version::version_cfg_tokens_opt;

pub fn generate_static(fns: &[FfiFunction]) -> TokenStream {
    let extern_decls: Vec<TokenStream> = fns.iter().map(gen_extern_decl).collect();
    let trait_methods: Vec<TokenStream> = fns.iter().map(gen_trait_method).collect();

    quote! {
        #[cfg(feature = "static")]
        unsafe extern "C" {
            #(#extern_decls)*
        }

        #[cfg(feature = "static")]
        pub struct StaticBindings;

        #[cfg(feature = "static")]
        #[allow(clippy::too_many_arguments)]
        impl PdfiumBindings for StaticBindings {
            #(#trait_methods)*
        }
    }
}

fn gen_extern_decl(f: &FfiFunction) -> TokenStream {
    let cfg = version_cfg_tokens_opt(f.since);
    let name = &f.name;
    let arg_decls: Vec<TokenStream> = f
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
            fn #name(#(#arg_decls),*) -> #ret;
        },
        None => quote! {
            #cfg
            fn #name(#(#arg_decls),*);
        },
    }
}

fn gen_trait_method(f: &FfiFunction) -> TokenStream {
    let cfg = version_cfg_tokens_opt(f.since);
    let name = &f.name;
    let arg_decls: Vec<TokenStream> = f
        .args
        .iter()
        .map(|a| {
            let n = &a.name;
            let t = &a.ty;
            quote! { #n: #t }
        })
        .collect();
    let arg_names: Vec<&syn::Ident> = f.args.iter().map(|a| &a.name).collect();

    match &f.ret {
        Some(ret) => quote! {
            #cfg
            unsafe fn #name(&self, #(#arg_decls),*) -> #ret {
                #name(#(#arg_names),*)
            }
        },
        None => quote! {
            #cfg
            unsafe fn #name(&self, #(#arg_decls),*) {
                #name(#(#arg_names),*)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FfiArg, FfiFunction};
    use proc_macro2::Span;

    fn make_fn(
        name: &str,
        args: Vec<(&str, &str)>,
        ret: Option<&str>,
        since: Option<u32>,
    ) -> FfiFunction {
        FfiFunction {
            name: syn::Ident::new(name, Span::call_site()),
            args: args
                .into_iter()
                .map(|(n, t)| FfiArg {
                    name: syn::Ident::new(n, Span::call_site()),
                    ty: syn::parse_str(t).unwrap(),
                })
                .collect(),
            ret: ret.map(|r| syn::parse_str(r).unwrap()),
            since,
        }
    }

    #[test]
    fn extern_block_generated() {
        let fns = vec![make_fn("FPDF_Init", vec![], None, None)];
        let tokens = generate_static(&fns);
        let s = tokens.to_string();
        assert!(s.contains("extern \"C\""));
        assert!(s.contains("FPDF_Init"));
    }

    #[test]
    fn versioned_extern() {
        let fns = vec![make_fn("FPDF_New", vec![], None, Some(7543))];
        let tokens = generate_static(&fns);
        let s = tokens.to_string();
        assert!(s.contains("pdfium_7543"));
    }

    #[test]
    fn unit_struct() {
        let fns = vec![make_fn("FPDF_Init", vec![], None, None)];
        let tokens = generate_static(&fns);
        let s = tokens.to_string();
        assert!(s.contains("pub struct StaticBindings"));
    }

    #[test]
    fn trait_impl() {
        let fns = vec![make_fn("FPDF_Init", vec![], None, None)];
        let tokens = generate_static(&fns);
        let s = tokens.to_string();
        assert!(s.contains("impl PdfiumBindings for StaticBindings"));
    }

    #[test]
    fn output_parses() {
        let fns = vec![
            make_fn("FPDF_Init", vec![], None, None),
            make_fn("FPDF_Get", vec![("x", "i32")], Some("i32"), Some(7811)),
        ];
        let tokens = generate_static(&fns);
        syn::parse2::<syn::File>(tokens).expect("should parse as valid Rust");
    }
}
