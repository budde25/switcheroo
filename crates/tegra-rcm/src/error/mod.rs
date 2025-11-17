mod error;
mod exploit;
mod hotplug;
mod payload;
mod usb;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
    } else if #[cfg(target_os = "macos")] {
        mod macos;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
    } else {
        compile_error!("Unsupported OS");
    }
}

/// A result of a function that may return a `Error`.
pub(crate) type Result<T> = std::result::Result<T, SwitchError>;

pub use error::SwitchError;
pub use exploit::{ExploitError, WindowsDriver};
pub use hotplug::HotplugError;
pub use payload::PayloadError;
pub use usb::UsbError;
