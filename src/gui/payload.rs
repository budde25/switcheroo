use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::Result;

use tegra_rcm::Payload;

use crate::favorites::{Favorite, Favorites};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadData {
    payload: Payload,
    path: PathBuf,
    file_name: String,
}

impl PayloadData {
    /// Makes a payload from a given file path
    pub fn new(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path)?;

        let payload_data = PayloadData {
            path: path.to_owned(),
            payload: Payload::new(&bytes)?,
            file_name: path
                .file_name()
                .unwrap_or_else(|| OsStr::new("Unknown File"))
                .to_string_lossy()
                .to_string(),
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
    pub fn path(&self) -> &PathBuf {
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
