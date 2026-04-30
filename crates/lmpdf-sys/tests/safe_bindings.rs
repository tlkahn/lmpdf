use lmpdf_sys::{DynamicBindings, SafeBindings};

fn make_bindings() -> SafeBindings<DynamicBindings> {
    let path = std::env::var("PDFIUM_PATH").expect("Set PDFIUM_PATH to run these tests");
    let lib = unsafe { libloading::Library::new(&path) }.unwrap();
    let raw = DynamicBindings::load(lib).unwrap();
    let sb = SafeBindings::new(raw);
    sb.init_library();
    sb
}

#[test]
#[ignore]
fn safe_load_mem_document_succeeds() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/hello.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    assert!(!doc.is_null());
    sb.close_document(doc);
    sb.destroy_library();
}

#[test]
#[ignore]
fn safe_load_mem_document_invalid_data_returns_error() {
    let sb = make_bindings();
    let result = sb.load_mem_document(b"not a pdf", None);
    assert!(result.is_err());
    sb.destroy_library();
}

#[test]
#[ignore]
fn safe_get_page_count() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/hello.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    let count = sb.get_page_count(doc);
    assert_eq!(count, 1);
    sb.close_document(doc);
    sb.destroy_library();
}

#[test]
#[ignore]
fn safe_load_page_succeeds() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/hello.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    let page = sb.load_page(doc, 0).unwrap();
    assert!(!page.is_null());
    sb.close_page(page);
    sb.close_document(doc);
    sb.destroy_library();
}

#[test]
#[ignore]
fn safe_get_page_dimensions() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/hello.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    let page = sb.load_page(doc, 0).unwrap();
    let width = sb.get_page_width(page);
    let height = sb.get_page_height(page);
    assert!(width > 0.0);
    assert!(height > 0.0);
    sb.close_page(page);
    sb.close_document(doc);
    sb.destroy_library();
}

#[test]
#[ignore]
fn safe_load_page_out_of_bounds_returns_error() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/hello.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    let result = sb.load_page(doc, 99);
    assert!(result.is_err());
    sb.close_document(doc);
    sb.destroy_library();
}
