use std::fmt;

use lmpdf_sys::SysError;

#[derive(Debug)]
pub enum Error {
    Library(LibraryError),
    Document(DocumentError),
    Page(PageError),
    Handle(HandleError),
    Render(RenderError),
    Text(TextError),
    Save(SaveError),
}

#[derive(Debug)]
pub enum LibraryError {
    LoadFailed(String),
    SymbolNotFound(String),
    InitFailed,
}

#[derive(Debug)]
pub enum DocumentError {
    InvalidFormat,
    IncorrectPassword,
    SecurityRestriction,
    IoError(String),
    TruncationError(String),
}

#[derive(Debug)]
pub enum RenderError {
    BitmapCreationFailed,
    InvalidDimensions { width: u32, height: u32 },
    BufferCopyFailed,
    ConversionFailed,
}

#[derive(Debug)]
pub enum PageError {
    IndexOutOfBounds { index: usize, count: usize },
    LoadFailed,
}

#[derive(Debug)]
pub enum HandleError {
    CrossDocument,
    Stale,
}

#[derive(Debug)]
pub enum TextError {
    LoadFailed,
    CharCountFailed,
}

#[derive(Debug)]
pub enum SaveError {
    WriteFailed,
    IoError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Library(e) => write!(f, "library error: {e}"),
            Error::Document(e) => write!(f, "document error: {e}"),
            Error::Page(e) => write!(f, "page error: {e}"),
            Error::Handle(e) => write!(f, "handle error: {e}"),
            Error::Render(e) => write!(f, "render error: {e}"),
            Error::Text(e) => write!(f, "text error: {e}"),
            Error::Save(e) => write!(f, "save error: {e}"),
        }
    }
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::LoadFailed(s) => write!(f, "load failed: {s}"),
            LibraryError::SymbolNotFound(s) => write!(f, "symbol not found: {s}"),
            LibraryError::InitFailed => write!(f, "initialization failed"),
        }
    }
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentError::InvalidFormat => write!(f, "invalid PDF format"),
            DocumentError::IncorrectPassword => write!(f, "incorrect password"),
            DocumentError::SecurityRestriction => write!(f, "unsupported security restriction"),
            DocumentError::IoError(s) => write!(f, "I/O error: {s}"),
            DocumentError::TruncationError(s) => write!(f, "truncation error: {s}"),
        }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::BitmapCreationFailed => write!(f, "bitmap creation failed"),
            RenderError::InvalidDimensions { width, height } => {
                write!(f, "invalid dimensions: {width}x{height}")
            }
            RenderError::BufferCopyFailed => write!(f, "buffer copy failed"),
            RenderError::ConversionFailed => write!(f, "coordinate conversion failed"),
        }
    }
}

impl fmt::Display for PageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PageError::IndexOutOfBounds { index, count } => {
                write!(f, "page index {index} out of bounds (count: {count})")
            }
            PageError::LoadFailed => write!(f, "page load failed"),
        }
    }
}

impl fmt::Display for HandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandleError::CrossDocument => write!(f, "page ref belongs to a different document"),
            HandleError::Stale => write!(f, "page ref is no longer valid"),
        }
    }
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextError::LoadFailed => write!(f, "text page load failed"),
            TextError::CharCountFailed => write!(f, "character count failed"),
        }
    }
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::WriteFailed => write!(f, "save write callback failed"),
            SaveError::IoError(s) => write!(f, "I/O error: {s}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}
impl std::error::Error for LibraryError {}
impl std::error::Error for DocumentError {}
impl std::error::Error for PageError {}
impl std::error::Error for HandleError {}
impl std::error::Error for RenderError {}
impl std::error::Error for TextError {}
impl std::error::Error for SaveError {}

impl From<LibraryError> for Error {
    fn from(e: LibraryError) -> Self {
        Error::Library(e)
    }
}

impl From<DocumentError> for Error {
    fn from(e: DocumentError) -> Self {
        Error::Document(e)
    }
}

impl From<PageError> for Error {
    fn from(e: PageError) -> Self {
        Error::Page(e)
    }
}

impl From<HandleError> for Error {
    fn from(e: HandleError) -> Self {
        Error::Handle(e)
    }
}

impl From<RenderError> for Error {
    fn from(e: RenderError) -> Self {
        Error::Render(e)
    }
}

impl From<TextError> for Error {
    fn from(e: TextError) -> Self {
        Error::Text(e)
    }
}

impl From<SaveError> for Error {
    fn from(e: SaveError) -> Self {
        Error::Save(e)
    }
}

impl From<SysError> for SaveError {
    fn from(e: SysError) -> Self {
        match e {
            SysError::Unknown => SaveError::WriteFailed,
            other => SaveError::IoError(other.to_string()),
        }
    }
}

impl From<SysError> for DocumentError {
    fn from(e: SysError) -> Self {
        match e {
            SysError::InvalidFormat => DocumentError::InvalidFormat,
            SysError::IncorrectPassword | SysError::NullInterior(_) => {
                DocumentError::IncorrectPassword
            }
            SysError::UnsupportedSecurity => DocumentError::SecurityRestriction,
            _ => DocumentError::InvalidFormat,
        }
    }
}

impl From<SysError> for Error {
    fn from(e: SysError) -> Self {
        match e {
            SysError::LoadFailed(s) => Error::Library(LibraryError::LoadFailed(s)),
            other => Error::Document(DocumentError::from(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_from_library_error() {
        let e: Error = LibraryError::LoadFailed("test".into()).into();
        assert!(matches!(e, Error::Library(LibraryError::LoadFailed(_))));
    }

    #[test]
    fn error_from_document_error() {
        let e: Error = DocumentError::InvalidFormat.into();
        assert!(matches!(e, Error::Document(DocumentError::InvalidFormat)));
    }

    #[test]
    fn error_from_page_error() {
        let e: Error = PageError::IndexOutOfBounds { index: 0, count: 1 }.into();
        assert!(matches!(e, Error::Page(PageError::IndexOutOfBounds { .. })));
    }

    #[test]
    fn page_error_index_out_of_bounds_uses_usize() {
        let err = PageError::IndexOutOfBounds { index: 5, count: 3 };
        if let PageError::IndexOutOfBounds { index, count } = err {
            let _: usize = index;
            let _: usize = count;
        }
    }

    #[test]
    fn error_from_handle_error() {
        let e: Error = HandleError::CrossDocument.into();
        assert!(matches!(e, Error::Handle(HandleError::CrossDocument)));
    }

    #[test]
    fn error_from_render_error() {
        let e: Error = RenderError::BitmapCreationFailed.into();
        assert!(matches!(
            e,
            Error::Render(RenderError::BitmapCreationFailed)
        ));
    }

    #[test]
    fn render_error_display() {
        assert!(!RenderError::BitmapCreationFailed.to_string().is_empty());
        assert!(
            !RenderError::InvalidDimensions {
                width: 0,
                height: 0
            }
            .to_string()
            .is_empty()
        );
        assert!(!RenderError::BufferCopyFailed.to_string().is_empty());
        assert!(!RenderError::ConversionFailed.to_string().is_empty());
    }

    #[test]
    fn render_error_implements_std_error() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<RenderError>();
    }

    #[test]
    fn document_error_io_error() {
        let e = DocumentError::IoError("file not found".into());
        assert!(e.to_string().contains("file not found"));
    }

    #[test]
    fn error_display_all_variants() {
        let cases: Vec<Error> = vec![
            LibraryError::LoadFailed("test".into()).into(),
            LibraryError::SymbolNotFound("sym".into()).into(),
            LibraryError::InitFailed.into(),
            DocumentError::InvalidFormat.into(),
            DocumentError::IncorrectPassword.into(),
            DocumentError::SecurityRestriction.into(),
            DocumentError::IoError("test".into()).into(),
            DocumentError::TruncationError("test".into()).into(),
            PageError::IndexOutOfBounds { index: 5, count: 3 }.into(),
            PageError::LoadFailed.into(),
            HandleError::CrossDocument.into(),
            HandleError::Stale.into(),
            RenderError::BitmapCreationFailed.into(),
            RenderError::InvalidDimensions {
                width: 0,
                height: 0,
            }
            .into(),
            RenderError::BufferCopyFailed.into(),
            RenderError::ConversionFailed.into(),
            TextError::LoadFailed.into(),
            TextError::CharCountFailed.into(),
            SaveError::WriteFailed.into(),
            SaveError::IoError("test".into()).into(),
        ];
        for e in cases {
            assert!(!e.to_string().is_empty());
        }
    }

    #[test]
    fn error_from_text_error() {
        let e: Error = TextError::LoadFailed.into();
        assert!(matches!(e, Error::Text(TextError::LoadFailed)));
    }

    #[test]
    fn text_error_char_count_failed_display() {
        let e = TextError::CharCountFailed;
        assert!(!e.to_string().is_empty());
        let e: Error = e.into();
        assert!(matches!(e, Error::Text(TextError::CharCountFailed)));
    }

    #[test]
    fn error_implements_std_error() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<Error>();
        assert_error::<LibraryError>();
        assert_error::<DocumentError>();
        assert_error::<PageError>();
        assert_error::<HandleError>();
    }

    #[test]
    fn save_error_display() {
        let e1 = SaveError::WriteFailed;
        let e2 = SaveError::IoError("disk full".into());
        assert!(e1.to_string().contains("write"));
        assert!(e2.to_string().contains("disk full"));
    }

    #[test]
    fn error_from_save_error() {
        let e: Error = SaveError::WriteFailed.into();
        assert!(matches!(e, Error::Save(SaveError::WriteFailed)));
    }

    #[test]
    fn error_save_display() {
        let e = Error::Save(SaveError::IoError("test".into()));
        let s = e.to_string();
        assert!(s.contains("save error"));
        assert!(s.contains("test"));
    }

    #[test]
    fn save_error_implements_std_error() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<SaveError>();
    }

    #[test]
    fn save_error_from_sys_error() {
        assert!(matches!(
            SaveError::from(SysError::Unknown),
            SaveError::WriteFailed
        ));
    }

    #[test]
    fn truncation_error_display() {
        let e = DocumentError::TruncationError("too many pages".into());
        let s = e.to_string();
        assert!(
            s.contains("truncation"),
            "should contain 'truncation', got: {s}"
        );
        assert!(
            s.contains("too many pages"),
            "should contain message, got: {s}"
        );
        let e2: Error = DocumentError::TruncationError("x".into()).into();
        assert!(matches!(
            e2,
            Error::Document(DocumentError::TruncationError(_))
        ));
    }

    #[test]
    fn save_error_from_sys_error_preserves_detail() {
        let e = SaveError::from(SysError::FileNotFound);
        assert!(
            matches!(e, SaveError::IoError(ref s) if s.contains("file not found")),
            "FileNotFound should map to IoError with detail, got: {e:?}"
        );

        let e2 = SaveError::from(SysError::InvalidFormat);
        assert!(
            matches!(e2, SaveError::IoError(ref s) if s.contains("invalid")),
            "InvalidFormat should map to IoError with detail, got: {e2:?}"
        );
    }

    #[test]
    fn save_error_from_sys_error_all_variants() {
        // Unknown -> WriteFailed (generic write failure)
        assert!(matches!(
            SaveError::from(SysError::Unknown),
            SaveError::WriteFailed
        ));

        // All other variants -> IoError with descriptive message
        let cases = vec![
            SysError::FileNotFound,
            SysError::InvalidFormat,
            SysError::IncorrectPassword,
            SysError::UnsupportedSecurity,
            SysError::PageNotFound,
            SysError::NullInterior("test".into()),
            SysError::LoadFailed("lib".into()),
        ];
        for sys_err in cases {
            let display = sys_err.to_string();
            let save_err = SaveError::from(sys_err);
            match save_err {
                SaveError::IoError(ref s) => assert!(
                    s.contains(&display) || display.contains(s),
                    "IoError should carry source detail, got: {s}"
                ),
                other => panic!("expected IoError, got {other:?}"),
            }
        }
    }

    #[test]
    fn document_error_from_sys_error() {
        assert!(matches!(
            DocumentError::from(SysError::InvalidFormat),
            DocumentError::InvalidFormat
        ));
        assert!(matches!(
            DocumentError::from(SysError::IncorrectPassword),
            DocumentError::IncorrectPassword
        ));
        assert!(matches!(
            DocumentError::from(SysError::UnsupportedSecurity),
            DocumentError::SecurityRestriction
        ));
    }
}
