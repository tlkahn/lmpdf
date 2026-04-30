use lmpdf_sys::{DynamicBindings, PdfiumBindings, PdfiumLibrary};
use std::os::raw::c_void;

fn pdfium_path() -> String {
    std::env::var("PDFIUM_PATH").expect("Set PDFIUM_PATH to run dynamic loading tests")
}

#[test]
#[ignore]
fn load_init_destroy() {
    let path = pdfium_path();
    let lib = unsafe { libloading::Library::new(&path) }.unwrap();
    let bindings = DynamicBindings::load(lib).unwrap();
    unsafe {
        bindings.FPDF_InitLibrary();
        bindings.FPDF_DestroyLibrary();
    }
}

#[test]
#[ignore]
fn load_pdf_from_memory() {
    let path = pdfium_path();
    let lib = unsafe { libloading::Library::new(&path) }.unwrap();
    let bindings = DynamicBindings::load(lib).unwrap();
    unsafe {
        bindings.FPDF_InitLibrary();

        let pdf_bytes = include_bytes!("../tests/fixtures/hello.pdf");
        let doc = bindings.FPDF_LoadMemDocument(
            pdf_bytes.as_ptr() as *const c_void,
            pdf_bytes.len() as i32,
            std::ptr::null(),
        );

        if !doc.is_null() {
            let page_count = bindings.FPDF_GetPageCount(doc);
            assert!(page_count > 0);
            bindings.FPDF_CloseDocument(doc);
        }

        bindings.FPDF_DestroyLibrary();
    }
}

#[test]
#[ignore]
fn pdfium_library_open_and_drop() {
    let path = pdfium_path();
    let lib = PdfiumLibrary::open(&path).unwrap();
    let _ = lib.bindings();
}
