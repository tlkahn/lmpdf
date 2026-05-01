use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use lmpdf_sys::{DocHandle, PageHandle, PdfiumLibrary};
use slotmap::SlotMap;

use crate::Result;
use crate::bitmap::Bitmap;
use crate::error::{DocumentError, Error, HandleError, PageError, RenderError};
use crate::render::{RenderConfig, compute_target_dimensions};

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
}
