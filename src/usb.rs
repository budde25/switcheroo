use log::debug;
use std::thread;
use tegra_rcm::{create_hotplug, Actions, Switch};

use crate::switch::SwitchDevice;

struct HotplugHandler<'callback> {
    switch: SwitchDevice,
    /// Must be none blocking
    callback: Box<dyn Fn() + 'callback>,
}

impl<'callback> Actions for HotplugHandler<'callback> {
    fn arrives(&mut self, switch: Switch) {
        debug!("Switch has been plugged in");
        if let Ok(mut inner) = self.switch.0.lock() {
            *inner = Some(switch);
        };

        (self.callback)();
    }

    fn leaves(&mut self) {
        debug!("Switch has been unplugged");
        if let Ok(mut inner) = self.switch.0.lock() {
            *inner = None;
        };

        (self.callback)();
    }
}

/// Spawn a separate thread too
pub fn spawn_thread(switch: SwitchDevice, callback: Box<dyn Fn() + Send>) {
    thread::spawn(move || create_hotplug(Box::new(HotplugHandler { switch, callback })));
}
