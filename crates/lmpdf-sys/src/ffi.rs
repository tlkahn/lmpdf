use std::os::raw::{c_double, c_int, c_ulong, c_void};

use super::{
    FPDF_BITMAP, FPDF_BOOL, FPDF_BYTESTRING, FPDF_DOCUMENT, FPDF_DWORD, FPDF_LIBRARY_CONFIG,
    FPDF_PAGE, FPDF_STRING, FS_MATRIX, FS_RECTF,
};

lmpdf_sys_macros::pdfium_ffi! {
    // Lifecycle
    fn FPDF_InitLibrary();
    fn FPDF_InitLibraryWithConfig(config: *const FPDF_LIBRARY_CONFIG);
    fn FPDF_DestroyLibrary();
    fn FPDF_GetLastError() -> c_ulong;

    // Document
    fn FPDF_LoadDocument(file_path: FPDF_STRING, password: FPDF_BYTESTRING) -> FPDF_DOCUMENT;
    fn FPDF_LoadMemDocument(data_buf: *const c_void, size: c_int, password: FPDF_BYTESTRING) -> FPDF_DOCUMENT;
    fn FPDF_LoadMemDocument64(data_buf: *const c_void, size: usize, password: FPDF_BYTESTRING) -> FPDF_DOCUMENT;
    fn FPDF_CloseDocument(document: FPDF_DOCUMENT);
    fn FPDF_GetPageCount(document: FPDF_DOCUMENT) -> c_int;
    fn FPDF_GetDocPermissions(document: FPDF_DOCUMENT) -> c_ulong;
    fn FPDF_GetFileVersion(document: FPDF_DOCUMENT, fileVersion: *mut c_int) -> FPDF_BOOL;

    // Page
    fn FPDF_LoadPage(document: FPDF_DOCUMENT, page_index: c_int) -> FPDF_PAGE;
    fn FPDF_ClosePage(page: FPDF_PAGE);
    fn FPDF_GetPageWidthF(page: FPDF_PAGE) -> f32;
    fn FPDF_GetPageHeightF(page: FPDF_PAGE) -> f32;
    fn FPDF_GetPageWidth(page: FPDF_PAGE) -> c_double;
    fn FPDF_GetPageHeight(page: FPDF_PAGE) -> c_double;
    fn FPDF_GetPageBoundingBox(page: FPDF_PAGE, rect: *mut FS_RECTF) -> FPDF_BOOL;

    // Bitmap
    fn FPDFBitmap_Create(width: c_int, height: c_int, alpha: c_int) -> FPDF_BITMAP;
    fn FPDFBitmap_CreateEx(width: c_int, height: c_int, format: c_int, first_scan: *mut c_void, stride: c_int) -> FPDF_BITMAP;
    fn FPDFBitmap_Destroy(bitmap: FPDF_BITMAP);
    fn FPDFBitmap_GetBuffer(bitmap: FPDF_BITMAP) -> *mut c_void;
    fn FPDFBitmap_GetWidth(bitmap: FPDF_BITMAP) -> c_int;
    fn FPDFBitmap_GetHeight(bitmap: FPDF_BITMAP) -> c_int;
    fn FPDFBitmap_GetStride(bitmap: FPDF_BITMAP) -> c_int;
    fn FPDFBitmap_GetFormat(bitmap: FPDF_BITMAP) -> c_int;
    fn FPDFBitmap_FillRect(bitmap: FPDF_BITMAP, left: c_int, top: c_int, width: c_int, height: c_int, color: FPDF_DWORD);

    // Render + Coord
    fn FPDF_RenderPageBitmap(bitmap: FPDF_BITMAP, page: FPDF_PAGE, start_x: c_int, start_y: c_int, size_x: c_int, size_y: c_int, rotate: c_int, flags: c_int);
    fn FPDF_RenderPageBitmapWithMatrix(bitmap: FPDF_BITMAP, page: FPDF_PAGE, matrix: *const FS_MATRIX, clipping: *const FS_RECTF, flags: c_int);
    fn FPDF_DeviceToPage(page: FPDF_PAGE, start_x: c_int, start_y: c_int, size_x: c_int, size_y: c_int, rotate: c_int, device_x: c_int, device_y: c_int, page_x: *mut c_double, page_y: *mut c_double) -> FPDF_BOOL;
    fn FPDF_PageToDevice(page: FPDF_PAGE, start_x: c_int, start_y: c_int, size_x: c_int, size_y: c_int, rotate: c_int, page_x: c_double, page_y: c_double, device_x: *mut c_int, device_y: *mut c_int) -> FPDF_BOOL;
}
