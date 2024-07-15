use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Sender;

use crate::{Switch, SwitchError};

const USB_DEV_PATH: &str = "/dev/bus/usb";

pub fn watcher_hotplug(
    sender: Sender<Result<Switch, SwitchError>>,
    callback: Option<impl Fn() + Send + Sync + 'static>,
) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(Path::new(USB_DEV_PATH), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                log::debug!("change: {event:?}");

                match event.kind {
                    EventKind::Create(_) => log::info!("usb notify event create"),
                    EventKind::Remove(_) => log::info!("usb notify event remove"),
                    EventKind::Access(_) => log::info!("usb notify event access"),
                    _ => continue, // skip all non create and remove events
                }

                let device = Switch::find();
                if let Err(e) = sender.send(device) {
                    log::error!("failed to send hotplug arrive event {}", e);
                }

                if let Some(callback) = callback.as_ref() {
                    callback();
                }
            }
            Err(error) => log::error!("error: {error:?}"),
        }
    }

    log::error!("watcher loop ended");

    Ok(())
}
