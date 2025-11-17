//! Tegra RCM library
//!
//! A library to help exploit the bootROM exploit for the Tegra X1's RCM mode
//! Current support OS's are Linux, MacOS, and Windows
//!
//! # Example
//!
//! ```no_run
//! use tegra_rcm::{Payload, Switch};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let payload = Payload::read("payload.bin")?;
//!     let switch = Switch::find()?;
//!     switch.execute(&payload)?;
//!     println!("Done!");
//!     Ok(())
//! }
//! ```

mod env;
mod error;
mod exploit;
mod hotplug;
mod payload;
mod switch;
mod usb;

use error::Result;
use usb::SwitchHandle;

pub use env::check_env;
pub use error::SwitchError;
pub use hotplug::{create_hotplug, Actions};
pub use payload::{Payload, PayloadError};
pub use switch::Switch;
