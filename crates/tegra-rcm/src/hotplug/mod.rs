use std::sync::mpsc::{self, Receiver, Sender};

use crate::{error::HotplugError, Switch, SwitchError};

cfg_if::cfg_if! {
    if #[cfg(all(feature = "notify", target_os = "linux"))] {
        mod notify;
    } else if #[cfg(any(target_os = "macos", target_os = "linux"))] {
        mod unix;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
    } else {
        compile_error!("Unsupported OS");
    }
}

/// Create a hotplug sender, this blocks the thread
pub fn create_channel(
    tx: Sender<Result<Switch, SwitchError>>,
    callback: Option<impl Fn() + Send + Sync + 'static>,
) -> Result<(), HotplugError> {
    cfg_if::cfg_if! {
        if #[cfg(all(feature = "notify", target_os = "linux"))] {
            notify::watcher_hotplug(tx, callback)
                .map_err(|_| HotplugError::Watcher)
        } else if #[cfg(any(target_os = "linux", target_os = "macos"))] {
            unix::libusb_hotplug(tx, callback)
        } else if #[cfg(target_os = "windows")] {
            windows::windows_hotplug(tx, callback)
        } else {
            Err(HotplugError::NotSupported)
        }
    }
}

/// Creates a new thread that sends hotplug events to the reciving channel
/// and allows a callbeck
pub fn spawn_hotplug_callback(
    callback: impl Fn() + Send + Sync + 'static,
) -> Receiver<crate::Result<Switch>> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(|| create_channel(tx, Some(callback)));
    rx
}

/// Creates a new thread that sends hotplug events to the reciving channel
pub fn spawn_hotplug() -> Receiver<crate::Result<Switch>> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(|| create_channel(tx, None::<fn()>));
    rx
}

struct HotplugHandler {
    sender: Sender<crate::Result<Switch>>,
    callback: Option<Box<dyn Fn() + Send + Sync>>,
}
