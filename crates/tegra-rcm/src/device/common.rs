use super::SwitchDeviceRaw;
use super::{Device, DeviceRaw};

use rusb::{DeviceHandle, GlobalContext};
use std::thread;
use std::time::Duration;

use crate::Result;

/// A connected and init switch device connection
#[derive(Debug)]
pub struct SwitchDevice {
    device: DeviceHandle<GlobalContext>,
    claimed: bool,
}

impl Device for SwitchDevice {
    /// Init the device
    fn init(&mut self) -> Result<()> {
        if !self.claimed {
            self.device.claim_interface(0)?;
            self.claimed = true;
        }
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
        Self {
            device,
            claimed: false,
        }
    }

    pub fn device(&self) -> &DeviceHandle<GlobalContext> {
        &self.device
    }
}

impl SwitchDeviceRaw {
    fn open_device_with_vid_pid(vid: u16, pid: u16) -> Result<DeviceHandle<GlobalContext>> {
        for device in rusb::devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();

            if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
                let dev = device.open()?;
                return Ok(dev);
            }
        }
        Err(crate::Error::SwitchNotFound)
    }
}

impl DeviceRaw for SwitchDeviceRaw {
    /// Tries to connect to the device and open and interface
    fn find_device(self, wait: bool) -> Result<SwitchDevice> {
        let mut device = Self::open_device_with_vid_pid(self.vid, self.pid);
        while wait && device.is_err() {
            thread::sleep(Duration::from_secs(1));
            device = Self::open_device_with_vid_pid(self.vid, self.pid);
        }

        if let Err(err) = device {
            if err == crate::Error::SwitchNotFound {
                return Err(crate::Error::SwitchNotFound);
            }
        }

        let device = device?;

        Ok(SwitchDevice::with_device_handle(device))
    }
}
