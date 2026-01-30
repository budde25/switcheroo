use crate::payload::{PAYLOAD_MAX_LENGTH, PAYLOAD_MIN_LENGTH};
use std::path::PathBuf;
use thiserror::Error;

/// An error while trying to create a payload
#[derive(Debug, PartialEq, Eq, Error, Clone)]
#[non_exhaustive]
pub enum PayloadError {
    /// Reading payload failed, std::io::Error
    #[error("Payload failed to read from file: {1}, io error: {0}")]
    Io(std::io::ErrorKind, PathBuf),

    /// Payload is less than the minimum length
    #[error("Payload invalid size: `{0}` (expected >= {min})", min = PAYLOAD_MIN_LENGTH)]
    PayloadTooShort(usize),

    /// Payload is greater than the maximum length
    #[error("Payload invalid size: `{0}` (expected < {max})", max =  PAYLOAD_MAX_LENGTH)]
    PayloadTooLong(usize),
}
