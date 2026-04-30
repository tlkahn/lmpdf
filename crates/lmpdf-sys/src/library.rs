use crate::{DynamicBindings, SafeBindings, SysError};

pub struct PdfiumLibrary {
    bindings: SafeBindings<DynamicBindings>,
}

impl PdfiumLibrary {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, SysError> {
        let lib = unsafe { libloading::Library::new(path.as_ref()) }
            .map_err(|e| SysError::LoadFailed(e.to_string()))?;
        let raw = DynamicBindings::load(lib).map_err(|e| SysError::LoadFailed(e.to_string()))?;
        let bindings = SafeBindings::new(raw);
        bindings.init_library();
        Ok(Self { bindings })
    }

    pub fn bindings(&self) -> &SafeBindings<DynamicBindings> {
        &self.bindings
    }
}

impl Drop for PdfiumLibrary {
    fn drop(&mut self) {
        self.bindings.destroy_library();
    }
}
