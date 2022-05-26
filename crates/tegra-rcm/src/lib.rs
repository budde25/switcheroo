//! A library to help exploit the bootROM exploit for the Tegra X1's RCM mode

#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts
)]

mod device;
mod error;
#[cfg(not(target_os = "windows"))]
mod hotplug;
mod payload;
mod rcm;
mod vulnerability;

use device::SwitchDevice;

pub use error::{Error, Result};
#[cfg(not(target_os = "windows"))]
pub use hotplug::{create_hotplug, Actions};
pub use payload::Payload;
pub use rcm::Rcm;
