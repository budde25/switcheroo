use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("An IO error occurred to path: {path}")]
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

pub(crate) trait AddPath {
    fn with_path<P: AsRef<Path>>(self, path: P) -> Error;
}

impl AddPath for std::io::Error {
    fn with_path<P: AsRef<Path>>(self, path: P) -> Error {
        Error::Io {
            source: self,
            path: path.as_ref().to_path_buf(),
        }
    }
}
