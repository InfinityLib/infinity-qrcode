#[derive(Debug)]
pub enum QrError {
    InvalidVersion,
    InvalidMode,
    DataTooLong,
    ReedSolomonError(String),
}