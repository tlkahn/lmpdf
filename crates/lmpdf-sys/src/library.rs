use crate::{DynamicBindings, PdfiumBindings};

pub struct PdfiumLibrary {
    bindings: DynamicBindings,
}

impl PdfiumLibrary {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, libloading::Error> {
        let lib = unsafe { libloading::Library::new(path.as_ref()) }?;
        let bindings = DynamicBindings::load(lib)?;
        unsafe {
            bindings.FPDF_InitLibrary();
        }
        Ok(Self { bindings })
    }

    pub fn bindings(&self) -> &DynamicBindings {
        &self.bindings
    }
}

impl Drop for PdfiumLibrary {
    fn drop(&mut self) {
        unsafe {
            self.bindings.FPDF_DestroyLibrary();
        }
    }
}
