use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, Type, parenthesized};

use crate::ir::{FfiArg, FfiFunction};

pub struct FfiBlock {
    pub functions: Vec<FfiFunction>,
}

impl Parse for FfiBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut functions = Vec::new();
        while !input.is_empty() {
            functions.push(input.parse()?);
        }
        Ok(FfiBlock { functions })
    }
}

impl Parse for FfiFunction {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let since = if input.peek(Token![#]) {
            Some(parse_since_attr(input)?)
        } else {
            None
        };

        input.parse::<Token![fn]>()?;
        let name: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);
        let args = parse_args(&content)?;

        let ret = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse::<Type>()?)
        } else {
            None
        };

        input.parse::<Token![;]>()?;

        Ok(FfiFunction {
            name,
            args,
            ret,
            since,
        })
    }
}

fn parse_since_attr(input: ParseStream) -> syn::Result<u32> {
    input.parse::<Token![#]>()?;
    let attr_content;
    syn::bracketed!(attr_content in input);

    let attr_name: Ident = attr_content.parse()?;
    if attr_name != "since" {
        return Err(syn::Error::new(
            attr_name.span(),
            format!("unknown attribute `{attr_name}`, expected `since`"),
        ));
    }

    let version_content;
    parenthesized!(version_content in attr_content);
    let version: syn::LitInt = version_content.parse()?;
    version.base10_parse()
}

fn parse_args(input: ParseStream) -> syn::Result<Vec<FfiArg>> {
    let mut args = Vec::new();
    while !input.is_empty() {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        args.push(FfiArg { name, ty });
        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_block(input: &str) -> syn::Result<FfiBlock> {
        syn::parse_str(input)
    }

    #[test]
    fn single_void_fn() {
        let block = parse_block("fn FPDF_InitLibrary();").unwrap();
        assert_eq!(block.functions.len(), 1);
        assert_eq!(block.functions[0].name, "FPDF_InitLibrary");
        assert!(block.functions[0].args.is_empty());
        assert!(block.functions[0].ret.is_none());
        assert!(block.functions[0].since.is_none());
    }

    #[test]
    fn fn_with_args_and_return() {
        let block = parse_block("fn FPDF_LoadDocument(file_path: FPDF_STRING, password: FPDF_BYTESTRING) -> FPDF_DOCUMENT;").unwrap();
        let f = &block.functions[0];
        assert_eq!(f.name, "FPDF_LoadDocument");
        assert_eq!(f.args.len(), 2);
        assert_eq!(f.args[0].name, "file_path");
        assert_eq!(f.args[1].name, "password");
        assert!(f.ret.is_some());
    }

    #[test]
    fn fn_with_since() {
        let block = parse_block(
            "#[since(7543)] fn FPDF_GetXFAPacketCount(document: FPDF_DOCUMENT) -> c_int;",
        )
        .unwrap();
        let f = &block.functions[0];
        assert_eq!(f.since, Some(7543));
    }

    #[test]
    fn multiple_fns() {
        let block = parse_block(
            "fn FPDF_InitLibrary();
             fn FPDF_DestroyLibrary();
             fn FPDF_GetLastError() -> c_ulong;",
        )
        .unwrap();
        assert_eq!(block.functions.len(), 3);
    }

    #[test]
    fn pointer_param_types() {
        let block = parse_block("fn FPDFBitmap_CreateEx(width: c_int, height: c_int, format: c_int, first_scan: *mut c_void, stride: c_int) -> FPDF_BITMAP;").unwrap();
        let f = &block.functions[0];
        assert_eq!(f.args.len(), 5);
        assert_eq!(f.args[3].name, "first_scan");
    }

    #[test]
    fn pointer_return_type() {
        let block =
            parse_block("fn FPDFBitmap_GetBuffer(bitmap: FPDF_BITMAP) -> *mut c_void;").unwrap();
        let f = &block.functions[0];
        assert!(f.ret.is_some());
    }

    #[test]
    fn empty_input() {
        let block = parse_block("").unwrap();
        assert!(block.functions.is_empty());
    }

    #[test]
    fn non_fn_item_errors() {
        assert!(parse_block("struct Foo;").is_err());
    }

    #[test]
    fn unknown_attribute_errors() {
        assert!(parse_block("#[version(7543)] fn Foo();").is_err());
    }

    #[test]
    fn missing_semicolon_errors() {
        assert!(parse_block("fn Foo()").is_err());
    }
}
