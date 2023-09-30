use log::error;
use rusb::{has_hotplug, Device, GlobalContext, Hotplug, HotplugBuilder, UsbContext};

use super::{Actions, HotplugError, HotplugHandler};
use crate::device::{SwitchDevice, SWITCH_PID, SWITCH_VID};
use crate::Switch;

impl Hotplug<GlobalContext> for HotplugHandler {
    /// Gets called whenever a new usb device arrives
    fn device_arrived(&mut self, device: Device<GlobalContext>) {
        // if this is not Ok, it probably got unplugged really fast
        if let Ok(dev) = device.open() {
            let device = SwitchDevice::with_device_handle(dev);
            // propagate error
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
    fn device_left(&mut self, _device: Device<GlobalContext>) {
        self.inner.leaves();
    }
}

/// create a hotplug setup, this blocks
pub fn create_hotplug(data: Box<dyn Actions>) -> Result<(), HotplugError> {
    if !has_hotplug() {
        return Err(HotplugError::NotSupported);
    }

    let context = GlobalContext::default();

    let _hotplug = HotplugBuilder::new()
        .vendor_id(SWITCH_VID)
        .product_id(SWITCH_PID)
        .enumerate(true)
        .register(context, Box::new(HotplugHandler { inner: data }))
        .expect("We where able to successfully wrap the context");

    loop {
        // blocks thread
        context
            .handle_events(None)
            .expect("We are able to handle USB hotplug events");
    }
}
