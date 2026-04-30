use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::FfiFunction;
use crate::version::version_cfg_tokens_opt;

pub fn generate_dynamic(fns: &[FfiFunction]) -> TokenStream {
    let fields: Vec<TokenStream> = fns.iter().map(gen_field).collect();
    let load_lets: Vec<TokenStream> = fns.iter().map(gen_load_let).collect();
    let init_fields: Vec<TokenStream> = fns.iter().map(gen_init_field).collect();
    let trait_methods: Vec<TokenStream> = fns.iter().map(gen_trait_method).collect();

    quote! {
        pub struct DynamicBindings {
            _library: libloading::Library,
            #(#fields)*
        }

        impl DynamicBindings {
            pub fn load(library: libloading::Library) -> Result<Self, libloading::Error> {
                unsafe {
                    #(#load_lets)*
                    Ok(Self {
                        _library: library,
                        #(#init_fields)*
                    })
                }
            }
        }

        #[allow(clippy::too_many_arguments)]
        impl PdfiumBindings for DynamicBindings {
            #(#trait_methods)*
        }
    }
}

fn snake_case(name: &syn::Ident) -> syn::Ident {
    let s = name.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            let prev_lower = i > 0 && chars[i - 1].is_ascii_lowercase();
            let prev_upper_next_lower = i > 0
                && chars[i - 1].is_ascii_uppercase()
                && i + 1 < chars.len()
                && chars[i + 1].is_ascii_lowercase();
            if prev_lower || prev_upper_next_lower {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    syn::Ident::new(&result, name.span())
}

fn fn_pointer_type(f: &FfiFunction) -> TokenStream {
    let arg_types: Vec<&syn::Type> = f.args.iter().map(|a| &a.ty).collect();
    match &f.ret {
        Some(ret) => quote! { unsafe extern "C" fn(#(#arg_types),*) -> #ret },
        None => quote! { unsafe extern "C" fn(#(#arg_types),*) },
    }
}

fn gen_field(f: &FfiFunction) -> TokenStream {
    let cfg = version_cfg_tokens_opt(f.since);
    let field_name = snake_case(&f.name);
    let fn_ptr = fn_pointer_type(f);
    quote! {
        #cfg
        #field_name: #fn_ptr,
    }
}

fn gen_load_let(f: &FfiFunction) -> TokenStream {
    let cfg = version_cfg_tokens_opt(f.since);
    let field_name = snake_case(&f.name);
    let symbol_bytes = format!("{}\0", f.name);
    let fn_ptr = fn_pointer_type(f);
    quote! {
        #cfg
        let #field_name = *library.get::<#fn_ptr>(#symbol_bytes.as_bytes())?;
    }
}

fn gen_init_field(f: &FfiFunction) -> TokenStream {
    let cfg = version_cfg_tokens_opt(f.since);
    let field_name = snake_case(&f.name);
    quote! {
        #cfg
        #field_name,
    }
}

fn gen_trait_method(f: &FfiFunction) -> TokenStream {
    let cfg = version_cfg_tokens_opt(f.since);
    let name = &f.name;
    let field_name = snake_case(&f.name);
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
                (self.#field_name)(#(#arg_names),*)
            }
        },
        None => quote! {
            #cfg
            unsafe fn #name(&self, #(#arg_decls),*) {
                (self.#field_name)(#(#arg_names),*)
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
    fn snake_case_conversion() {
        let id = syn::Ident::new("FPDF_InitLibrary", Span::call_site());
        assert_eq!(snake_case(&id).to_string(), "fpdf_init_library");

        let id2 = syn::Ident::new("FPDFBitmap_Create", Span::call_site());
        assert_eq!(snake_case(&id2).to_string(), "fpdf_bitmap_create");
    }

    #[test]
    fn struct_has_fields() {
        let fns = vec![make_fn("FPDF_Init", vec![], None, None)];
        let tokens = generate_dynamic(&fns);
        let s = tokens.to_string();
        assert!(s.contains("fpdf_init"));
        assert!(s.contains("DynamicBindings"));
    }

    #[test]
    fn versioned_field_has_cfg() {
        let fns = vec![make_fn("FPDF_New", vec![], None, Some(7543))];
        let tokens = generate_dynamic(&fns);
        let s = tokens.to_string();
        assert!(s.contains("cfg"));
        assert!(s.contains("pdfium_7543"));
    }

    #[test]
    fn load_fn_signature() {
        let fns = vec![make_fn("FPDF_Init", vec![], None, None)];
        let tokens = generate_dynamic(&fns);
        let s = tokens.to_string();
        assert!(s.contains("fn load"));
        assert!(s.contains("libloading :: Library") || s.contains("libloading::Library"));
        assert!(s.contains("libloading :: Error") || s.contains("libloading::Error"));
    }

    #[test]
    fn trait_impl_delegates() {
        let fns = vec![make_fn(
            "FPDF_GetPageCount",
            vec![("doc", "i32")],
            Some("i32"),
            None,
        )];
        let tokens = generate_dynamic(&fns);
        let s = tokens.to_string();
        assert!(s.contains("impl PdfiumBindings for DynamicBindings"));
        assert!(s.contains("FPDF_GetPageCount"));
    }

    #[test]
    fn output_parses() {
        let fns = vec![
            make_fn("FPDF_Init", vec![], None, None),
            make_fn("FPDF_Get", vec![("x", "i32")], Some("i32"), Some(7811)),
        ];
        let tokens = generate_dynamic(&fns);
        syn::parse2::<syn::File>(tokens).expect("should parse as valid Rust");
    }
}
