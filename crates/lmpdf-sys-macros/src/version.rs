use proc_macro2::TokenStream;
use quote::quote;

const VERSIONS: &[u32] = &[7543, 7763, 7811];

pub fn versions_since(since: u32) -> Vec<String> {
    let idx = VERSIONS
        .iter()
        .position(|&v| v == since)
        .unwrap_or_else(|| panic!("Unknown pdfium version: {since}. Known versions: {VERSIONS:?}"));
    VERSIONS[idx..]
        .iter()
        .map(|v| format!("pdfium_{v}"))
        .collect()
}

pub fn version_cfg_tokens(since: u32) -> TokenStream {
    let features = versions_since(since);
    if features.len() == 1 {
        let feat = &features[0];
        quote! { #[cfg(feature = #feat)] }
    } else {
        let feat_exprs: Vec<TokenStream> =
            features.iter().map(|f| quote! { feature = #f }).collect();
        quote! { #[cfg(any(#(#feat_exprs),*))] }
    }
}

pub fn version_cfg_tokens_opt(since: Option<u32>) -> TokenStream {
    match since {
        Some(v) => version_cfg_tokens(v),
        None => TokenStream::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_since_earliest() {
        assert_eq!(
            versions_since(7543),
            vec!["pdfium_7543", "pdfium_7763", "pdfium_7811"]
        );
    }

    #[test]
    fn versions_since_middle() {
        assert_eq!(versions_since(7763), vec!["pdfium_7763", "pdfium_7811"]);
    }

    #[test]
    fn versions_since_latest() {
        assert_eq!(versions_since(7811), vec!["pdfium_7811"]);
    }

    #[test]
    #[should_panic(expected = "Unknown pdfium version: 9999")]
    fn versions_since_unknown_panics() {
        versions_since(9999);
    }

    #[test]
    fn single_version_no_any() {
        let tokens = version_cfg_tokens(7811);
        let s = tokens.to_string();
        assert!(s.contains("feature = \"pdfium_7811\""));
        assert!(!s.contains("any"));
    }

    #[test]
    fn multiple_versions_use_any() {
        let tokens = version_cfg_tokens(7543);
        let s = tokens.to_string();
        assert!(s.contains("any"));
        assert!(s.contains("pdfium_7543"));
        assert!(s.contains("pdfium_7763"));
        assert!(s.contains("pdfium_7811"));
    }

    #[test]
    fn opt_none_is_empty() {
        let tokens = version_cfg_tokens_opt(None);
        assert!(tokens.is_empty());
    }
}
