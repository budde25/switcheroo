use libusbk::{has_hotplug, Device, Hotplug, HotplugBuilder};
use log::error;

use crate::device::{SwitchDevice, SWITCH_PID, SWITCH_VID};
use crate::{Actions, Switch};

use super::{HotplugError, HotplugHandler};

impl Hotplug for HotplugHandler {
    /// Gets called whenever a new usb device arrives
    fn device_arrived(&mut self, device: Device) {
        // if this is not Ok, it probably got unplugged really fast
        if let Ok(dev) = device.open() {
            let device = SwitchDevice::with_device_handle(dev);
            // prapogate error
            let switch = match Switch::with_device(device) {
                Ok(switch) => switch,
                Err(e) => {
                    error!("Failed to initialize switch: {e}");
                    return;
                }
            };

            // if this is not Some, it probably got unplugged really fast
            if let Some(switch) = switch {
                self.inner.arrives(switch);
            }
        }
    }

    /// Gets called whenever a usb device leaves
    fn device_left(&mut self, _device: Device) {
        self.inner.leaves();
    }
}

/// create a hotplug setup, this blocks
pub fn create_hotplug(data: Box<dyn Actions>) -> Result<(), HotplugError> {
    if !has_hotplug() {
        return Err(HotplugError::NotSupported);
    }

    let mut register = HotplugBuilder::new()
        .vendor_id(SWITCH_VID as i32)
        .product_id(SWITCH_PID as i32)
        .register(Box::new(HotplugHandler { inner: data }))
        .expect("We where able to successfully wrap the context");

    register.init().expect("Register init should pass");

    let leak = Box::new(register);
    let _ = Box::leak(leak);

    Ok(())
}
