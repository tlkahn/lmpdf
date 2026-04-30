#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod ffi;
pub use ffi::*;

mod library;
pub use library::PdfiumLibrary;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::{null, null_mut};

    #[test]
    fn bindgen_types_exist() {
        let _doc: FPDF_DOCUMENT = null_mut();
        let _page: FPDF_PAGE = null_mut();
        let _bitmap: FPDF_BITMAP = null_mut();
        let _textpage: FPDF_TEXTPAGE = null_mut();
        let _annot: FPDF_ANNOTATION = null_mut();
        let _form: FPDF_FORMHANDLE = null_mut();
        let _font: FPDF_FONT = null_mut();
        let _obj: FPDF_PAGEOBJECT = null_mut();
        let _bookmark: FPDF_BOOKMARK = null_mut();
        let _link: FPDF_LINK = null_mut();
        let _action: FPDF_ACTION = null_mut();
        let _dest: FPDF_DEST = null_mut();
        let _sig: FPDF_SIGNATURE = null();
        let _attach: FPDF_ATTACHMENT = null_mut();
    }

    #[test]
    fn types_are_distinct() {
        fn takes_doc(_: FPDF_DOCUMENT) {}
        fn takes_page(_: FPDF_PAGE) {}
        takes_doc(null_mut());
        takes_page(null_mut());
    }

    #[test]
    fn value_types_exist() {
        let m = FS_MATRIX {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        };
        assert_eq!(m.a, 1.0);

        let r = FS_RECTF {
            left: 0.0,
            top: 100.0,
            right: 200.0,
            bottom: 0.0,
        };
        assert_eq!(r.top, 100.0);

        let s = FS_SIZEF {
            width: 612.0,
            height: 792.0,
        };
        assert_eq!(s.width, 612.0);

        let p = FS_POINTF { x: 72.0, y: 72.0 };
        assert_eq!(p.x, 72.0);
    }

    #[test]
    fn constants_exist() {
        assert_eq!(FPDF_OBJECT_UNKNOWN, 0);
        assert_eq!(FPDF_OBJECT_BOOLEAN, 1);
        assert_eq!(FPDF_OBJECT_NUMBER, 2);
        assert_eq!(FPDF_OBJECT_STRING, 3);
        assert_eq!(FPDF_OBJECT_NAME, 4);
        assert_eq!(FPDF_OBJECT_ARRAY, 5);
        assert_eq!(FPDF_OBJECT_DICTIONARY, 6);
        assert_eq!(FPDF_OBJECT_STREAM, 7);
        assert_eq!(FPDF_OBJECT_NULLOBJ, 8);
        assert_eq!(FPDF_OBJECT_REFERENCE, 9);
    }

    #[test]
    fn type_aliases_exist() {
        let _b: FPDF_BOOL = 1;
        let _f: FS_FLOAT = 1.0;
        let _d: FPDF_DWORD = 0;
    }

    #[test]
    fn trait_can_be_named_as_bound() {
        fn accepts_bindings<T: PdfiumBindings>(_: &T) {}
        let _ = accepts_bindings::<DynamicBindings>;
    }

    #[test]
    fn dynamic_bindings_load_signature() {
        fn check_signature(
            _f: fn(libloading::Library) -> Result<DynamicBindings, libloading::Error>,
        ) {
        }
        check_signature(DynamicBindings::load);
    }

    #[test]
    fn pdfium_library_open_compiles() {
        fn accepts_result(_: Result<PdfiumLibrary, libloading::Error>) {}
        let r = PdfiumLibrary::open("/nonexistent/libpdfium.so");
        accepts_result(r);
    }
}
