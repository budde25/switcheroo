use std::sync::mpsc::Sender;

use thiserror::Error;

use crate::error::Result;
use crate::Switch;

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "macos", target_os = "linux"))] {
        mod unix;
        pub use unix::create_hotplug;
    } else if #[cfg(target_os = "windows")] {
        #[cfg(target_os = "windows")]
        mod windows;
        pub use windows::create_hotplug;
    } else {
        compile_error!("Unsupported OS");
    }
}

/// Defines the two actions for when a device is plugged in or removed
pub trait Actions {
    /// A switch device has a arrived
    fn arrives(&mut self, switch: Result<Switch>);
    /// A switch device has left
    fn leaves(&mut self);
}

struct HotplugHandler {
    sender: Sender<Result<Switch>>,
    callback: Option<Box<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Error)]
pub enum HotplugError {
    #[error("The hotplug API is not supported on this platform")]
    NotSupported,
}
