use thiserror::Error;

/// USB-related errors when communicating with the Switch
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UsbError {
    /// Cannot find a switch in RCM mode connected
    #[error("switch in RCM mode not found")]
    SwitchNotFound,

    /// Usb device was not initialized
    #[error("usb device was not initialized")]
    NotInit,

    /// Unable to claim the Switches interface
    #[error("unable to claim interface: `{0}`")]
    BadInterface(u8),

    /// USB permission error
    ///
    /// See <https://github.com/budde25/switcheroo#linux-permission-denied-error>
    #[error("access denied (insufficient permissions)")]
    AccessDenied,

    /// This is a catchall error for various other things that can go wrong with underlying usb library.
    ///
    /// It has been converted to a string to not expose the underlying usb implemenation api's
    #[error("usb error: `{0}`")]
    Other(String),
}
