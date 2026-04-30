use lmpdf::error::{DocumentError, HandleError, PageError};
use lmpdf::{Bitmap, BitmapFormat, Error, Pdfium, RenderConfig, RenderFlags, Rotation};

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

// --- Rendering tests ---

#[test]
#[ignore]
fn render_page_default_config() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let bm = doc.render_page(r, &RenderConfig::default()).unwrap();
    assert_eq!(bm.width(), 612);
    assert_eq!(bm.height(), 792);
    assert_eq!(bm.format(), BitmapFormat::Bgra);
    assert!(!bm.data().is_empty());
}

#[test]
#[ignore]
fn render_page_custom_width() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().width(300);
    let bm = doc.render_page(r, &cfg).unwrap();
    assert_eq!(bm.width(), 300);
}

#[test]
#[ignore]
fn render_page_scale() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().scale(2.0);
    let bm = doc.render_page(r, &cfg).unwrap();
    assert_eq!(bm.width(), 1224);
    assert_eq!(bm.height(), 1584);
}

#[test]
#[ignore]
fn render_page_bgr_format() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().format(BitmapFormat::Bgr);
    let bm = doc.render_page(r, &cfg).unwrap();
    assert_eq!(bm.format(), BitmapFormat::Bgr);
    assert!(!bm.data().is_empty());
}

#[test]
#[ignore]
fn render_page_max_dimensions() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().max_width(200).max_height(200);
    let bm = doc.render_page(r, &cfg).unwrap();
    assert!(bm.width() <= 200);
    assert!(bm.height() <= 200);
}

#[test]
#[ignore]
fn render_page_background_color() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new()
        .width(10)
        .height(10)
        .background_color(0xFF0000FF)
        .no_annotations();
    let bm = doc.render_page(r, &cfg).unwrap();
    assert_eq!(bm.width(), 10);
}

#[test]
#[ignore]
fn render_page_cross_document_error() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc1 = p.load_document(hello_pdf(), None).unwrap();
    let doc2 = p.load_document(hello_pdf(), None).unwrap();
    let r1 = doc1.page(0).unwrap();
    let result = doc2.render_page(r1, &RenderConfig::default());
    assert!(matches!(
        result,
        Err(Error::Handle(HandleError::CrossDocument))
    ));
}

#[test]
#[ignore]
fn render_page_out_of_bounds() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let result = doc.page(99);
    assert!(matches!(
        result,
        Err(Error::Page(PageError::IndexOutOfBounds { .. }))
    ));
}

// --- Document::open / Pdfium::open_document tests ---

#[test]
#[ignore]
fn open_document_from_path() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p
        .open_document("crates/lmpdf-sys/tests/fixtures/hello.pdf", None)
        .unwrap();
    assert_eq!(doc.page_count(), 1);
}

#[test]
#[ignore]
fn open_document_not_found() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let result = p.open_document("/nonexistent/file.pdf", None);
    assert!(matches!(
        result,
        Err(Error::Document(DocumentError::IoError(_)))
    ));
}

#[test]
#[ignore]
fn open_document_invalid_file() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let result = p.open_document("Cargo.toml", None);
    assert!(matches!(
        result,
        Err(Error::Document(DocumentError::InvalidFormat))
    ));
}

// --- Re-export compile checks ---

#[test]
fn re_exports_compile_check() {
    fn _accepts_bitmap(_: Bitmap) {}
    fn _accepts_format(_: BitmapFormat) {}
    fn _accepts_config(_: RenderConfig) {}
    fn _accepts_flags(_: RenderFlags) {}
    fn _accepts_rotation(_: Rotation) {}
}
