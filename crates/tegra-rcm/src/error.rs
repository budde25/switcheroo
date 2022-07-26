use std::fmt::Display;

use crate::payload::{PAYLOAD_MAX_LENGTH, PAYLOAD_MIN_LENGTH};

use thiserror::Error;

/// A result of a function that may return a `Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Contains all of the Errors that can be returned by this crate
#[derive(Error, Debug, Clone, PartialEq, Eq)]
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

    /// Running on an unsupported platform
    #[error("Platform not supported")]
    PlatformNotSupported,

    /// A Windows error that the switch RCM has the wrong driver, please install libusbK
    #[error("Wrong RCM USB driver installed (installed: `{0}` but must be libusbK)")]
    WindowsWrongDriver(WindowsDriver),

    /// This is a catchall error for various other things that can go wrong with underlying usb library.
    /// It has been converted to a string to not expose the underlying api
    #[error("Usb Error: `{0}`")]
    Usb(String),
}

/// Defines the installed driver on windows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsDriver {
    LibUsbK,
    LibUsb0,
    WinUsb,
    LibUsb0Filter,
    Count,
}

impl Display for WindowsDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibUsbK => write!(f, "libusbK"),
            Self::LibUsb0 => write!(f, "libusb0"),
            Self::WinUsb => write!(f, "winusb"),
            Self::LibUsb0Filter => write!(f, "libusb0 filter"),
            Self::Count => write!(f, "count"),
        }
    }
}

#[cfg(target_os = "windows")]
impl From<libusbk::DriverId> for WindowsDriver {
    fn from(driver: libusbk::DriverId) -> Self {
        use libusbk::DriverId;

        match driver {
            DriverId::LibUsbK => Self::LibUsbK,
            DriverId::LibUsb0 => Self::LibUsb0,
            DriverId::WinUsb => Self::WinUsb,
            DriverId::LibUsb0Filter => Self::LibUsb0Filter,
            DriverId::Count => Self::Count,
        }
    }
}

#[cfg(target_os = "windows")]
impl From<libusbk::Error> for Error {
    fn from(err: libusbk::Error) -> Self {
        Self::Usb(err.to_string())
    }
}

#[cfg(not(target_os = "windows"))]
impl From<rusb::Error> for Error {
    fn from(err: rusb::Error) -> Self {
        match err {
            rusb::Error::Access => Self::AccessDenied,
            _ => Self::Usb(err.to_string()),
        }
    }
}
