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

#[test]
#[ignore]
fn safe_create_bitmap_succeeds() {
    let sb = make_bindings();
    let bitmap = sb.create_bitmap(100, 100, 1).unwrap();
    let w = sb.bitmap_width(bitmap);
    let h = sb.bitmap_height(bitmap);
    assert_eq!(w, 100);
    assert_eq!(h, 100);
    assert!(sb.bitmap_stride(bitmap) > 0);
    assert!(sb.bitmap_format(bitmap) > 0);
    sb.destroy_bitmap(bitmap);
    sb.destroy_library();
}

#[test]
#[ignore]
fn safe_fill_rect_no_crash() {
    let sb = make_bindings();
    let bitmap = sb.create_bitmap(100, 100, 1).unwrap();
    sb.bitmap_fill_rect(bitmap, 0, 0, 100, 100, 0xFFFFFFFF);
    sb.destroy_bitmap(bitmap);
    sb.destroy_library();
}

#[test]
#[ignore]
fn safe_render_page_bitmap_no_crash() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/hello.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    let page = sb.load_page(doc, 0).unwrap();
    let bitmap = sb.create_bitmap(200, 200, 1).unwrap();
    sb.bitmap_fill_rect(bitmap, 0, 0, 200, 200, 0xFFFFFFFF);
    sb.render_page_bitmap(bitmap, page, 0, 0, 200, 200, 0, 0);
    let data = sb.bitmap_copy_buffer(bitmap).unwrap();
    assert!(!data.is_empty());
    sb.destroy_bitmap(bitmap);
    sb.close_page(page);
    sb.close_document(doc);
    sb.destroy_library();
}

#[test]
#[ignore]
fn test_text_extraction_returns_nonempty_string() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/born_digital.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    let page = sb.load_page(doc, 0).unwrap();
    let text_page = sb.load_text_page(page).unwrap();
    let count = sb.text_count_chars(text_page);
    assert!(
        count > 0,
        "born_digital page 0 should have chars, got {count}"
    );
    let text = sb.text_get_text(text_page, 0, count);
    assert!(!text.is_empty(), "extracted text should not be empty");
    assert!(
        text.contains("comprehensive survey of neural retrieval models"),
        "expected substring not found in extracted text"
    );
    sb.close_text_page(text_page);
    sb.close_page(page);
    sb.close_document(doc);
    sb.destroy_library();
}

#[test]
#[ignore]
fn test_text_extraction_scanned_returns_empty() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/scanned.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    let page = sb.load_page(doc, 0).unwrap();
    let text_page = sb.load_text_page(page).unwrap();
    let count = sb.text_count_chars(text_page);
    assert!(count <= 0, "scanned PDF should have 0 chars, got {count}");
    let text = sb.text_get_text(text_page, 0, count.max(0));
    assert!(
        text.trim().is_empty(),
        "scanned PDF text should be empty, got: {text:?}"
    );
    sb.close_text_page(text_page);
    sb.close_page(page);
    sb.close_document(doc);
    sb.destroy_library();
}

#[test]
#[ignore]
fn test_meta_text_returns_title() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/born_digital.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    let title = sb.get_meta_text(doc, "Title").unwrap();
    assert_eq!(
        title,
        Some("Advances in Neural Retrieval Models for Scholarly Document Processing".to_string()),
        "Title metadata should match"
    );
    sb.close_document(doc);
    sb.destroy_library();
}

#[test]
#[ignore]
fn test_meta_text_missing_key_returns_none() {
    let sb = make_bindings();
    let pdf = include_bytes!("fixtures/born_digital.pdf");
    let doc = sb.load_mem_document(pdf, None).unwrap();
    let result = sb.get_meta_text(doc, "NonexistentKey").unwrap();
    assert_eq!(result, None, "missing key should return None");
    sb.close_document(doc);
    sb.destroy_library();
}
