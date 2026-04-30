use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::FfiFunction;
use crate::version::version_cfg_tokens_opt;

pub fn generate_wasm(fns: &[FfiFunction]) -> TokenStream {
    let methods: Vec<TokenStream> = fns.iter().map(gen_stub_method).collect();

    quote! {
        #[cfg(target_arch = "wasm32")]
        pub struct WasmBindings;

        #[cfg(target_arch = "wasm32")]
        #[allow(clippy::too_many_arguments)]
        impl PdfiumBindings for WasmBindings {
            #(#methods)*
        }
    }
}

fn gen_stub_method(f: &FfiFunction) -> TokenStream {
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
            unsafe fn #name(&self, #(#arg_decls),*) -> #ret {
                todo!("WASM backend not yet implemented")
            }
        },
        None => quote! {
            #cfg
            unsafe fn #name(&self, #(#arg_decls),*) {
                todo!("WASM backend not yet implemented")
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
    fn struct_exists() {
        let fns = vec![make_fn("FPDF_Init", vec![], None, None)];
        let tokens = generate_wasm(&fns);
        let s = tokens.to_string();
        assert!(s.contains("pub struct WasmBindings"));
        assert!(s.contains("impl PdfiumBindings for WasmBindings"));
        assert!(s.contains("todo !"));
    }

    #[test]
    fn everything_cfg_gated() {
        let fns = vec![make_fn("FPDF_Init", vec![], None, None)];
        let tokens = generate_wasm(&fns);
        let s = tokens.to_string();
        assert!(s.contains("target_arch = \"wasm32\""));
    }
}
