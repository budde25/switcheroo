use std::fmt::Display;

use thiserror::Error;

/// A result of a function that may return a `Error`.
pub(crate) type Result<T> = std::result::Result<T, SwitchError>;

/// An error interacting with the Switch in RCM mode
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SwitchError {
    /// Expected to get a timeout after smashing the stack but we did not
    #[error("expected timeout error after smashing the stack")]
    ExpectedError,

    /// Cannot find a switch in RCM mode connected
    #[error("switch in RCM mode not found")]
    SwitchNotFound,

    /// Usb device was not initialized
    #[error("usb device was not initialized")]
    NotInit,

    /// Unable to claim the Switches interface
    #[error("unable to claim interface: `{0}`")]
    UsbBadInterface(u8),

    /// A linux environment error such as not having the correct usb driver support
    #[error(
        "linux environment error, typically this means the system lacks a supported USB driver"
    )]
    LinuxEnv,

    /// USB permission error
    /// See <https://github.com/budde25/switcheroo#linux-permission-denied-error>
    #[error("access denied (insufficient permissions)")]
    AccessDenied,

    /// Running on an unsupported platform
    #[error("platform not supported")]
    PlatformNotSupported,

    /// A Windows error that the switch RCM has the wrong driver, please install libusbK
    /// See <https://github.com/budde25/switcheroo#windows-wrong-driver-error>
    #[error("wrong RCM USB driver installed (installed: `{0}` but must be libusbK)")]
    WindowsWrongDriver(WindowsDriver),

    /// udev rules not installed
    #[error("udev rules not found at `/etc/udev/rules.d/99-switch.rules`")]
    UdevRulesNotFound,

    /// This is a catchall error for various other things that can go wrong with underlying usb library.
    /// It has been converted to a string to not expose the underlying api
    #[error("usb error: `{0}`")]
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
impl From<libusbk::Error> for SwitchError {
    fn from(err: libusbk::Error) -> Self {
        Self::Usb(err.to_string())
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl From<rusb::Error> for SwitchError {
    fn from(err: rusb::Error) -> Self {
        match err {
            rusb::Error::Access => Self::AccessDenied,
            rusb::Error::NoDevice => Self::SwitchNotFound,
            rusb::Error::NotFound => Self::SwitchNotFound,
            _ => Self::Usb(err.to_string()),
        }
    }
}
