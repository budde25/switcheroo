use log::debug;
use std::thread;
use tegra_rcm::{create_hotplug, Actions, Switch};

use crate::switch::SwitchDevice;

struct HotplugHandler<'callback> {
    switch: SwitchDevice,
    callback: Box<dyn FnMut() + 'callback>,
}

impl<'callback> Actions for HotplugHandler<'callback> {
    fn arrives(&mut self, rcm: Switch) {
        let lock = self.switch.0.lock();
        debug!("Switch has been plugged in");

        if let Ok(mut inner) = lock {
            *inner = Some(rcm);
            (self.callback)();
            //self.ctx.request_repaint();
        }
    }

    fn leaves(&mut self) {
        let lock = self.switch.0.lock();
        debug!("Switch has been unplugged");

        if let Ok(mut inner) = lock {
            *inner = None;
            (self.callback)();
            //self.ctx.request_repaint();
        }
    }
}

/// Spawn a separate thread too
pub fn spawn_thread(switch: SwitchDevice, callback: Box<dyn FnMut() + Send>) {
    thread::spawn(move || create_hotplug(Box::new(HotplugHandler { switch, callback })));
}
