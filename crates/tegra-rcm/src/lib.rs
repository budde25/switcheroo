//! A library to help exploit the bootROM exploit for the Tegra X1's RCM mode
//! Current support OS's are Linux, MacOS, and Windows

#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts
)]

mod device;
mod error;
mod hotplug;
mod payload;
mod switch;
mod vulnerability;

use device::SwitchDevice;
use error::Result;

pub use error::SwitchError;
pub use hotplug::{create_hotplug, Actions};
pub use payload::{Payload, PayloadError};
pub use switch::Switch;
