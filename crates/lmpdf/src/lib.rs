pub use lmpdf_sys;

pub mod error;
pub mod document;
pub mod pdfium;

pub use document::{Document, DocumentId, PageKey, PageRef};
pub use error::Error;
pub use pdfium::Pdfium;

#[cfg(test)]
mod tests {
    #[test]
    fn can_import_sys_types() {
        let _doc: lmpdf_sys::FPDF_DOCUMENT = std::ptr::null_mut();
    }
}
