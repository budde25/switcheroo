use super::{Device, SWITCH_PID, SWITCH_VID};

use rusb::{Context, DeviceHandle, UsbContext};
use std::time::Duration;

use crate::Result;

/// A connected and init switch device connection
#[derive(Debug)]
pub struct SwitchDevice {
    handle: DeviceHandle<Context>,
}

impl Device for SwitchDevice {
    /// Tries to connect to the device and open and interface
    fn find_device() -> Result<Option<Self>> {
        let context = Context::new()?;
        for device in context.devices()?.iter() {
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
        self.handle.claim_interface(0)?;
        Ok(())
    }

    /// Read from the device into the buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let amount = self.handle.read_bulk(0x81, buf, Duration::from_secs(1))?;
        Ok(amount)
    }

    /// Write to the device from the buffer
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let amount = self.handle.write_bulk(0x01, buf, Duration::from_secs(1))?;
        Ok(amount)
    }
}

impl SwitchDevice {
    pub fn with_device_handle(device: DeviceHandle<Context>) -> Self {
        Self { handle: device }
    }

    pub fn device(&self) -> &DeviceHandle<Context> {
        &self.handle
    }
}
