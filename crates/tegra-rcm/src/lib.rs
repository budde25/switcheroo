//! A library to help exploit the bootROM exploit for the Tegra X1's RCM mode

#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts
)]

mod device;
mod error;
mod payload;
mod rcm;
mod vulnerability;

use device::SwitchDevice;

pub use error::Error;
pub use payload::Payload;
pub use rcm::Rcm;
