use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use log::{debug, trace};
use thiserror::Error;

/// A constructed payload, this is transferred to the switch in RCM mode to execute bootROM exploit
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Payload {
    data: Rc<[u8]>,
}

/// The max length for the total payload
const BUILT_PAYLOAD_MAX_LENGTH: usize = 0x30298;
// TODO: find out if this is true
/// The min length of the provided payload (inclusive)
const PAYLOAD_MIN_LENGTH: usize = PADDING_SIZE_2;
/// The max length of the provided payload (exclusive)
const PAYLOAD_MAX_LENGTH: usize = 183_640;

/// hardcoded address for the start of the stack spray
const STACK_SPRAY_START: usize = 0x4001_4E40;
/// hardcoded address for the end of the stack spray
const STACK_SPRAY_END: usize = 0x4001_7000;
const PADDING_SIZE_2: usize = STACK_SPRAY_START - PAYLOAD_START_ADDR;

const PAYLOAD_START_ADDR: usize = 0x4001_0E40;
const RCM_PAYLOAD_ADDR: usize = 0x4001_0000;

const REPEAT_COUNT: usize = (STACK_SPRAY_END - STACK_SPRAY_START) / 4;

impl Payload {
    /// Construct a new payload
    /// length should be >= 16384 and < 183640
    pub fn new(payload: &[u8]) -> Result<Self, PayloadError> {
        if payload.len() < PAYLOAD_MIN_LENGTH {
            return Err(PayloadError::PayloadTooShort(payload.len()));
        }

        if payload.len() >= PAYLOAD_MAX_LENGTH {
            return Err(PayloadError::PayloadTooLong(payload.len()));
        }

        debug!(
            "Provided payload within size bounds with a size is: {} bytes",
            payload.len()
        );

        const INTERMEZZO: &[u8; 124] = include_bytes!("intermezzo/intermezzo.bin");
        trace!("Injected intermezzo.bin");

        let mut payload_builder = Vec::with_capacity(BUILT_PAYLOAD_MAX_LENGTH);
        // start with the max_len arg
        payload_builder.extend((BUILT_PAYLOAD_MAX_LENGTH as u32).to_le_bytes());
        // pad with data to get to the start of IRAM
        payload_builder
            .extend([b'\0'; 680 - (BUILT_PAYLOAD_MAX_LENGTH as u32).to_le_bytes().len()]);
        // add the intermezzo bin
        payload_builder.extend(INTERMEZZO);

        const PADDING_SIZE_1: usize = PAYLOAD_START_ADDR - (RCM_PAYLOAD_ADDR + INTERMEZZO.len());
        payload_builder.extend([b'\0'; PADDING_SIZE_1]);

        // fit a a part of the payload before the stack spray
        let split = payload.split_at(PADDING_SIZE_2);
        payload_builder.extend(split.0);
        // start stack spray
        for _ in 0..REPEAT_COUNT {
            payload_builder.extend((RCM_PAYLOAD_ADDR as u32).to_le_bytes());
        }
        payload_builder.extend(split.1);

        // finish padding to be a size of 0x1000
        let padding_size = 0x1000 - (payload_builder.len() % 0x1000);
        payload_builder.resize(payload_builder.len() + padding_size, b'\0');

        debug_assert_eq!(payload_builder.len() % 0x1000, 0);

        let data = payload_builder.into_boxed_slice();

        debug_assert!(data.len() <= BUILT_PAYLOAD_MAX_LENGTH);
        debug!(
            "A completed payload has been build with a size of: {} bytes",
            data.len()
        );

        Ok(Self {
            data: Rc::from(data),
        })
    }

    /// Read a payload from a file
    pub fn read<P: AsRef<Path>>(path: &P) -> Result<Self, PayloadError> {
        let Ok(bytes) = std::fs::read(path) else {
            return Err(PayloadError::Io(path.as_ref().into()));
        };
        Self::new(&bytes)
    }

    /// Get the data for the payload
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// An error while trying to create a payload
#[derive(Debug, PartialEq, Eq, Error, Clone)]
#[non_exhaustive]
pub enum PayloadError {
    /// Reading payload failed, std::io::Error
    #[error("Payload failed to read from file: {0}")]
    Io(PathBuf),

    /// Payload is less than the minimum length
    #[error("Payload invalid size: `{0}` (expected >= {})", PAYLOAD_MIN_LENGTH)]
    PayloadTooShort(usize),

    /// Payload is greater than the maximum length
    #[error("Payload invalid size: `{0}` (expected < {})", PAYLOAD_MAX_LENGTH)]
    PayloadTooLong(usize),
}

#[cfg(test)]
mod tests {
    use super::Payload;

    /// Tests that we generate the same bin as the reference implementation
    #[test]
    fn basic_correctness() {
        let correct = include_bytes!("test/hekate_ctcaer_5.7.2_ref_payload.bin");
        let payload = Payload::new(include_bytes!("test/hekate_ctcaer_5.7.2.bin"))
            .expect("This should give us a valid payload");

        assert_eq!(payload.data.as_ref(), correct);
    }
}
