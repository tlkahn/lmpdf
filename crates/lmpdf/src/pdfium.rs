use std::path::Path;
use std::sync::Arc;

use lmpdf_sys::PdfiumLibrary;

use crate::document::Document;
use crate::error::{Error, LibraryError};
use crate::Result;

pub struct Pdfium {
    lib: Arc<PdfiumLibrary>,
}

impl Pdfium {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let lib = PdfiumLibrary::open(path)
            .map_err(|e| Error::Library(LibraryError::LoadFailed(e.to_string())))?;
        Ok(Self { lib: Arc::new(lib) })
    }

    pub fn load_document(&self, data: &[u8], password: Option<&str>) -> Result<Document> {
        Document::from_bytes(self.lib.clone(), data, password)
    }

    pub fn open_document(
        &self,
        path: impl AsRef<Path>,
        password: Option<&str>,
    ) -> Result<Document> {
        Document::open(self.lib.clone(), path, password)
    }
}
