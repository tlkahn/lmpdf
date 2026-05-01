pub use lmpdf_sys;

pub mod bitmap;
pub mod document;
pub mod error;
pub mod pdfium;
pub mod render;

pub use bitmap::{Bitmap, BitmapFormat};
pub use document::{Document, DocumentId, PageKey, PageRef};
pub use error::{Error, Result};
pub use pdfium::Pdfium;
pub use render::{RenderConfig, RenderFlags, Rotation};

#[cfg(test)]
mod tests {
    #[test]
    fn can_import_sys_types() {
        let _doc: lmpdf_sys::FPDF_DOCUMENT = std::ptr::null_mut();
    }

    #[test]
    fn result_alias_is_exported() {
        let _: crate::Result<()> = Ok(());
        let _: crate::Result<()> = Err(crate::error::Error::Library(
            crate::error::LibraryError::InitFailed,
        ));
    }
}
