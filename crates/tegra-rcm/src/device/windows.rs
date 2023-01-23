use libusbk::{DeviceHandle, DeviceList};

use super::DeviceRaw;
use super::{Device, SwitchDeviceRaw};
use crate::vulnerability::Vulnerability;
use crate::{Result, SwitchError};

/// A connected and init switch device connection
#[derive(Debug)]
pub struct SwitchDevice {
    device: DeviceHandle,
    claimed: bool,
}

impl Device for SwitchDevice {
    /// Init the device
    fn init(&mut self) -> Result<()> {
        if !self.claimed {
            self.claimed = true;
        }
        self.validate_environment()
    }

    /// Read from the device into the buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let amount = self.device.read_pipe(0x81, buf)?;
        Ok(amount as usize)
    }

    /// Write to the device from the buffer
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let amount = self.device.write_pipe(0x01, buf)?;
        Ok(amount as usize)
    }
}

impl SwitchDevice {
    pub fn with_device_handle(device: DeviceHandle) -> Self {
        Self {
            device,
            claimed: false,
        }
    }

    pub fn device(&self) -> &DeviceHandle {
        &self.device
    }
}

impl SwitchDeviceRaw {
    fn open_device_with_vid_pid(vid: u16, pid: u16) -> Result<DeviceHandle> {
        let devices = DeviceList::new()?;
        let device = devices.find_with_vid_and_pid(vid as i32, pid as i32);
        if let Ok(dev) = device {
            let handle = dev.open()?;
            return Ok(handle);
        }

        Err(SwitchError::SwitchNotFound)
    }
}

impl DeviceRaw for SwitchDeviceRaw {
    /// Tries to connect to the device and open and interface
    fn find_device(self) -> Option<Result<SwitchDevice>> {
        let device = Self::open_device_with_vid_pid(self.vid, self.pid);

        let device = match device {
            Ok(dev) => dev,
            Err(e) => {
                if e == SwitchError::SwitchNotFound {
                    return None;
                }
                return Some(Err(e));
            }
        };

        Some(Ok(SwitchDevice::with_device_handle(device)))
    }
}
