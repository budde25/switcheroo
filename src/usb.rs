use log::{debug, warn};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use tegra_rcm::{create_hotplug, Actions, Switch, SwitchError};

pub(crate) struct HotplugHandler {
    sender: Sender<Result<Switch, SwitchError>>,
}

impl Actions for HotplugHandler {
    fn arrives(&mut self, switch: Result<Switch, SwitchError>) {
        debug!("Switch has been plugged in");
        if let Err(e) = self.sender.send(switch) {
            warn!("Failed to send hotplug arrive event {}", e);
        }
    }

    fn leaves(&mut self) {
        debug!("Switch has been unplugged");
        if let Err(e) = self.sender.send(Err(SwitchError::SwitchNotFound)) {
            warn!("Failed to send hotplug leaves event {}", e);
        }
    }
}

/// Spawn a separate thread
pub fn spawn_thread() -> Receiver<Result<Switch, SwitchError>> {
    let (tx, rx) = channel();
    thread::spawn(move || create_hotplug(tx, None::<fn()>));
    rx
}
