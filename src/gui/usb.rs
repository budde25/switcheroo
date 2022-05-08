use std::thread;

use crate::Error;

use rusb::{Device, GlobalContext, HotplugBuilder, UsbContext};
use tegra_rcm::Rcm;

use egui::Context;

use super::ThreadSwitchResult;

struct HotplugHandler {
    tswitch: ThreadSwitchResult,
    ctx: Context,
}

const SWITCH_VID: u16 = 0x0955;
const SWITCH_PID: u16 = 0x7321;

impl rusb::Hotplug<GlobalContext> for HotplugHandler {
    /// Gets called whenever a new usb device arrives
    fn device_arrived(&mut self, device: Device<GlobalContext>) {
        if let Ok(device_desc) = device.device_descriptor() {
            if device_desc.vendor_id() == SWITCH_VID && device_desc.product_id() == SWITCH_PID {
                //println!("Switch arrived {:?}", device);

                let dev = device.open().unwrap();
                let rcm = Rcm::with_device_handle(dev);

                let lock = self.tswitch.lock();
                if let Ok(mut inner) = lock {
                    *inner = Ok(rcm);
                    self.ctx.request_repaint();
                }
            }
        }
    }

    /// Gets called whenever a usb device leaves
    fn device_left(&mut self, device: Device<GlobalContext>) {
        if let Ok(device_desc) = device.device_descriptor() {
            if device_desc.vendor_id() == SWITCH_VID && device_desc.product_id() == SWITCH_PID {
                //println!("Switch left {:?}", device);

                let lock = self.tswitch.lock();

                if let Ok(mut inner) = lock {
                    *inner = Err(Error::SwitchNotFound);
                    self.ctx.request_repaint();
                }
            }
        }
    }
}

/// create a hotplug setup, this blocks
fn create_hotplug(tswitch: ThreadSwitchResult, ctx: Context) {
    if rusb::has_hotplug() {
        let context = rusb::GlobalContext::default();

        let _hotplug = HotplugBuilder::new()
            .enumerate(true)
            .register(&context, Box::new(HotplugHandler { tswitch, ctx }))
            .expect("We where able to successfully wrap the context");

        loop {
            // blocks thread
            context.handle_events(None).unwrap();
        }
    } else {
        panic!("libusb hotplug API unsupported");
    }
}

/// Spawn a seperate thread too
pub fn spawn_thread(tswitch: ThreadSwitchResult, ctx: Context) {
    thread::spawn(move || create_hotplug(tswitch, ctx));
}
