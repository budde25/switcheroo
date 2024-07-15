use super::{RCM_PID, RCM_VID};

use rusb::{Context, Device, DeviceHandle, UsbContext};
use std::time::Duration;

use crate::vulnerability::Vulnerability;
use crate::Result;

/// A connected and init switch device connection
#[derive(Debug, Clone)]
pub struct SwitchDevice {
    device: Device<Context>,
}

impl super::Device for SwitchDevice {
    /// Tries to connect to the device and open and interface
    fn find_device() -> Result<Self> {
        let context = Context::new()?;
        for device in context.devices()?.iter() {
            let desc = device.device_descriptor()?;

            if desc.vendor_id() == RCM_VID && desc.product_id() == RCM_PID {
                return Ok(Self { device });
            }
        }
        // We did not find the device
        Err(crate::SwitchError::SwitchNotFound)
    }

    /// Init the device
    fn init(&mut self) -> Result<SwitchHandle> {
        let handle = self.device.open()?;
        handle.claim_interface(0)?;
        let switch_handle = SwitchHandle { handle };
        switch_handle.validate_environment()?;
        Ok(switch_handle)
    }
}

impl SwitchDevice {
    pub(crate) fn new(device: Device<Context>) -> Self {
        let dev = device.device_descriptor().unwrap();
        assert_eq!(dev.product_id(), RCM_PID);
        assert_eq!(dev.vendor_id(), RCM_VID);
        Self { device }
    }
}

/// A connected and init switch device connection
#[derive(Debug)]
pub struct SwitchHandle {
    pub(crate) handle: DeviceHandle<Context>,
}

impl super::DeviceHandle for SwitchHandle {
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
