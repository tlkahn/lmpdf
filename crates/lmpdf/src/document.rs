use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use lmpdf_sys::{FPDF_DOCUMENT, FPDF_PAGE, PdfiumLibrary};
use slotmap::SlotMap;

use crate::error::{Error, HandleError, PageError};

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
    pub handle: FPDF_PAGE,
    pub width: f32,
    pub height: f32,
}

pub struct Document {
    id: DocumentId,
    handle: FPDF_DOCUMENT,
    lib: Arc<PdfiumLibrary>,
    page_count: i32,
    pages: RefCell<SlotMap<PageKey, PageData>>,
    page_index_map: RefCell<Vec<Option<PageKey>>>,
}

impl Document {
    pub fn from_bytes(
        lib: Arc<PdfiumLibrary>,
        data: &[u8],
        password: Option<&str>,
    ) -> Result<Self, Error> {
        let bindings = lib.bindings();
        let handle = bindings
            .load_mem_document(data, password)
            .map_err(|e| Error::Document(crate::error::DocumentError::from(e)))?;
        let page_count = bindings.get_page_count(handle).max(0);

        Ok(Self {
            id: DocumentId::next(),
            handle,
            lib,
            page_count,
            pages: RefCell::new(SlotMap::with_key()),
            page_index_map: RefCell::new(vec![None; page_count as usize]),
        })
    }

    pub fn id(&self) -> DocumentId {
        self.id
    }

    pub fn page_count(&self) -> i32 {
        self.page_count
    }

    pub fn page(&self, index: i32) -> Result<PageRef, Error> {
        if index < 0 || index >= self.page_count {
            return Err(PageError::IndexOutOfBounds {
                index,
                count: self.page_count,
            }
            .into());
        }

        let idx = index as usize;
        if let Some(key) = self.page_index_map.borrow()[idx] {
            return Ok(PageRef {
                doc_id: self.id,
                key,
            });
        }

        let bindings = self.lib.bindings();
        let page_handle = bindings
            .load_page(self.handle, index)
            .map_err(|_| PageError::LoadFailed)?;
        let width = bindings.get_page_width(page_handle);
        let height = bindings.get_page_height(page_handle);

        let data = PageData {
            handle: page_handle,
            width,
            height,
        };
        let key = self.pages.borrow_mut().insert(data);
        self.page_index_map.borrow_mut()[idx] = Some(key);

        Ok(PageRef {
            doc_id: self.id,
            key,
        })
    }

    pub fn page_width(&self, r: PageRef) -> Result<f32, Error> {
        let data = self.resolve_page(r)?;
        Ok(data.width)
    }

    pub fn page_height(&self, r: PageRef) -> Result<f32, Error> {
        let data = self.resolve_page(r)?;
        Ok(data.height)
    }

    fn resolve_page(&self, r: PageRef) -> Result<PageData, Error> {
        resolve_page_inner(self.id, &self.pages.borrow(), r)
    }
}

fn resolve_page_inner(
    doc_id: DocumentId,
    pages: &SlotMap<PageKey, PageData>,
    r: PageRef,
) -> Result<PageData, Error> {
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
            handle: std::ptr::null_mut(),
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
            handle: std::ptr::null_mut(),
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
            handle: std::ptr::null_mut(),
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
            handle: std::ptr::null_mut(),
            width: 612.0,
            height: 792.0,
        });
        let r = PageRef { doc_id, key };
        let result = resolve_page_inner(doc_id, &pages, r);
        let data = result.unwrap();
        assert_eq!(data.width, 612.0);
        assert_eq!(data.height, 792.0);
    }
}
