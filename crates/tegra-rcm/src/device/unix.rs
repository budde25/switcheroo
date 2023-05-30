use super::{Device, SWITCH_PID, SWITCH_VID};

use rusb::{DeviceHandle, GlobalContext};
use std::time::Duration;

use crate::Result;

/// A connected and init switch device connection
#[derive(Debug)]
pub struct SwitchDevice {
    device: DeviceHandle<GlobalContext>,
}

impl Device for SwitchDevice {
    /// Tries to connect to the device and open and interface
    fn find_device() -> Result<Option<Self>> {
        for device in rusb::devices()?.iter() {
            let device_desc = device.device_descriptor()?;

            if device_desc.vendor_id() == SWITCH_VID && device_desc.product_id() == SWITCH_PID {
                let dev = device.open()?;
                return Ok(Some(Self::with_device_handle(dev)));
            }
        }
        // We did not find the device
        Ok(None)
    }

    /// Init the device
    fn init(&mut self) -> Result<()> {
        self.device.claim_interface(0)?;
        Ok(())
    }

    /// Read from the device into the buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let amount = self.device.read_bulk(0x81, buf, Duration::from_secs(1))?;
        Ok(amount)
    }

    /// Write to the device from the buffer
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let amount = self.device.write_bulk(0x01, buf, Duration::from_secs(1))?;
        Ok(amount)
    }
}

impl SwitchDevice {
    pub fn with_device_handle(device: DeviceHandle<GlobalContext>) -> Self {
        Self { device }
    }

    pub fn device(&self) -> &DeviceHandle<GlobalContext> {
        &self.device
    }
}
