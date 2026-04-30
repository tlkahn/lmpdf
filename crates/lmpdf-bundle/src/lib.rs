pub use lmpdf_sys;

#[cfg(test)]
mod tests {
    #[test]
    fn can_import_sys_types() {
        let _doc: lmpdf_sys::FPDF_DOCUMENT = std::ptr::null_mut();
    }
}
