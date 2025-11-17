mod error;
mod payload;

/// A result of a function that may return a `Error`.
pub(crate) type Result<T> = std::result::Result<T, SwitchError>;

pub use error::SwitchError;
pub use payload::PayloadError;
