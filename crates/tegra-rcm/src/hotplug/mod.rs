#[cfg(not(target_os = "windows"))]
mod common;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
pub use common::create_hotplug;

#[cfg(target_os = "windows")]
pub use windows::create_hotplug;

use crate::Rcm;

/// Defines the two actions for when a device is plugged in or removed
pub trait Actions {
    /// A switch device has a arrived
    fn arrives(&mut self, rcm: Rcm);
    /// A switch device has left
    fn leaves(&mut self);
}
