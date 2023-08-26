use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("An IO error occured to: {path}")]
    Io {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },

    #[error(transparent)]
    Generic(#[from] anyhow::Error),

    #[error(transparent)]
    Switch(#[from] tegra_rcm::SwitchError),

    #[error(transparent)]
    Payload(#[from] tegra_rcm::PayloadError),

    #[error("Favorite not found: {0}")]
    FavoriteNotFound(String),
}

impl Error {
    pub(crate) fn io_with_path(path: impl AsRef<Path>) -> Self {
        Self::Io {
            source: std::io::Error::from_raw_os_error(2),
            path: path.as_ref().to_owned(),
        }
    }
}
