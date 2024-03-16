use log::{debug, trace};

use crate::Result;

use crate::buffer::BufferState;
use crate::device::{Device, DeviceHandle, SwitchDevice, SwitchHandle};
use crate::vulnerability::Vulnerability;
use crate::Payload;

/// Switch Device
#[derive(Debug, Clone)]
pub struct Switch {
    switch: SwitchDevice,
}

impl Switch {
    /// Create a new Rcm object from an existing SwitchDevice
    /// Should not have its interface claimed yet
    pub(crate) fn new(device: SwitchDevice) -> Self {
        Self { switch: device }
    }

    /// Finds and connects to a Switch device
    pub fn find() -> Result<Self> {
        let device = SwitchDevice::find_device()?;
        Ok(Self { switch: device })
    }

    /// Gets the Switch handle
    pub fn handle(&mut self) -> Result<Handle> {
        let handle = self.switch.init()?;
        Ok(Handle {
            handle,
            current_buffer: BufferState::Low,
            total_written: 0,
        })
    }
}

/// An RCM connection object
/// This is the main interface to communicate with the switch
#[derive(Debug)]
pub struct Handle {
    handle: SwitchHandle,
    current_buffer: BufferState,
    total_written: usize,
}

impl Handle {
    /// This will execute the payload on the connected device
    /// This consumes the device
    pub fn execute(mut self, payload: &Payload) -> Result<()> {
        let device_id = self.read_device_id()?;
        trace!("Device ID: {:?}", device_id);

        self.write(payload.data())?;
        self.switch_to_highbuf()?;

        // Smash the stack
        self.trigger_controlled_memcopy()
    }

    /// Writes data to the RCM protocol endpoint
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        const PACKET_SIZE: usize = 0x1000;
        const MAX_LENGTH: usize = 0x30298;

        assert!(buf.len() <= MAX_LENGTH);

        let mut remaining_buf = buf;
        let mut length_remaining = buf.len();

        let mut written = 0;

        while length_remaining != 0 {
            let data_to_transmit = length_remaining.min(PACKET_SIZE);
            length_remaining -= data_to_transmit;

            let chunk = &remaining_buf[..data_to_transmit];
            remaining_buf = &remaining_buf[data_to_transmit..];
            match self.write_buffer(chunk) {
                Ok(size) => written += size,
                Err(e) => return Err(e),
            };
        }
        // update the current amount of bytes written
        self.total_written += written;

        Ok(written)
    }

    fn switch_to_highbuf(&mut self) -> Result<()> {
        if self.current_buffer != BufferState::High {
            let buf = &[b'\0'; 0x1000];
            self.write(buf)?;
        }
        Ok(())
    }

    fn trigger_controlled_memcopy(&self) -> Result<()> {
        const STACK_END: usize = 0x40010000;
        debug!(
            "Wrote a total of {} bytes to the switch, performing the controlled memcopy",
            self.total_written
        );

        let length = STACK_END - self.current_buffer.address();
        self.handle.trigger(length)?;
        Ok(())
    }

    /// Reads the device ID
    /// Note: The is a REQUIRED step before executing
    fn read_device_id(&mut self) -> Result<[u8; 16]> {
        let mut buf = [b'\0'; 16];
        self.read(&mut buf)?;
        Ok(buf)
    }

    fn write_buffer(&mut self, buf: &[u8]) -> Result<usize> {
        self.current_buffer.toggle();
        let written = self.handle.write(buf)?;
        Ok(written)
    }

    /// Read from the device
    /// Returns bytes read
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read = self.handle.read(buf)?;
        Ok(read)
    }
}
