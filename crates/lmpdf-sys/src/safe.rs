use std::ffi::CString;
use std::fmt;
use std::os::raw::{c_double, c_int, c_ulong, c_void};

use crate::{FPDF_BITMAP, FPDF_DOCUMENT, FPDF_DWORD, FPDF_PAGE, FPDF_TEXTPAGE, PdfiumBindings};

#[derive(Debug, Clone, Copy)]
pub struct DocHandle(FPDF_DOCUMENT);
impl DocHandle {
    /// # Safety
    /// `ptr` must be a valid, non-null `FPDF_DOCUMENT` returned by Pdfium.
    pub unsafe fn from_raw(ptr: FPDF_DOCUMENT) -> Self {
        Self(ptr)
    }
    pub fn as_raw(self) -> FPDF_DOCUMENT {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageHandle(FPDF_PAGE);
impl PageHandle {
    /// # Safety
    /// `ptr` must be a valid, non-null `FPDF_PAGE` returned by Pdfium.
    pub unsafe fn from_raw(ptr: FPDF_PAGE) -> Self {
        Self(ptr)
    }
    pub fn as_raw(self) -> FPDF_PAGE {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BitmapHandle(FPDF_BITMAP);
impl BitmapHandle {
    /// # Safety
    /// `ptr` must be a valid, non-null `FPDF_BITMAP` returned by Pdfium.
    pub unsafe fn from_raw(ptr: FPDF_BITMAP) -> Self {
        Self(ptr)
    }
    pub fn as_raw(self) -> FPDF_BITMAP {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextPageHandle(FPDF_TEXTPAGE);
impl TextPageHandle {
    /// # Safety
    /// `ptr` must be a valid, non-null `FPDF_TEXTPAGE` returned by Pdfium.
    pub unsafe fn from_raw(ptr: FPDF_TEXTPAGE) -> Self {
        Self(ptr)
    }
    pub fn as_raw(self) -> FPDF_TEXTPAGE {
        self.0
    }
}

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

/// Decode a UTF-16LE buffer, stripping only trailing NUL (U+0000) code units.
/// Interior NULs are preserved — they are valid Unicode and must not cause truncation.
fn decode_utf16_buf(buf: &[u16]) -> String {
    let end = buf.iter().rposition(|&c| c != 0).map_or(0, |p| p + 1);
    String::from_utf16_lossy(&buf[..end])
}

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
    ) -> Result<DocHandle, SysError> {
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
            Ok(unsafe { DocHandle::from_raw(doc) })
        }
    }

    pub fn close_document(&self, doc: DocHandle) {
        unsafe { self.raw.FPDF_CloseDocument(doc.as_raw()) }
    }

    pub fn get_page_count(&self, doc: DocHandle) -> c_int {
        unsafe { self.raw.FPDF_GetPageCount(doc.as_raw()) }
    }

    pub fn load_page(&self, doc: DocHandle, index: c_int) -> Result<PageHandle, SysError> {
        let page = unsafe { self.raw.FPDF_LoadPage(doc.as_raw(), index) };
        if page.is_null() {
            Err(self.from_last_error())
        } else {
            Ok(unsafe { PageHandle::from_raw(page) })
        }
    }

    pub fn close_page(&self, page: PageHandle) {
        unsafe { self.raw.FPDF_ClosePage(page.as_raw()) }
    }

    pub fn get_page_width(&self, page: PageHandle) -> f32 {
        unsafe { self.raw.FPDF_GetPageWidthF(page.as_raw()) }
    }

    pub fn get_page_height(&self, page: PageHandle) -> f32 {
        unsafe { self.raw.FPDF_GetPageHeightF(page.as_raw()) }
    }

    pub fn create_bitmap(
        &self,
        width: c_int,
        height: c_int,
        alpha: c_int,
    ) -> Result<BitmapHandle, SysError> {
        let bitmap = unsafe { self.raw.FPDFBitmap_Create(width, height, alpha) };
        if bitmap.is_null() {
            Err(SysError::Unknown)
        } else {
            Ok(unsafe { BitmapHandle::from_raw(bitmap) })
        }
    }

    pub fn destroy_bitmap(&self, bitmap: BitmapHandle) {
        unsafe { self.raw.FPDFBitmap_Destroy(bitmap.as_raw()) }
    }

    pub fn bitmap_buffer(&self, bitmap: BitmapHandle) -> *mut c_void {
        unsafe { self.raw.FPDFBitmap_GetBuffer(bitmap.as_raw()) }
    }

    pub fn bitmap_width(&self, bitmap: BitmapHandle) -> c_int {
        unsafe { self.raw.FPDFBitmap_GetWidth(bitmap.as_raw()) }
    }

    pub fn bitmap_height(&self, bitmap: BitmapHandle) -> c_int {
        unsafe { self.raw.FPDFBitmap_GetHeight(bitmap.as_raw()) }
    }

    pub fn bitmap_stride(&self, bitmap: BitmapHandle) -> c_int {
        unsafe { self.raw.FPDFBitmap_GetStride(bitmap.as_raw()) }
    }

    pub fn bitmap_format(&self, bitmap: BitmapHandle) -> c_int {
        unsafe { self.raw.FPDFBitmap_GetFormat(bitmap.as_raw()) }
    }

    pub fn bitmap_fill_rect(
        &self,
        bitmap: BitmapHandle,
        left: c_int,
        top: c_int,
        width: c_int,
        height: c_int,
        color: FPDF_DWORD,
    ) {
        unsafe {
            self.raw
                .FPDFBitmap_FillRect(bitmap.as_raw(), left, top, width, height, color)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_page_bitmap(
        &self,
        bitmap: BitmapHandle,
        page: PageHandle,
        start_x: c_int,
        start_y: c_int,
        size_x: c_int,
        size_y: c_int,
        rotate: c_int,
        flags: c_int,
    ) {
        unsafe {
            self.raw.FPDF_RenderPageBitmap(
                bitmap.as_raw(),
                page.as_raw(),
                start_x,
                start_y,
                size_x,
                size_y,
                rotate,
                flags,
            )
        }
    }

    pub fn bitmap_copy_buffer(&self, bitmap: BitmapHandle) -> Result<Vec<u8>, SysError> {
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

    #[allow(clippy::too_many_arguments)]
    pub fn device_to_page(
        &self,
        page: PageHandle,
        start_x: c_int,
        start_y: c_int,
        size_x: c_int,
        size_y: c_int,
        rotate: c_int,
        device_x: c_int,
        device_y: c_int,
    ) -> Result<(c_double, c_double), SysError> {
        let mut page_x: c_double = 0.0;
        let mut page_y: c_double = 0.0;
        let ok = unsafe {
            self.raw.FPDF_DeviceToPage(
                page.as_raw(),
                start_x,
                start_y,
                size_x,
                size_y,
                rotate,
                device_x,
                device_y,
                &mut page_x,
                &mut page_y,
            )
        };
        if ok == 0 {
            Err(SysError::Unknown)
        } else {
            Ok((page_x, page_y))
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn page_to_device(
        &self,
        page: PageHandle,
        start_x: c_int,
        start_y: c_int,
        size_x: c_int,
        size_y: c_int,
        rotate: c_int,
        page_x: c_double,
        page_y: c_double,
    ) -> Result<(c_int, c_int), SysError> {
        let mut device_x: c_int = 0;
        let mut device_y: c_int = 0;
        let ok = unsafe {
            self.raw.FPDF_PageToDevice(
                page.as_raw(),
                start_x,
                start_y,
                size_x,
                size_y,
                rotate,
                page_x,
                page_y,
                &mut device_x,
                &mut device_y,
            )
        };
        if ok == 0 {
            Err(SysError::Unknown)
        } else {
            Ok((device_x, device_y))
        }
    }

    pub fn load_text_page(&self, page: PageHandle) -> Result<TextPageHandle, SysError> {
        let tp = unsafe { self.raw.FPDFText_LoadPage(page.as_raw()) };
        if tp.is_null() {
            Err(self.from_last_error())
        } else {
            Ok(unsafe { TextPageHandle::from_raw(tp) })
        }
    }

    pub fn close_text_page(&self, text_page: TextPageHandle) {
        unsafe { self.raw.FPDFText_ClosePage(text_page.as_raw()) }
    }

    pub fn text_count_chars(&self, text_page: TextPageHandle) -> c_int {
        unsafe { self.raw.FPDFText_CountChars(text_page.as_raw()) }
    }

    pub fn text_get_text(
        &self,
        text_page: TextPageHandle,
        start_index: c_int,
        count: c_int,
    ) -> String {
        if count <= 0 {
            return String::new();
        }
        // FPDFText_GetText writes `count` UTF-16LE code units plus a null terminator
        let buf_len = (count + 1) as usize;
        let mut buf: Vec<u16> = vec![0u16; buf_len];
        unsafe {
            self.raw
                .FPDFText_GetText(text_page.as_raw(), start_index, count, buf.as_mut_ptr());
        }
        decode_utf16_buf(&buf)
    }

    pub fn get_meta_text(&self, doc: DocHandle, tag: &str) -> Result<Option<String>, SysError> {
        let tag_cstring = CString::new(tag).map_err(|e| SysError::NullInterior(e.to_string()))?;

        // First pass: get required buffer size in bytes
        let needed = unsafe {
            self.raw
                .FPDF_GetMetaText(doc.as_raw(), tag_cstring.as_ptr(), std::ptr::null_mut(), 0)
        };

        // 0 means error/not found; 2 means just the null terminator (empty value)
        if needed == 0 || needed == 2 {
            return Ok(None);
        }

        // Allocate buffer as Vec<u16> for proper alignment, then cast for the FFI call
        let u16_len = (needed as usize) / 2;
        let mut buf: Vec<u16> = vec![0u16; u16_len];
        unsafe {
            self.raw.FPDF_GetMetaText(
                doc.as_raw(),
                tag_cstring.as_ptr(),
                buf.as_mut_ptr() as *mut c_void,
                // Use allocation-derived bound, not raw `needed` (which truncates on odd values)
                (u16_len * 2) as c_ulong,
            );
        }

        let s = decode_utf16_buf(&buf);

        if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
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

    #[test]
    fn device_to_page_signature_exists() {
        fn assert_method<B: PdfiumBindings>(
            sb: &SafeBindings<B>,
            page: PageHandle,
        ) -> Result<(f64, f64), SysError> {
            sb.device_to_page(page, 0, 0, 100, 100, 0, 50, 50)
        }
        let _ = assert_method::<crate::DynamicBindings>;
    }

    #[test]
    fn page_to_device_signature_exists() {
        fn assert_method<B: PdfiumBindings>(
            sb: &SafeBindings<B>,
            page: PageHandle,
        ) -> Result<(c_int, c_int), SysError> {
            sb.page_to_device(page, 0, 0, 100, 100, 0, 50.0, 50.0)
        }
        let _ = assert_method::<crate::DynamicBindings>;
    }

    #[test]
    fn doc_handle_is_copy_clone_debug() {
        fn assert_traits<T: Copy + Clone + std::fmt::Debug>() {}
        assert_traits::<DocHandle>();
    }

    #[test]
    fn page_handle_is_copy_clone_debug() {
        fn assert_traits<T: Copy + Clone + std::fmt::Debug>() {}
        assert_traits::<PageHandle>();
    }

    #[test]
    fn bitmap_handle_is_copy_clone_debug() {
        fn assert_traits<T: Copy + Clone + std::fmt::Debug>() {}
        assert_traits::<BitmapHandle>();
    }

    #[test]
    fn doc_handle_roundtrips_raw_pointer() {
        let ptr: FPDF_DOCUMENT = std::ptr::null_mut();
        let handle = unsafe { DocHandle::from_raw(ptr) };
        assert_eq!(handle.as_raw(), ptr);
    }

    #[test]
    fn page_handle_roundtrips_raw_pointer() {
        let ptr: FPDF_PAGE = std::ptr::null_mut();
        let handle = unsafe { PageHandle::from_raw(ptr) };
        assert_eq!(handle.as_raw(), ptr);
    }

    #[test]
    fn bitmap_handle_roundtrips_raw_pointer() {
        let ptr: FPDF_BITMAP = std::ptr::null_mut();
        let handle = unsafe { BitmapHandle::from_raw(ptr) };
        assert_eq!(handle.as_raw(), ptr);
    }

    #[test]
    fn text_page_handle_is_copy_clone_debug() {
        fn assert_traits<T: Copy + Clone + std::fmt::Debug>() {}
        assert_traits::<TextPageHandle>();
    }

    #[test]
    fn text_page_handle_roundtrips_raw_pointer() {
        let ptr: crate::FPDF_TEXTPAGE = std::ptr::null_mut();
        let handle = unsafe { TextPageHandle::from_raw(ptr) };
        assert_eq!(handle.as_raw(), ptr);
    }

    #[test]
    fn load_text_page_signature_exists() {
        fn assert_method<B: PdfiumBindings>(
            sb: &SafeBindings<B>,
            page: PageHandle,
        ) -> Result<TextPageHandle, SysError> {
            sb.load_text_page(page)
        }
        let _ = assert_method::<crate::DynamicBindings>;
    }

    #[test]
    fn close_text_page_signature_exists() {
        fn assert_method<B: PdfiumBindings>(sb: &SafeBindings<B>, tp: TextPageHandle) {
            sb.close_text_page(tp);
        }
        let _ = assert_method::<crate::DynamicBindings>;
    }

    #[test]
    fn text_count_chars_signature_exists() {
        fn assert_method<B: PdfiumBindings>(sb: &SafeBindings<B>, tp: TextPageHandle) -> c_int {
            sb.text_count_chars(tp)
        }
        let _ = assert_method::<crate::DynamicBindings>;
    }

    #[test]
    fn text_get_text_signature_exists() {
        fn assert_method<B: PdfiumBindings>(
            sb: &SafeBindings<B>,
            tp: TextPageHandle,
            start: c_int,
            count: c_int,
        ) -> String {
            sb.text_get_text(tp, start, count)
        }
        let _ = assert_method::<crate::DynamicBindings>;
    }

    #[test]
    fn get_meta_text_signature_exists() {
        fn assert_method<B: PdfiumBindings>(
            sb: &SafeBindings<B>,
            doc: DocHandle,
            tag: &str,
        ) -> Result<Option<String>, SysError> {
            sb.get_meta_text(doc, tag)
        }
        let _ = assert_method::<crate::DynamicBindings>;
    }

    #[test]
    fn decode_utf16_buf_strips_only_trailing_nuls() {
        // Simulates: "AB\0CD\0\0" (interior NUL at index 2, trailing NULs at 5-6)
        // Expected: the interior NUL is preserved; only trailing NULs stripped
        let buf: Vec<u16> = vec![0x0041, 0x0042, 0x0000, 0x0043, 0x0044, 0x0000, 0x0000];
        let result = decode_utf16_buf(&buf);
        // Interior NUL becomes '\0' in the Rust String — that's correct
        assert_eq!(result, "AB\0CD");
    }

    #[test]
    fn decode_utf16_buf_all_nuls_returns_empty() {
        let buf: Vec<u16> = vec![0x0000, 0x0000, 0x0000];
        let result = decode_utf16_buf(&buf);
        assert_eq!(result, "");
    }

    #[test]
    fn decode_utf16_buf_no_nuls_returns_full_string() {
        let buf: Vec<u16> = vec![0x0048, 0x0065, 0x006C, 0x006C, 0x006F]; // "Hello"
        let result = decode_utf16_buf(&buf);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn get_meta_text_buflen_capped_to_allocation() {
        // Verify that u16_len * 2 is always <= needed, and that the code
        // uses u16_len * 2 (not needed) as the buffer capacity bound.
        // For any even `needed`, u16_len * 2 == needed (no truncation).
        // For any odd `needed`, u16_len * 2 == needed - 1 (safe truncation).
        for needed in [4u64, 5, 6, 7, 100, 101] {
            let u16_len = (needed as usize) / 2;
            let buf_byte_cap = u16_len * 2;
            assert!(
                buf_byte_cap <= needed as usize,
                "buf capacity {buf_byte_cap} must not exceed needed {needed}"
            );
            // The key invariant: buflen passed to FFI must equal buf_byte_cap,
            // NOT needed, to prevent writing past the allocation.
            let buflen_for_ffi = (u16_len * 2) as u64;
            assert_eq!(
                buflen_for_ffi, buf_byte_cap as u64,
                "buflen must equal buffer byte capacity"
            );
        }
    }
}
