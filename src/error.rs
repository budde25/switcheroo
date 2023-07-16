use std::{path::PathBuf, string::FromUtf8Error};

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
    Uf8(FromUtf8Error),
}
