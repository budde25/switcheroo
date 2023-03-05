use eframe::egui::Context;
use log::debug;
use std::thread;
use tegra_rcm::{create_hotplug, Actions, Switch};

use super::SwitchDevice;

struct HotplugHandler {
    switch: SwitchDevice,
    ctx: Context,
}

impl Actions for HotplugHandler {
    fn arrives(&mut self, rcm: Switch) {
        let lock = self.switch.0.lock();
        debug!("Switch has been plugged in");

        if let Ok(mut inner) = lock {
            *inner = Some(rcm);
            self.ctx.request_repaint();
        }
    }

    fn leaves(&mut self) {
        let lock = self.switch.0.lock();
        debug!("Switch has been unplugged");

        if let Ok(mut inner) = lock {
            *inner = None;
            self.ctx.request_repaint();
        }
    }
}

/// Spawn a separate thread too
pub fn spawn_thread(switch: SwitchDevice, ctx: Context) {
    thread::spawn(move || create_hotplug(Box::new(HotplugHandler { switch, ctx })));
}
