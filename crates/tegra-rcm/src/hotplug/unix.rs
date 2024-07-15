use std::sync::mpsc::Sender;

use log::{error, info};
use rusb::{has_hotplug, Context, Device, Hotplug, HotplugBuilder, UsbContext};

use super::{HotplugError, HotplugHandler};
use crate::device::{SwitchDevice, RCM_PID, RCM_VID};
use crate::switch::Switch;
use crate::SwitchError;

impl Hotplug<Context> for HotplugHandler {
    /// Gets called whenever a new usb device arrives
    fn device_arrived(&mut self, device: Device<Context>) {
        let device = SwitchDevice::new(device);
        let switch = Switch::new(device);

        if let Err(e) = self.sender.send(Ok(switch)) {
            error!("device arrive event {e}");
        }

        info!("rcm device arrived");

        if let Some(callback) = &self.callback {
            callback();
        }
    }

    /// Gets called whenever a usb device leaves
    fn device_left(&mut self, _device: Device<Context>) {
        if let Err(e) = self.sender.send(Err(crate::SwitchError::SwitchNotFound)) {
            error!("device left event {e}");
        }

        info!("rcm device left");

        if let Some(callback) = &self.callback {
            callback();
        }
    }
}

/// create a hotplug setup, this blocks
pub fn create_hotplug(
    tx: Sender<Result<Switch, SwitchError>>,
    callback: Option<impl Fn() + Send + Sync + 'static>,
) -> Result<(), HotplugError> {
    cfg_if::cfg_if! {
        if #[cfg(all(feature = "notify", target_os = "linux"))] {
            super::notify::watcher_hotplug(tx, callback)
                .map_err(|_| HotplugError::Watcher)
        } else {
            libusb_hotplug(tx, callback)
        }
    }
}

pub fn libusb_hotplug(
    tx: Sender<Result<Switch, SwitchError>>,
    callback: Option<impl Fn() + Send + Sync + 'static>,
) -> Result<(), HotplugError> {
    if !has_hotplug() {
        return Err(HotplugError::NotSupported);
    }

    let mut callback = callback;
    let context = Context::new().unwrap();

    let hotplug_handler = match callback.take() {
        Some(callback) => HotplugHandler {
            sender: tx,
            callback: Some(Box::new(callback)),
        },
        None => HotplugHandler {
            sender: tx,
            callback: None,
        },
    };

    let _hotplug = HotplugBuilder::new()
        .vendor_id(RCM_VID)
        .product_id(RCM_PID)
        .enumerate(true)
        .register(context.clone(), Box::new(hotplug_handler))
        .expect("We where able to successfully wrap the context");

    loop {
        // blocks thread
        context
            .handle_events(None)
            .expect("We are able to handle USB hotplug events");
    }
}
