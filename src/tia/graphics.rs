// Graphics Scan Counter
pub(crate) struct ScanCounter {
    pub(crate) bit_idx: Option<isize>,
    pub(crate) bit_copies_written: usize,
    pub(crate) bit_value: Option<bool>,
}

impl Default for ScanCounter {
    fn default() -> Self {
        Self {
            bit_idx: None,
            bit_copies_written: 0,
            bit_value: None,
        }
    }
}
