use std::thread;
use std::time::Duration;

use crate::Error;
use rusb::{DeviceHandle, GlobalContext};

/// A switch device that has not been init yet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchDeviceRaw {
    vid: u16,
    pid: u16,
}

impl SwitchDeviceRaw {
    /// Creates a new uninit device with a custom vid and pid
    pub fn new(vid: u16, pid: u16) -> Self {
        Self { vid, pid }
    }

    fn open_device_with_vid_pid(vid: u16, pid: u16) -> rusb::Result<DeviceHandle<GlobalContext>> {
        for device in rusb::devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();

            if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
                return device.open();
            }
        }
        Err(rusb::Error::NotFound)
    }

    /// Tries to connect to the device and open and interface
    pub fn find_device(self, wait: bool) -> Result<SwitchDevice, Error> {
        let mut device = Self::open_device_with_vid_pid(self.vid, self.pid);
        while wait && device.is_err() {
            thread::sleep(Duration::from_secs(1));
            device = Self::open_device_with_vid_pid(self.vid, self.pid);
        }

        if let Err(err) = device {
            if err == rusb::Error::NotFound {
                return Err(Error::SwitchNotFound);
            }
        }

        let device = device?;

        Ok(SwitchDevice {
            device,
            claimed: false,
        })
    }
}

impl Default for SwitchDeviceRaw {
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
    claimed: bool,
}

impl SwitchDevice {
    /// Init the device
    pub fn init(&mut self) -> Result<(), Error> {
        if !self.claimed {
            self.device.claim_interface(0)?;
            self.claimed = true;
        }
        Ok(())
    }

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
