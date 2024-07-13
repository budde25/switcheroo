use camino::Utf8Path;
use tegra_rcm::{Payload, PayloadError};

use crate::favorites::Favorite;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadData {
    payload: Payload,
    path: Box<Utf8Path>,
}

impl PayloadData {
    /// Makes a payload from a given file path
    pub fn new<P: AsRef<Utf8Path>>(path: P) -> Result<Self, PayloadError> {
        let payload = Payload::read(path.as_ref())?;

        let payload_data = PayloadData {
            path: path.as_ref().to_owned().into_boxed_path(),
            payload,
        };
        Ok(payload_data)
    }

    /// Get the file name
    pub fn file_name(&self) -> &str {
        self.path.file_name().expect("Should be a file")
    }

    pub fn file_stem(&self) -> &str {
        self.path().file_stem().expect("should be a file")
    }

    /// Get the payload
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// Get the path
    pub fn path(&self) -> &Utf8Path {
        &self.path
    }
}

impl Favorite {
    pub fn read_payload_data(&self) -> Result<PayloadData, PayloadError> {
        let payload = Payload::read(self.path().as_std_path())?;
        Ok(PayloadData {
            payload,
            path: self.path(),
        })
    }
}
