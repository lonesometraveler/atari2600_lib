// Graphics Scan Counter
#[derive(Default)]
pub(crate) struct ScanCounter {
    pub(crate) bit_idx: Option<isize>,
    pub(crate) bit_copies_written: usize,
    pub(crate) bit_value: Option<bool>,
}
