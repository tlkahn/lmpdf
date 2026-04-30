use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let version_dir = if cfg!(feature = "pdfium_7811") {
        "pdfium_7811"
    } else if cfg!(feature = "pdfium_7763") {
        "pdfium_7763"
    } else if cfg!(feature = "pdfium_7543") {
        "pdfium_7543"
    } else {
        panic!(
            "No pdfium version feature enabled. Enable one of: pdfium_7811, pdfium_7763, pdfium_7543"
        );
    };

    let include_dir = manifest_dir.join("include").join(version_dir);

    let headers: Vec<_> = std::fs::read_dir(&include_dir)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", include_dir.display()))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("h") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    let mut wrapper = String::new();
    for header in &headers {
        wrapper.push_str(&format!("#include \"{}\"\n", header.display()));
    }

    let wrapper_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("wrapper.h");
    std::fs::write(&wrapper_path, &wrapper).unwrap();

    let bindings = bindgen::Builder::default()
        .header(wrapper_path.to_str().unwrap())
        .clang_arg(format!("-I{}", include_dir.display()))
        .blocklist_function(".*")
        .allowlist_type("FPDF_.*")
        .allowlist_type("FS_.*")
        .allowlist_type("fpdf_.*")
        .allowlist_var("FPDF_.*")
        .layout_tests(false)
        .size_t_is_usize(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings.write_to_file(&out_path).unwrap();
}
