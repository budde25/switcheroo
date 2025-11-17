use super::{SwitchError, UsbError};

#[cfg(target_os = "macos")]
impl From<rusb::Error> for UsbError {
    fn from(err: rusb::Error) -> Self {
        match err {
            rusb::Error::Access => Self::AccessDenied,
            rusb::Error::NoDevice => Self::SwitchNotFound,
            rusb::Error::NotFound => Self::SwitchNotFound,
            _ => Self::Other(err.to_string()),
        }
    }
}

#[cfg(target_os = "macos")]
impl From<rusb::Error> for SwitchError {
    fn from(err: rusb::Error) -> Self {
        Self::Usb(err.into())
    }
}
