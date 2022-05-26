use crate::payload::{PAYLOAD_MAX_LENGTH, PAYLOAD_MIN_LENGTH};

use thiserror::Error;

/// A result of a function that may return a `Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Contains all of the Errors that can be returned by this crate
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Payload is less than the minimum length
    #[error("Invalid payload size: `{0}` (expected >= {})", PAYLOAD_MIN_LENGTH)]
    PayloadTooShort(usize),
    /// Payload is greater than the maximum length
    #[error("Invalid payload size: `{0}` (expected < {})", PAYLOAD_MAX_LENGTH)]
    PayloadTooLong(usize),
    /// We expected to get a timeout after smashing the stack but we did not
    #[error("Expected timeout error after smashing the stack")]
    RcmExpectedError,
    /// We cannot find a switch in RCM mode connected
    #[error("Nintendo Switch in RCM mode not found")]
    SwitchNotFound,
    /// Unable to claim the Switches interface
    #[error("Unable to claim interface: `{0}`")]
    UsbBadInterface(u8),
    /// A linux environment error such as not having the correct usb driver support
    #[error("Linux environment error")]
    LinuxEnv,
    /// See <https://github.com/budde25/switcheroo#linux-permission-denied-error>
    #[error("Access denied (insufficient permissions)")]
    AccessDenied,
    /// The Usb device has timed out
    #[error("Usb has timed out, if returned on the controlled memcopy it should be considered a success")]
    Timeout,
    /// This is a catchall error for various other things that can go wrong with libusb, contains rusb's Error type
    #[error(transparent)]
    Usb(rusb::Error),
}

impl From<rusb::Error> for Error {
    fn from(err: rusb::Error) -> Self {
        match err {
            rusb::Error::Access => Self::AccessDenied,
            rusb::Error::Timeout => Self::Timeout,
            _ => Self::Usb(err),
        }
    }
}
