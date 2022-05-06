use crate::payload::{PAYLOAD_MAX_LENGTH, PAYLOAD_MIN_LENGTH};

use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    #[error("Invalid payload size: `{0}` (expected >= {})", PAYLOAD_MIN_LENGTH)]
    PayloadTooShort(usize),
    #[error("Invalid payload size: `{0}` (expected < {})", PAYLOAD_MAX_LENGTH)]
    PayloadTooLong(usize),
    /// We expected to get a timeout after smashing the stack but we did not
    #[error("Expected timeout error after smashing the stack")]
    RcmExpectedError,
    #[error("Nintento Switch in RCM mode not found")]
    SwitchNotFound,
    #[error("Unable to claim interface: `{0}`")]
    UsbBadInterface(u8),
    #[error("Linux environment error")]
    LinuxEnv,
    /// See <https://github.com/budde25/switcheroo#linux-permission-denied-error>
    #[error("Access denied (insufficient permissions)")]
    AccessDenied,
    #[error(transparent)]
    Usb(rusb::Error),
}

impl From<rusb::Error> for Error {
    fn from(err: rusb::Error) -> Self {
        match err {
            rusb::Error::Access => Self::AccessDenied,
            _ => Self::Usb(err),
        }
    }
}
