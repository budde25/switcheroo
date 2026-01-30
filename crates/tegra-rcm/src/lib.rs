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

mod error;
mod exploit;
mod hotplug;
mod payload;
mod platform;
mod rcm;
mod switch;
mod usb;

use error::Result;
use usb::SwitchHandle;

pub use error::{ExploitError, PayloadError, SwitchError, UsbError, WindowsDriver};
pub use hotplug::{create_channel, spawn_hotplug, spawn_hotplug_callback};
pub use payload::Payload;
pub use platform::validate_environment;
pub use switch::Switch;
