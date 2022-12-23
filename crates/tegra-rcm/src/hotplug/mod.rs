#[cfg(any(target_os = "macos", target_os = "linux"))]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub use unix::create_hotplug;

#[cfg(target_os = "windows")]
pub use windows::create_hotplug;

use crate::Switch;

/// Defines the two actions for when a device is plugged in or removed
pub trait Actions {
    /// A switch device has a arrived
    fn arrives(&mut self, rcm: Switch);
    /// A switch device has left
    fn leaves(&mut self);
}
