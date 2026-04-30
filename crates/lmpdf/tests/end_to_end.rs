use lmpdf::error::{HandleError, PageError};
use lmpdf::{Error, Pdfium};

fn pdfium_path() -> String {
    std::env::var("PDFIUM_PATH").expect("Set PDFIUM_PATH to run these tests")
}

fn hello_pdf() -> &'static [u8] {
    include_bytes!("../../lmpdf-sys/tests/fixtures/hello.pdf")
}

#[test]
#[ignore]
fn pdfium_open_succeeds() {
    let _p = Pdfium::open(pdfium_path()).unwrap();
}

#[test]
#[ignore]
fn pdfium_open_invalid_path_returns_error() {
    let result = Pdfium::open("/nonexistent/libpdfium.dylib");
    assert!(result.is_err());
}

#[test]
#[ignore]
fn pdfium_load_document_succeeds() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let _doc = p.load_document(hello_pdf(), None).unwrap();
}

#[test]
#[ignore]
fn pdfium_load_document_page_count() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    assert_eq!(doc.page_count(), 1);
}

#[test]
#[ignore]
fn pdfium_load_document_invalid_bytes_returns_error() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let result = p.load_document(b"not a pdf", None);
    assert!(result.is_err());
}

#[test]
#[ignore]
fn page_returns_valid_ref() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let _page = doc.page(0).unwrap();
}

#[test]
#[ignore]
fn page_same_index_returns_same_ref() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r1 = doc.page(0).unwrap();
    let r2 = doc.page(0).unwrap();
    assert_eq!(r1, r2);
}

#[test]
#[ignore]
fn page_out_of_bounds_returns_error() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let result = doc.page(99);
    assert!(matches!(
        result,
        Err(Error::Page(PageError::IndexOutOfBounds { .. }))
    ));
}

#[test]
#[ignore]
fn page_width_returns_positive() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    assert!(doc.page_width(r).unwrap() > 0.0);
}

#[test]
#[ignore]
fn page_height_returns_positive() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    assert!(doc.page_height(r).unwrap() > 0.0);
}

#[test]
#[ignore]
fn document_drop_no_crash() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    drop(doc);
}

#[test]
#[ignore]
fn document_drop_with_pages_no_crash() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let _r = doc.page(0).unwrap();
    drop(doc);
}

#[test]
#[ignore]
fn end_to_end_full_flow() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    assert_eq!(doc.page_count(), 1);
    let r = doc.page(0).unwrap();
    let w = doc.page_width(r).unwrap();
    let h = doc.page_height(r).unwrap();
    assert!((w - 612.0).abs() < 1.0);
    assert!((h - 792.0).abs() < 1.0);
}

#[test]
#[ignore]
fn cross_document_handle_returns_error() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc1 = p.load_document(hello_pdf(), None).unwrap();
    let doc2 = p.load_document(hello_pdf(), None).unwrap();
    let r1 = doc1.page(0).unwrap();
    let result = doc2.page_width(r1);
    assert!(matches!(
        result,
        Err(Error::Handle(HandleError::CrossDocument))
    ));
}
