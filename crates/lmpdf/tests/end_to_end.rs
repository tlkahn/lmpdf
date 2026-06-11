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
    assert_eq!(bm.width(), 1224);
    assert_eq!(bm.height(), 1584);
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

// --- Coordinate conversion tests ---

#[test]
#[ignore]
fn device_to_page_top_left() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().scale(1.0);
    let (px, py) = doc.device_to_page(r, &cfg, 0, 0).unwrap();
    assert!((px - 0.0).abs() < 1.0);
    assert!((py - 792.0).abs() < 1.0);
}

#[test]
#[ignore]
fn page_to_device_origin() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().scale(1.0);
    let (dx, dy) = doc.page_to_device(r, &cfg, 0.0, 792.0).unwrap();
    assert!((dx as f64).abs() < 2.0);
    assert!((dy as f64).abs() < 2.0);
}

#[test]
#[ignore]
fn round_trip_device_to_page_to_device() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().scale(1.0);
    for (dx, dy) in [(100, 200), (0, 0), (300, 400)] {
        let (px, py) = doc.device_to_page(r, &cfg, dx, dy).unwrap();
        let (dx2, dy2) = doc.page_to_device(r, &cfg, px, py).unwrap();
        assert!((dx2 - dx).abs() <= 1);
        assert!((dy2 - dy).abs() <= 1);
    }
}

#[test]
#[ignore]
fn round_trip_page_to_device_to_page() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().scale(1.0);
    for (px, py) in [(100.0, 200.0), (0.0, 0.0), (306.0, 396.0)] {
        let (dx, dy) = doc.page_to_device(r, &cfg, px, py).unwrap();
        let (px2, py2) = doc.device_to_page(r, &cfg, dx, dy).unwrap();
        assert!((px2 - px).abs() < 2.0);
        assert!((py2 - py).abs() < 2.0);
    }
}

#[test]
#[ignore]
fn coord_conversion_with_rotation() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().scale(1.0).rotation(Rotation::Degrees90);
    for (dx, dy) in [(100, 200), (50, 50)] {
        let (px, py) = doc.device_to_page(r, &cfg, dx, dy).unwrap();
        let (dx2, dy2) = doc.page_to_device(r, &cfg, px, py).unwrap();
        assert!((dx2 - dx).abs() <= 1);
        assert!((dy2 - dy).abs() <= 1);
    }
}

#[test]
#[ignore]
fn coord_conversion_with_scale() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let r = doc.page(0).unwrap();
    let cfg = RenderConfig::new().scale(3.0);
    for (dx, dy) in [(150, 300), (0, 0)] {
        let (px, py) = doc.device_to_page(r, &cfg, dx, dy).unwrap();
        let (dx2, dy2) = doc.page_to_device(r, &cfg, px, py).unwrap();
        assert!((dx2 - dx).abs() <= 1);
        assert!((dy2 - dy).abs() <= 1);
    }
}

#[test]
#[ignore]
fn coord_conversion_cross_document_error() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc1 = p.load_document(hello_pdf(), None).unwrap();
    let doc2 = p.load_document(hello_pdf(), None).unwrap();
    let r1 = doc1.page(0).unwrap();
    let cfg = RenderConfig::default();
    let result = doc2.device_to_page(r1, &cfg, 0, 0);
    assert!(matches!(
        result,
        Err(Error::Handle(HandleError::CrossDocument))
    ));
    let result = doc2.page_to_device(r1, &cfg, 0.0, 0.0);
    assert!(matches!(
        result,
        Err(Error::Handle(HandleError::CrossDocument))
    ));
}

// --- Text extraction tests ---

fn born_digital_pdf() -> &'static [u8] {
    include_bytes!("../../lmpdf-sys/tests/fixtures/born_digital.pdf")
}

fn scanned_pdf() -> &'static [u8] {
    include_bytes!("../../lmpdf-sys/tests/fixtures/scanned.pdf")
}

#[test]
#[ignore]
fn test_page_text_born_digital() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(born_digital_pdf(), None).unwrap();
    let text = doc.page_text(0).unwrap();
    assert!(
        !text.is_empty(),
        "page_text(0) should return non-empty text"
    );
    assert!(
        text.contains("comprehensive survey of neural retrieval models"),
        "expected substring not found in page 0 text"
    );
}

#[test]
#[ignore]
fn test_page_text_scanned_empty() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(scanned_pdf(), None).unwrap();
    let text = doc.page_text(0).unwrap();
    assert!(
        text.trim().is_empty(),
        "scanned PDF page_text should be empty or whitespace, got: {text:?}"
    );
}

#[test]
#[ignore]
fn test_page_text_out_of_bounds() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let result = doc.page_text(999);
    assert!(result.is_err(), "page_text(999) should return Err");
}

// --- Metadata tests ---

#[test]
#[ignore]
fn test_meta_returns_title() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(born_digital_pdf(), None).unwrap();
    let title = doc.meta("Title").unwrap();
    assert_eq!(
        title,
        Some("Advances in Neural Retrieval Models for Scholarly Document Processing".to_string()),
        "meta('Title') should return the PDF title"
    );
}

#[test]
#[ignore]
fn test_meta_missing_returns_none() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(born_digital_pdf(), None).unwrap();
    let result = doc.meta("NonexistentKey").unwrap();
    assert_eq!(result, None, "meta for missing key should return None");
}

#[test]
#[ignore]
fn test_info_collects_known_keys() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(born_digital_pdf(), None).unwrap();
    let info = doc.info().unwrap();
    assert!(
        info.contains_key("Title"),
        "info() should contain 'Title' key, got keys: {:?}",
        info.keys().collect::<Vec<_>>()
    );
    assert!(
        info.contains_key("Author"),
        "info() should contain 'Author' key, got keys: {:?}",
        info.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        info.get("Title").unwrap(),
        "Advances in Neural Retrieval Models for Scholarly Document Processing"
    );
    assert_eq!(
        info.get("Author").unwrap(),
        "Dr. Elena Vasquez and Prof. Martin Chen"
    );
}

// --- delete_page / save_to_vec / truncate tests ---

#[test]
#[ignore]
fn delete_page_decrements_page_count() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let mut doc = p.load_document(born_digital_pdf(), None).unwrap();
    let original = doc.page_count();
    assert!(
        original >= 2,
        "need multi-page PDF for this test, got {original}"
    );
    doc.delete_page(0).unwrap();
    assert_eq!(doc.page_count(), original - 1);
}

#[test]
#[ignore]
fn truncate_removes_lead_and_trail_pages() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let mut doc = p.load_document(born_digital_pdf(), None).unwrap();
    let original = doc.page_count();
    assert!(
        original >= 3,
        "need >= 3 pages for truncate(1,1) test, got {original}"
    );
    doc.truncate(1, 1).unwrap();
    assert_eq!(doc.page_count(), original - 2);
}

#[test]
#[ignore]
fn truncate_excessive_returns_error() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let mut doc = p.load_document(hello_pdf(), None).unwrap();
    assert_eq!(doc.page_count(), 1);
    let result = doc.truncate(1, 0);
    assert!(result.is_err(), "truncate(1,0) on 1-page doc should fail");
}

#[test]
#[ignore]
fn save_after_delete_produces_valid_pdf() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let mut doc = p.load_document(born_digital_pdf(), None).unwrap();
    let original = doc.page_count();
    assert!(original >= 2, "need multi-page PDF, got {original}");
    doc.delete_page(0).unwrap();
    let bytes = doc.save_to_vec().unwrap();
    assert!(bytes.starts_with(b"%PDF"));
    // Reload and verify
    let doc2 = p.load_document(&bytes, None).unwrap();
    assert_eq!(doc2.page_count(), original - 1);
}

#[test]
#[ignore]
fn save_to_vec_produces_valid_pdf() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let doc = p.load_document(hello_pdf(), None).unwrap();
    let bytes = doc.save_to_vec().unwrap();
    assert!(!bytes.is_empty(), "saved PDF should be non-empty");
    assert!(
        bytes.starts_with(b"%PDF"),
        "saved PDF should start with %PDF header"
    );
}

// --- delete_page / truncate PageRef preservation tests ---

#[test]
#[ignore]
fn delete_page_preserves_later_page_refs() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let mut doc = p.load_document(born_digital_pdf(), None).unwrap();
    let original = doc.page_count();
    assert!(original >= 3, "need >= 3 pages, got {original}");
    // Cache page at index 2
    let r2 = doc.page(2).unwrap();
    let w2 = doc.page_width(r2).unwrap();
    // Delete page 0 (earlier page)
    doc.delete_page(0).unwrap();
    // r2 should still resolve (not Stale), and width should match
    let w2_after = doc.page_width(r2).unwrap();
    assert_eq!(
        w2, w2_after,
        "page width should be unchanged after earlier page deleted"
    );
}

#[test]
#[ignore]
fn truncate_preserves_interior_page_refs() {
    let p = Pdfium::open(pdfium_path()).unwrap();
    let mut doc = p.load_document(born_digital_pdf(), None).unwrap();
    let original = doc.page_count();
    assert!(original >= 4, "need >= 4 pages, got {original}");
    // Cache an interior page
    let r = doc.page(2).unwrap();
    let w = doc.page_width(r).unwrap();
    // Truncate lead=1, trail=1
    doc.truncate(1, 1).unwrap();
    // The interior page ref should still be valid
    let w_after = doc.page_width(r).unwrap();
    assert_eq!(w, w_after, "interior page ref should survive truncation");
    assert_eq!(doc.page_count(), original - 2);
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
