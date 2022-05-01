use std::thread;
use std::time::Duration;

use rusb::{DeviceHandle, GlobalContext};
use thiserror::Error;

/// Errors for converting a unit device to an init one
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwitchDeviceUninitError {
    #[error("Nintento Switch in RCM mode not found")]
    NotFound,
    #[error("Unable to claim interface: `{0}`")]
    BadInterface(u8),
}

/// A switch device that has not been init yet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchDeviceUninit {
    vid: u16,
    pid: u16,
}

impl SwitchDeviceUninit {
    /// Creates a new uninit device with a custom vid and pid
    pub fn new(vid: u16, pid: u16) -> Self {
        Self { vid, pid }
    }

    /// Tries to connect to the device and open and interface
    pub fn find_device(self, wait: bool) -> Result<SwitchDevice, SwitchDeviceUninitError> {
        let mut device = rusb::open_device_with_vid_pid(self.vid, self.pid);
        while wait && device.is_none() {
            thread::sleep(Duration::from_secs(1));
            device = rusb::open_device_with_vid_pid(self.vid, self.pid);
        }

        let mut device = device.ok_or(SwitchDeviceUninitError::NotFound)?;
        if device.claim_interface(0).is_err() {
            return Err(SwitchDeviceUninitError::BadInterface(0));
        }

        Ok(SwitchDevice { device })
    }
}

impl Default for SwitchDeviceUninit {
    fn default() -> Self {
        // Default Nintendo Switch RCM VID and PIC
        let vid = 0x0955;
        let pid = 0x7321;

        Self::new(vid, pid)
    }
}

/// A connected and init switch device connection
#[derive(Debug)]
pub struct SwitchDevice {
    device: DeviceHandle<GlobalContext>,
}

impl SwitchDevice {
    /// Read from the device into the buffer
    pub fn read(&self, buf: &mut [u8]) -> rusb::Result<usize> {
        self.device.read_bulk(0x81, buf, Duration::from_secs(1))
    }

    /// Write to the device from the buffer
    pub fn write(&self, buf: &[u8]) -> rusb::Result<usize> {
        self.device.write_bulk(0x01, buf, Duration::from_secs(1))
    }

    /// Returns the device
    pub fn device(&self) -> &DeviceHandle<GlobalContext> {
        &self.device
    }
}
