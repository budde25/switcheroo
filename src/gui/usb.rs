use std::thread;

use crate::Error;

use tegra_rcm::{Rcm, Actions, create_hotplug};

use egui::Context;

use super::ThreadSwitchResult;

struct HotplugHandler {
    tswitch: ThreadSwitchResult,
    ctx: Context,
}

impl Actions for HotplugHandler {
    fn arrives(&mut self, rcm: Rcm) {
        let lock = self.tswitch.lock();

        if let Ok(mut inner) = lock {
            *inner = Ok(rcm);
            self.ctx.request_repaint();
        }
    }

    fn leaves(&mut self) {
        let lock = self.tswitch.lock();

        if let Ok(mut inner) = lock {
            *inner = Err(Error::SwitchNotFound);
            self.ctx.request_repaint();
        }
    }
}

/// Spawn a separate thread too
pub fn spawn_thread(tswitch: ThreadSwitchResult, ctx: Context) {
    thread::spawn(move || create_hotplug(Box::new(HotplugHandler {tswitch, ctx})));
}
