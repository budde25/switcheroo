use log::{debug, trace};

use crate::Result;

use crate::exploit::Exploit;
use crate::usb::{Device, DeviceHandle, SwitchDevice, SwitchHandle};
use crate::Payload;

/// A Switch device in RCM (Recovery Mode)
///
/// This represents a Nintendo Switch device connected in RCM mode,
/// ready to receive and execute payloads via the Fusée Gelée exploit.
#[derive(Debug, Clone)]
pub struct Switch {
    switch: SwitchDevice,
}

impl Switch {
    /// Create a new Switch from an existing SwitchDevice
    ///
    /// This is used internally by the hotplug system.
    /// Should not have its interface claimed yet.
    pub(crate) fn new(device: SwitchDevice) -> Self {
        Self { switch: device }
    }

    /// Find and connect to a Switch device in RCM mode
    ///
    /// This will search for a connected Switch device in RCM mode.
    /// Returns an error if no device is found.
    pub fn find() -> Result<Self> {
        let device = SwitchDevice::find_device()?;
        Ok(Self { switch: device })
    }

    /// Execute a payload on the Switch
    ///
    /// This will send the payload to the Switch and trigger the exploit.
    /// The payload is executed and the Switch will boot into the payload.
    ///
    /// This method consumes the Switch as the device connection is closed
    /// after execution.
    pub fn execute(mut self, payload: &Payload) -> Result<()> {
        let handle = self.init_handle()?;
        handle.execute_payload(payload)
    }

    /// Initialize a handle for internal operations
    fn init_handle(&mut self) -> Result<Handle> {
        let handle = self.switch.init()?;
        Ok(Handle {
            handle,
            current_buffer: BufferState::Low,
            total_written: 0,
        })
    }
}

/// An RCM connection handle
#[derive(Debug)]
struct Handle {
    handle: SwitchHandle,
    current_buffer: BufferState,
    total_written: usize,
}

impl Handle {
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

/// The current state of the Buffer
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BufferState {
    #[default]
    Low,
    High,
}

impl BufferState {
    /// Toggle the buffer
    pub(crate) fn toggle(&mut self) {
        match self {
            BufferState::High => *self = BufferState::Low,
            BufferState::Low => *self = BufferState::High,
        }
    }

    /// Gets the address of the buffer
    pub(crate) fn address(self) -> usize {
        const COPY_BUFFER_ADDRESSES_LOW: usize = 0x4000_5000;
        const COPY_BUFFER_ADDRESSES_HIGH: usize = 0x4000_9000;
        match self {
            BufferState::Low => COPY_BUFFER_ADDRESSES_LOW,
            BufferState::High => COPY_BUFFER_ADDRESSES_HIGH,
        }
    }
}
