use thiserror::Error;

use super::{ExploitError, HotplugError, UsbError};

/// An error interacting with the Switch in RCM mode
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SwitchError {
    /// USB-related error
    #[error(transparent)]
    Usb(#[from] UsbError),

    /// Exploit-related error
    #[error(transparent)]
    Exploit(#[from] ExploitError),

    /// Hotplug-related error
    #[error(transparent)]
    Hotplug(#[from] HotplugError),

    /// Running on an unsupported platform
    #[error("platform not supported")]
    PlatformNotSupported,
}
