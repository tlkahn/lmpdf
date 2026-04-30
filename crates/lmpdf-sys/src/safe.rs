use std::ffi::CString;
use std::fmt;
use std::os::raw::{c_int, c_ulong, c_void};

use crate::{FPDF_BITMAP, FPDF_DOCUMENT, FPDF_DWORD, FPDF_PAGE, PdfiumBindings};

#[derive(Debug, Clone)]
pub enum SysError {
    Unknown,
    FileNotFound,
    InvalidFormat,
    IncorrectPassword,
    UnsupportedSecurity,
    PageNotFound,
    NullInterior(String),
    LoadFailed(String),
}

impl SysError {
    pub fn from_error_code(code: c_ulong) -> Self {
        match code {
            2 => SysError::FileNotFound,
            3 => SysError::InvalidFormat,
            4 => SysError::IncorrectPassword,
            5 => SysError::UnsupportedSecurity,
            6 => SysError::PageNotFound,
            _ => SysError::Unknown,
        }
    }
}

impl fmt::Display for SysError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SysError::Unknown => write!(f, "unknown pdfium error"),
            SysError::FileNotFound => write!(f, "file not found"),
            SysError::InvalidFormat => write!(f, "invalid PDF format"),
            SysError::IncorrectPassword => write!(f, "incorrect password"),
            SysError::UnsupportedSecurity => write!(f, "unsupported security scheme"),
            SysError::PageNotFound => write!(f, "page not found"),
            SysError::NullInterior(s) => write!(f, "null byte in string: {s}"),
            SysError::LoadFailed(s) => write!(f, "library load failed: {s}"),
        }
    }
}

impl std::error::Error for SysError {}

pub struct SafeBindings<B> {
    raw: B,
}

impl<B> SafeBindings<B> {
    pub fn new(raw: B) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> &B {
        &self.raw
    }
}

impl<B: PdfiumBindings> SafeBindings<B> {
    pub fn init_library(&self) {
        unsafe { self.raw.FPDF_InitLibrary() }
    }

    pub fn destroy_library(&self) {
        unsafe { self.raw.FPDF_DestroyLibrary() }
    }

    pub fn from_last_error(&self) -> SysError {
        let code = unsafe { self.raw.FPDF_GetLastError() };
        SysError::from_error_code(code)
    }

    pub fn load_mem_document(
        &self,
        data: &[u8],
        password: Option<&str>,
    ) -> Result<FPDF_DOCUMENT, SysError> {
        let password_cstring = password
            .map(|p| CString::new(p).map_err(|e| SysError::NullInterior(e.to_string())))
            .transpose()?;
        let password_ptr = password_cstring
            .as_ref()
            .map_or(std::ptr::null(), |c| c.as_ptr());

        let doc = unsafe {
            self.raw.FPDF_LoadMemDocument64(
                data.as_ptr() as *const c_void,
                data.len(),
                password_ptr,
            )
        };

        if doc.is_null() {
            Err(self.from_last_error())
        } else {
            Ok(doc)
        }
    }

    pub fn close_document(&self, doc: FPDF_DOCUMENT) {
        unsafe { self.raw.FPDF_CloseDocument(doc) }
    }

    pub fn get_page_count(&self, doc: FPDF_DOCUMENT) -> c_int {
        unsafe { self.raw.FPDF_GetPageCount(doc) }
    }

    pub fn load_page(&self, doc: FPDF_DOCUMENT, index: c_int) -> Result<FPDF_PAGE, SysError> {
        let page = unsafe { self.raw.FPDF_LoadPage(doc, index) };
        if page.is_null() {
            Err(self.from_last_error())
        } else {
            Ok(page)
        }
    }

    pub fn close_page(&self, page: FPDF_PAGE) {
        unsafe { self.raw.FPDF_ClosePage(page) }
    }

    pub fn get_page_width(&self, page: FPDF_PAGE) -> f32 {
        unsafe { self.raw.FPDF_GetPageWidthF(page) }
    }

    pub fn get_page_height(&self, page: FPDF_PAGE) -> f32 {
        unsafe { self.raw.FPDF_GetPageHeightF(page) }
    }

    pub fn create_bitmap(
        &self,
        width: c_int,
        height: c_int,
        alpha: c_int,
    ) -> Result<FPDF_BITMAP, SysError> {
        let bitmap = unsafe { self.raw.FPDFBitmap_Create(width, height, alpha) };
        if bitmap.is_null() {
            Err(SysError::Unknown)
        } else {
            Ok(bitmap)
        }
    }

    pub fn destroy_bitmap(&self, bitmap: FPDF_BITMAP) {
        unsafe { self.raw.FPDFBitmap_Destroy(bitmap) }
    }

    pub fn bitmap_buffer(&self, bitmap: FPDF_BITMAP) -> *mut c_void {
        unsafe { self.raw.FPDFBitmap_GetBuffer(bitmap) }
    }

    pub fn bitmap_width(&self, bitmap: FPDF_BITMAP) -> c_int {
        unsafe { self.raw.FPDFBitmap_GetWidth(bitmap) }
    }

    pub fn bitmap_height(&self, bitmap: FPDF_BITMAP) -> c_int {
        unsafe { self.raw.FPDFBitmap_GetHeight(bitmap) }
    }

    pub fn bitmap_stride(&self, bitmap: FPDF_BITMAP) -> c_int {
        unsafe { self.raw.FPDFBitmap_GetStride(bitmap) }
    }

    pub fn bitmap_format(&self, bitmap: FPDF_BITMAP) -> c_int {
        unsafe { self.raw.FPDFBitmap_GetFormat(bitmap) }
    }

    pub fn bitmap_fill_rect(
        &self,
        bitmap: FPDF_BITMAP,
        left: c_int,
        top: c_int,
        width: c_int,
        height: c_int,
        color: FPDF_DWORD,
    ) {
        unsafe {
            self.raw
                .FPDFBitmap_FillRect(bitmap, left, top, width, height, color)
        }
    }

    pub fn render_page_bitmap(
        &self,
        bitmap: FPDF_BITMAP,
        page: FPDF_PAGE,
        start_x: c_int,
        start_y: c_int,
        size_x: c_int,
        size_y: c_int,
        rotate: c_int,
        flags: c_int,
    ) {
        unsafe {
            self.raw.FPDF_RenderPageBitmap(
                bitmap, page, start_x, start_y, size_x, size_y, rotate, flags,
            )
        }
    }

    pub fn bitmap_copy_buffer(&self, bitmap: FPDF_BITMAP) -> Result<Vec<u8>, SysError> {
        let buf = self.bitmap_buffer(bitmap);
        if buf.is_null() {
            return Err(SysError::Unknown);
        }
        let stride = self.bitmap_stride(bitmap);
        let height = self.bitmap_height(bitmap);
        let len = (stride * height) as usize;
        let slice = unsafe { std::slice::from_raw_parts(buf as *const u8, len) };
        Ok(slice.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sys_error_from_error_code_maps_correctly() {
        assert!(matches!(SysError::from_error_code(0), SysError::Unknown));
        assert!(matches!(SysError::from_error_code(1), SysError::Unknown));
        assert!(matches!(
            SysError::from_error_code(2),
            SysError::FileNotFound
        ));
        assert!(matches!(
            SysError::from_error_code(3),
            SysError::InvalidFormat
        ));
        assert!(matches!(
            SysError::from_error_code(4),
            SysError::IncorrectPassword
        ));
        assert!(matches!(
            SysError::from_error_code(5),
            SysError::UnsupportedSecurity
        ));
        assert!(matches!(
            SysError::from_error_code(6),
            SysError::PageNotFound
        ));
        assert!(matches!(SysError::from_error_code(99), SysError::Unknown));
    }

    #[test]
    fn sys_error_display_variants() {
        assert!(!SysError::Unknown.to_string().is_empty());
        assert!(!SysError::FileNotFound.to_string().is_empty());
        assert!(!SysError::InvalidFormat.to_string().is_empty());
        assert!(!SysError::IncorrectPassword.to_string().is_empty());
        assert!(!SysError::UnsupportedSecurity.to_string().is_empty());
        assert!(!SysError::PageNotFound.to_string().is_empty());
        assert!(!SysError::NullInterior("test".into()).to_string().is_empty());
        assert!(!SysError::LoadFailed("test".into()).to_string().is_empty());
    }

    #[test]
    fn sys_error_implements_std_error() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<SysError>();
    }

    #[test]
    fn sys_error_load_failed_stores_message() {
        let msg = "cannot find libpdfium.so";
        let err = SysError::LoadFailed(msg.to_string());
        assert!(err.to_string().contains(msg));
    }

    #[test]
    fn safe_bindings_signature_check() {
        fn accepts<B: PdfiumBindings>(_: &SafeBindings<B>) {}
        let _ = accepts::<crate::DynamicBindings>;
    }
}
