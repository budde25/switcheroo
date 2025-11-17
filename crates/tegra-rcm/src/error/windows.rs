use super::{SwitchError, UsbError};

#[cfg(target_os = "windows")]
impl From<libusbk::Error> for UsbError {
    fn from(err: libusbk::Error) -> Self {
        Self::Other(err.to_string())
    }
}

#[cfg(target_os = "windows")]
impl From<libusbk::Error> for SwitchError {
    fn from(err: libusbk::Error) -> Self {
        Self::Usb(err.into())
    }
}
