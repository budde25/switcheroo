use crate::payload::{PAYLOAD_MAX_LENGTH, PAYLOAD_MIN_LENGTH};

use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid payload size: `{0}` (expected >= {})", PAYLOAD_MIN_LENGTH)]
    PayloadTooShort(usize),
    #[error("Invalid payload size: `{0}` (expected < {})", PAYLOAD_MAX_LENGTH)]
    PayloadTooLong(usize),
    #[error("Expected timeout error after smashing the stack")]
    RcmExpectedError,
    #[error("Nintento Switch in RCM mode not found")]
    SwitchNotFound,
    #[error("Unable to claim interface: `{0}`")]
    UsbBadInterface(u8),
    #[error("Linux environment error")]
    LinuxEnvError,
    #[error("Usb Error: {0}")]
    UsbError(rusb::Error),
}

impl From<rusb::Error> for Error {
    fn from(err: rusb::Error) -> Self {
        Self::UsbError(err)
    }
}
