use libusbk::{DeviceHandle, DeviceList};

use super::{Device, SWITCH_PID, SWITCH_VID};
use crate::Result;

/// A connected and init switch device connection
#[derive(Debug)]
pub struct SwitchDevice {
    device: DeviceHandle,
}

impl Device for SwitchDevice {
    /// Tries to connect to the device and open and interface
    fn find_device() -> Result<Option<Self>> {
        let devices = DeviceList::new()?;
        let device = devices.find_with_vid_and_pid(SWITCH_PID as i32, SWITCH_VID as i32);
        if let Ok(device) = device {
            let handle = device.open()?;
            return Ok(Some(Self::with_device_handle(handle)));
        }

        // We did not find the device
        Ok(None)
    }

    /// Init the device
    fn init(&mut self) -> Result<()> {
        // stub
        Ok(())
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
        Self { device }
    }

    pub fn device(&self) -> &DeviceHandle {
        &self.device
    }
}
