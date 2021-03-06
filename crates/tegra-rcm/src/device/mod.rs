use crate::Result;

#[cfg(not(target_os = "windows"))]
mod common;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
pub use common::SwitchDevice;
#[cfg(target_os = "windows")]
pub use windows::SwitchDevice;

pub const SWITCH_VID: u16 = 0x0955;
pub const SWITCH_PID: u16 = 0x7321;

pub(crate) trait Device {
    fn init(&mut self) -> Result<()>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn validate(&self) -> Result<()>;
}

pub(crate) trait DeviceRaw {
    fn find_device(self, wait: bool) -> Result<SwitchDevice>;
}

/// A switch device that has not been init yet
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SwitchDeviceRaw {
    vid: u16,
    pid: u16,
}

impl SwitchDeviceRaw {
    /// Creates a new uninit device with a custom vid and pid
    pub fn new(vid: u16, pid: u16) -> Self {
        Self { vid, pid }
    }
}

impl Default for SwitchDeviceRaw {
    fn default() -> Self {
        // Default Nintendo Switch RCM VID and PIC
        Self::new(SWITCH_VID, SWITCH_PID)
    }
}
