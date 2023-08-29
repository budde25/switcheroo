use anyhow::Result;

use camino::{Utf8Path, Utf8PathBuf};
use tegra_rcm::Payload;

use crate::favorites::{Favorite, Favorites};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadData {
    payload: Payload,
    path: Utf8PathBuf,
    file_name: String,
}

impl PayloadData {
    /// Makes a payload from a given file path
    pub fn new(path: &Utf8Path) -> Result<Self> {
        let payload = Payload::read(&path)?;

        let payload_data = PayloadData {
            path: path.to_owned(),
            payload,
            file_name: path.file_name().unwrap().to_string(),
        };
        Ok(payload_data)
    }

    /// Get the file name
    pub fn file_name(&self) -> &str {
        &self.file_name
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
    pub fn read_payload_data(&self) -> Result<PayloadData> {
        let payload = self.read()?;
        Ok(PayloadData {
            payload,
            path: Favorites::directory().to_owned(),
            file_name: self.name().to_string(),
        })
    }
}
