use libusbk::{Device, DeviceHandle, DeviceList};

use super::{RCM_PID, RCM_VID};
use crate::Result;

/// A connected and init switch device connection
#[derive(Debug, Clone)]
pub struct SwitchDevice {
    pub(crate) device: Device,
}

/// A connected and init switch device connection
#[derive(Debug)]
pub struct SwitchHandle {
    pub(crate) handle: DeviceHandle,
}

impl super::Device for SwitchDevice {
    /// Tries to connect to the device and open and interface
    fn find_device() -> Result<Self> {
        let devices = DeviceList::new()?;
        let device = devices.find_with_vid_and_pid(RCM_PID as i32, RCM_VID as i32)?;
        Ok(Self { device })
    }

    /// Init the device
    fn init(&mut self) -> Result<SwitchHandle> {
        Ok(SwitchHandle {
            handle: self.device.open()?,
        })
    }
}

impl super::DeviceHandle for SwitchHandle {
    /// Read from the device into the buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let amount = self.handle.read_pipe(0x81, buf)?;
        Ok(amount as usize)
    }

    /// Write to the device from the buffer
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let amount = self.handle.write_pipe(0x01, buf)?;
        Ok(amount as usize)
    }
}
