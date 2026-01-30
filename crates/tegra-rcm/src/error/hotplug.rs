use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum HotplugError {
    #[error("the hotplug API is not supported on this platform")]
    NotSupported,

    #[error("a file watcher error occurred")]
    Watcher,
}
