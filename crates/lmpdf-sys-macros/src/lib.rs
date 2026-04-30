mod gen_dynamic;
mod gen_static;
mod gen_trait;
mod gen_wasm;
mod ir;
mod parse;
mod version;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use gen_dynamic::generate_dynamic;
use gen_static::generate_static;
use gen_trait::generate_trait;
use gen_wasm::generate_wasm;
use parse::FfiBlock;

#[proc_macro]
pub fn pdfium_ffi(input: TokenStream) -> TokenStream {
    let block = parse_macro_input!(input as FfiBlock);
    let fns = &block.functions;

    let trait_tokens = generate_trait(fns);
    let dynamic_tokens = generate_dynamic(fns);
    let static_tokens = generate_static(fns);
    let wasm_tokens = generate_wasm(fns);

    quote! {
        #trait_tokens
        #dynamic_tokens
        #static_tokens
        #wasm_tokens
    }
    .into()
}
