use std::sync::mpsc::Sender;

use libusbk::{has_hotplug, Device, Hotplug, HotplugBuilder};
use log::{error, info};

use crate::device::{SwitchDevice, RCM_PID, RCM_VID};
use crate::{Switch, SwitchError};

use super::{HotplugError, HotplugHandler};

impl Hotplug for HotplugHandler {
    /// Gets called whenever a new usb device arrives
    fn device_arrived(&mut self, device: Device) {
        let device = SwitchDevice { device };
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
    fn device_left(&mut self, _device: Device) {
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
    mut callback: Option<impl Fn() + Send + Sync + 'static>,
) -> Result<(), HotplugError> {
    if !has_hotplug() {
        return Err(HotplugError::NotSupported);
    }

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

    let mut register = HotplugBuilder::new()
        .vendor_id(RCM_VID as i32)
        .product_id(RCM_PID as i32)
        .register(Box::new(hotplug_handler))
        .expect("We where able to successfully wrap the context");

    register.init().expect("Register init should pass");

    let leak = Box::new(register);
    let _ = Box::leak(leak);

    Ok(())
}
