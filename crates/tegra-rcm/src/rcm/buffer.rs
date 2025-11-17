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
