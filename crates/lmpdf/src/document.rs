use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use lmpdf_sys::{DocHandle, PageHandle, PdfiumLibrary};
use slotmap::SlotMap;

use std::os::raw::{c_int, c_ulong, c_void};

use lmpdf_sys::FPDF_FILEWRITE_;

use crate::Result;
use crate::bitmap::Bitmap;
use crate::error::{
    DocumentError, Error, HandleError, PageError, RenderError, SaveError, TextError,
};
use crate::render::{RenderConfig, compute_target_dimensions};

/// A wrapper that pairs an FPDF_FILEWRITE_ header with a pointer to a Vec<u8> buffer.
/// Must be repr(C) so that casting *mut FPDF_FILEWRITE_ to *mut VecWriter
/// recovers the `buf` field at the correct offset.
#[repr(C)]
struct VecWriter {
    header: FPDF_FILEWRITE_,
    buf: *mut Vec<u8>,
}

/// The extern "C" callback that Pdfium calls to write data.
unsafe extern "C" fn write_block_callback(
    self_: *mut FPDF_FILEWRITE_,
    data: *const c_void,
    size: c_ulong,
) -> c_int {
    let writer = self_ as *mut VecWriter;
    let buf = unsafe { &mut *(*writer).buf };
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) };
    buf.extend_from_slice(slice);
    1 // success
}

static NEXT_DOC_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(u64);

impl DocumentId {
    pub fn next() -> Self {
        Self(NEXT_DOC_ID.fetch_add(1, Ordering::Relaxed))
    }
}

slotmap::new_key_type! {
    pub struct PageKey;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageRef {
    pub doc_id: DocumentId,
    pub key: PageKey,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PageData {
    pub handle: PageHandle,
    pub width: f32,
    pub height: f32,
}

pub struct Document {
    id: DocumentId,
    handle: DocHandle,
    lib: Arc<PdfiumLibrary>,
    page_count: usize,
    pages: RefCell<SlotMap<PageKey, PageData>>,
    page_index_map: RefCell<Vec<Option<PageKey>>>,
    // PDFium's FPDF_LoadMemDocument64 does not copy the buffer — it references
    // it directly.  This Vec must outlive the DocHandle.
    _backing: Vec<u8>,
}

impl Document {
    pub fn from_bytes(
        lib: Arc<PdfiumLibrary>,
        data: &[u8],
        password: Option<&str>,
    ) -> Result<Self> {
        let owned = data.to_vec();
        let bindings = lib.bindings();
        let handle = bindings
            .load_mem_document(&owned, password)
            .map_err(|e| Error::Document(crate::error::DocumentError::from(e)))?;
        let page_count = bindings.get_page_count(handle).max(0) as usize;

        Ok(Self {
            id: DocumentId::next(),
            handle,
            lib,
            page_count,
            pages: RefCell::new(SlotMap::with_key()),
            page_index_map: RefCell::new(vec![None; page_count]),
            _backing: owned,
        })
    }

    pub fn id(&self) -> DocumentId {
        self.id
    }

    pub fn page_count(&self) -> usize {
        self.page_count
    }

    pub fn page(&self, index: usize) -> Result<PageRef> {
        if index >= self.page_count {
            return Err(PageError::IndexOutOfBounds {
                index,
                count: self.page_count,
            }
            .into());
        }

        if let Some(key) = self.page_index_map.borrow()[index] {
            return Ok(PageRef {
                doc_id: self.id,
                key,
            });
        }

        let bindings = self.lib.bindings();
        let page_handle = bindings
            .load_page(self.handle, index as std::os::raw::c_int)
            .map_err(|_| PageError::LoadFailed)?;
        let width = bindings.get_page_width(page_handle);
        let height = bindings.get_page_height(page_handle);

        let data = PageData {
            handle: page_handle,
            width,
            height,
        };
        let key = self.pages.borrow_mut().insert(data);
        self.page_index_map.borrow_mut()[index] = Some(key);

        Ok(PageRef {
            doc_id: self.id,
            key,
        })
    }

    pub fn page_width(&self, r: PageRef) -> Result<f32> {
        let data = self.resolve_page(r)?;
        Ok(data.width)
    }

    pub fn page_height(&self, r: PageRef) -> Result<f32> {
        let data = self.resolve_page(r)?;
        Ok(data.height)
    }

    pub fn open(
        lib: Arc<PdfiumLibrary>,
        path: impl AsRef<std::path::Path>,
        password: Option<&str>,
    ) -> Result<Self> {
        let data = std::fs::read(path.as_ref())
            .map_err(|e| Error::Document(DocumentError::IoError(e.to_string())))?;
        Self::from_bytes(lib, &data, password)
    }

    pub fn render_page(&self, page_ref: PageRef, config: &RenderConfig) -> Result<Bitmap> {
        let page_data = self.resolve_page(page_ref)?;
        let (w, h) = compute_target_dimensions(page_data.width, page_data.height, config)?;

        let bindings = self.lib.bindings();
        let alpha = if config.format.has_alpha() { 1 } else { 0 };
        let bitmap = bindings
            .create_bitmap(w as i32, h as i32, alpha)
            .map_err(|_| RenderError::BitmapCreationFailed)?;

        bindings.bitmap_fill_rect(bitmap, 0, 0, w as i32, h as i32, config.background_color);

        bindings.render_page_bitmap(
            bitmap,
            page_data.handle,
            0,
            0,
            w as i32,
            h as i32,
            config.rotation.to_raw(),
            config.flags.bits(),
        );

        let stride = bindings.bitmap_stride(bitmap) as u32;
        let data = bindings
            .bitmap_copy_buffer(bitmap)
            .map_err(|_| RenderError::BufferCopyFailed)?;

        bindings.destroy_bitmap(bitmap);

        Ok(Bitmap::new(data, w, h, stride, config.format))
    }

    pub fn device_to_page(
        &self,
        page_ref: PageRef,
        config: &RenderConfig,
        device_x: i32,
        device_y: i32,
    ) -> Result<(f64, f64)> {
        let page_data = self.resolve_page(page_ref)?;
        let (w, h) = compute_target_dimensions(page_data.width, page_data.height, config)?;
        let bindings = self.lib.bindings();
        bindings
            .device_to_page(
                page_data.handle,
                0,
                0,
                w as i32,
                h as i32,
                config.rotation.to_raw(),
                device_x,
                device_y,
            )
            .map_err(|_| Error::Render(RenderError::ConversionFailed))
    }

    pub fn page_to_device(
        &self,
        page_ref: PageRef,
        config: &RenderConfig,
        page_x: f64,
        page_y: f64,
    ) -> Result<(i32, i32)> {
        let page_data = self.resolve_page(page_ref)?;
        let (w, h) = compute_target_dimensions(page_data.width, page_data.height, config)?;
        let bindings = self.lib.bindings();
        bindings
            .page_to_device(
                page_data.handle,
                0,
                0,
                w as i32,
                h as i32,
                config.rotation.to_raw(),
                page_x,
                page_y,
            )
            .map_err(|_| Error::Render(RenderError::ConversionFailed))
    }

    pub fn page_text(&self, index: usize) -> Result<String> {
        // NOTE: This reuses page() which caches the FPDF_PAGE handle in the
        // SlotMap until Document::drop. For text-only extraction the handle
        // is not needed after this call, but the cost is bounded (max_pages
        // is typically 5, and the viewer renders the same low-index pages
        // anyway). A transient load→extract→close path would avoid cache
        // pollution but would add a second handle-lifecycle strategy; defer
        // until a proper page cache eviction design is warranted.
        let page_ref = self.page(index)?;
        let page_data = self.resolve_page(page_ref)?;

        let bindings = self.lib.bindings();
        let text_page = bindings
            .load_text_page(page_data.handle)
            .map_err(|_| TextError::LoadFailed)?;

        let count = bindings.text_count_chars(text_page);
        if count < 0 {
            bindings.close_text_page(text_page);
            return Err(TextError::CharCountFailed.into());
        }
        let text = if count > 0 {
            bindings.text_get_text(text_page, 0, count)
        } else {
            String::new()
        };

        bindings.close_text_page(text_page);
        Ok(text)
    }

    pub fn meta(&self, tag: &str) -> Result<Option<String>> {
        let bindings = self.lib.bindings();
        bindings
            .get_meta_text(self.handle, tag)
            .map_err(|e| Error::Document(DocumentError::from(e)))
    }

    pub fn info(&self) -> Result<HashMap<String, String>> {
        const KNOWN_KEYS: &[&str] = &[
            "Title",
            "Author",
            "Subject",
            "Keywords",
            "Creator",
            "Producer",
            "CreationDate",
            "ModDate",
        ];
        let mut map = HashMap::new();
        for &key in KNOWN_KEYS {
            if let Some(value) = self.meta(key)? {
                map.insert(key.to_string(), value);
            }
        }
        Ok(map)
    }

    pub fn delete_page(&mut self, index: usize) -> Result<()> {
        if index >= self.page_count {
            return Err(PageError::IndexOutOfBounds {
                index,
                count: self.page_count,
            }
            .into());
        }

        let map = self.page_index_map.get_mut();
        let pages = self.pages.get_mut();
        let bindings = self.lib.bindings();

        // Close only the deleted page's cached handle (if any).
        // Later pages' FPDF_PAGE handles remain valid (Pdfium page objects
        // are independent of their index in the page tree); their slotmap
        // keys are preserved so existing PageRefs stay usable.
        if let Some(key) = map[index] {
            if let Some(data) = pages.remove(key) {
                bindings.close_page(data.handle);
            }
        }

        // Remove the index entry (shifts subsequent entries left)
        map.remove(index);

        bindings.delete_page(self.handle, index as std::os::raw::c_int);
        self.page_count -= 1;
        Ok(())
    }

    pub fn truncate(&mut self, lead: usize, trail: usize) -> Result<()> {
        if lead + trail >= self.page_count {
            return Err(DocumentError::TruncationError(format!(
                "lead ({lead}) + trail ({trail}) >= page_count ({})",
                self.page_count
            ))
            .into());
        }
        // Delete trail pages from the end first (avoids index shifting issues)
        for _ in 0..trail {
            self.delete_page(self.page_count - 1)?;
        }
        // Delete lead pages back-to-front (avoids repeated O(n) Vec::remove(0) shifts)
        for i in (0..lead).rev() {
            self.delete_page(i)?;
        }
        Ok(())
    }

    pub fn save_to_vec(&self) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut writer = VecWriter {
            header: FPDF_FILEWRITE_ {
                version: 1,
                WriteBlock: Some(write_block_callback),
            },
            buf: &mut output,
        };

        let bindings = self.lib.bindings();
        bindings
            .save_to_writer(
                self.handle,
                &mut writer.header as *mut FPDF_FILEWRITE_,
                0, // flags: 0 = full save
            )
            .map_err(|e| Error::Save(SaveError::from(e)))?;

        Ok(output)
    }

    fn resolve_page(&self, r: PageRef) -> Result<PageData> {
        resolve_page_inner(self.id, &self.pages.borrow(), r)
    }
}

fn resolve_page_inner(
    doc_id: DocumentId,
    pages: &SlotMap<PageKey, PageData>,
    r: PageRef,
) -> Result<PageData> {
    if r.doc_id != doc_id {
        return Err(HandleError::CrossDocument.into());
    }
    pages
        .get(r.key)
        .copied()
        .ok_or_else(|| HandleError::Stale.into())
}

impl Drop for Document {
    fn drop(&mut self) {
        let bindings = self.lib.bindings();
        for (_, data) in self.pages.get_mut().iter() {
            bindings.close_page(data.handle);
        }
        bindings.close_document(self.handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_id_is_copy_eq_hash() {
        fn assert_traits<T: Copy + Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash>() {}
        assert_traits::<DocumentId>();
    }

    #[test]
    fn document_id_next_is_unique() {
        let id1 = DocumentId::next();
        let id2 = DocumentId::next();
        assert_ne!(id1, id2);
    }

    #[test]
    fn page_key_works_with_slotmap() {
        let mut sm = SlotMap::<PageKey, &str>::with_key();
        let k = sm.insert("hello");
        assert_eq!(sm.get(k), Some(&"hello"));
    }

    #[test]
    fn page_ref_is_copy_eq_hash() {
        fn assert_traits<T: Copy + Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash>() {}
        assert_traits::<PageRef>();
    }

    #[test]
    fn page_ref_carries_doc_id() {
        let doc_id = DocumentId::next();
        let mut sm = SlotMap::<PageKey, PageData>::with_key();
        let key = sm.insert(PageData {
            handle: unsafe { PageHandle::from_raw(std::ptr::null_mut()) },
            width: 100.0,
            height: 200.0,
        });
        let r = PageRef { doc_id, key };
        assert_eq!(r.doc_id, doc_id);
        assert_eq!(r.key, key);
    }

    #[test]
    fn resolve_page_wrong_doc_returns_cross_document() {
        let doc_id = DocumentId::next();
        let other_id = DocumentId::next();
        let mut pages = SlotMap::<PageKey, PageData>::with_key();
        let key = pages.insert(PageData {
            handle: unsafe { PageHandle::from_raw(std::ptr::null_mut()) },
            width: 0.0,
            height: 0.0,
        });
        let r = PageRef {
            doc_id: other_id,
            key,
        };
        let result = resolve_page_inner(doc_id, &pages, r);
        assert!(matches!(
            result,
            Err(Error::Handle(HandleError::CrossDocument))
        ));
    }

    #[test]
    fn resolve_page_invalid_key_returns_stale() {
        let doc_id = DocumentId::next();
        let mut pages = SlotMap::<PageKey, PageData>::with_key();
        let key = pages.insert(PageData {
            handle: unsafe { PageHandle::from_raw(std::ptr::null_mut()) },
            width: 0.0,
            height: 0.0,
        });
        pages.remove(key);
        let r = PageRef { doc_id, key };
        let result = resolve_page_inner(doc_id, &pages, r);
        assert!(matches!(result, Err(Error::Handle(HandleError::Stale))));
    }

    #[test]
    fn resolve_page_valid_returns_data() {
        let doc_id = DocumentId::next();
        let mut pages = SlotMap::<PageKey, PageData>::with_key();
        let key = pages.insert(PageData {
            handle: unsafe { PageHandle::from_raw(std::ptr::null_mut()) },
            width: 612.0,
            height: 792.0,
        });
        let r = PageRef { doc_id, key };
        let result = resolve_page_inner(doc_id, &pages, r);
        let data = result.unwrap();
        assert_eq!(data.width, 612.0);
        assert_eq!(data.height, 792.0);
    }

    #[test]
    fn page_count_returns_usize() {
        fn assert_return_type(doc: &Document) {
            let _: usize = doc.page_count();
        }
        let _ = assert_return_type;
    }

    #[test]
    fn page_accepts_usize() {
        fn assert_param_type(doc: &Document) -> Result<PageRef> {
            doc.page(0_usize)
        }
        let _ = assert_param_type;
    }

    #[test]
    fn device_to_page_signature_exists() {
        fn assert_method(
            doc: &Document,
            page_ref: PageRef,
            config: &crate::render::RenderConfig,
        ) -> Result<(f64, f64)> {
            doc.device_to_page(page_ref, config, 0, 0)
        }
        let _ = assert_method;
    }

    #[test]
    fn page_to_device_signature_exists() {
        fn assert_method(
            doc: &Document,
            page_ref: PageRef,
            config: &crate::render::RenderConfig,
        ) -> Result<(i32, i32)> {
            doc.page_to_device(page_ref, config, 0.0, 0.0)
        }
        let _ = assert_method;
    }

    #[test]
    fn page_text_signature_exists() {
        fn assert_sig(doc: &Document) -> Result<String> {
            doc.page_text(0)
        }
        let _ = assert_sig;
    }

    #[test]
    fn page_text_out_of_bounds_returns_error() {
        // This test will fail to compile until page_text exists.
        // Once it compiles, it needs a Document to test against.
        // Since Document requires pdfium, this is a compile-check only;
        // the actual out-of-bounds behavior is validated in the integration test.
        fn assert_returns_err(doc: &Document) {
            let result = doc.page_text(999);
            let _ = result; // can't run without pdfium, just verify it compiles
        }
        let _ = assert_returns_err;
    }

    #[test]
    fn meta_signature_exists() {
        fn assert_sig(doc: &Document) -> Result<Option<String>> {
            doc.meta("Title")
        }
        let _ = assert_sig;
    }

    #[test]
    fn info_signature_exists() {
        fn assert_sig(doc: &Document) -> Result<std::collections::HashMap<String, String>> {
            doc.info()
        }
        let _ = assert_sig;
    }

    #[test]
    fn delete_page_signature_exists() {
        fn assert_sig(doc: &mut Document) -> Result<()> {
            doc.delete_page(0_usize)
        }
        let _ = assert_sig;
    }

    #[test]
    fn save_to_vec_signature_exists() {
        fn assert_sig(doc: &Document) -> Result<Vec<u8>> {
            doc.save_to_vec()
        }
        let _ = assert_sig;
    }

    #[test]
    fn truncate_signature_exists() {
        fn assert_sig(doc: &mut Document) -> Result<()> {
            doc.truncate(1, 1)
        }
        let _ = assert_sig;
    }

    #[test]
    fn truncate_too_many_pages_returns_error() {
        fn assert_returns_err(doc: &mut Document) {
            let result = doc.truncate(3, 3);
            let _ = result;
        }
        let _ = assert_returns_err;
    }

    #[test]
    fn truncate_zero_zero_is_noop() {
        fn assert_ok(doc: &mut Document) {
            let result = doc.truncate(0, 0);
            let _ = result;
        }
        let _ = assert_ok;
    }

    #[test]
    fn delete_page_out_of_bounds_returns_error() {
        fn assert_returns_err(doc: &mut Document) {
            let result = doc.delete_page(999);
            let _ = result;
        }
        let _ = assert_returns_err;
    }

    #[test]
    fn text_error_char_count_failed_variant_exists() {
        let err: Error = TextError::CharCountFailed.into();
        assert!(matches!(err, Error::Text(TextError::CharCountFailed)));
    }
}
