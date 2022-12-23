use rusb::{has_hotplug, Device, DeviceHandle, GlobalContext, Hotplug, HotplugBuilder, UsbContext};

use super::Actions;
use crate::device::{SwitchDevice, SWITCH_PID, SWITCH_VID};
use crate::Switch;

impl Switch {
    /// Create a new Rcm object from an existing DeviceHandle
    /// Should not have its interface claimed yet
    fn with_device_handle(device: DeviceHandle<GlobalContext>) -> Self {
        Self::with_device(SwitchDevice::with_device_handle(device)).unwrap()
    }
}

struct HotplugHandler {
    inner: Box<dyn Actions>,
}
unsafe impl Send for HotplugHandler {}

impl Hotplug<GlobalContext> for HotplugHandler {
    /// Gets called whenever a new usb device arrives
    fn device_arrived(&mut self, device: Device<GlobalContext>) {
        // if this is not Ok, it probably got unplugged really fast
        if let Ok(dev) = device.open() {
            let rcm = Switch::with_device_handle(dev);
            self.inner.arrives(rcm);
        }
    }

    /// Gets called whenever a usb device leaves
    fn device_left(&mut self, _device: Device<GlobalContext>) {
        self.inner.leaves();
    }
}

/// create a hotplug setup, this blocks
pub fn create_hotplug(data: Box<dyn Actions>) {
    if has_hotplug() {
        let context = rusb::GlobalContext::default();

        let _hotplug = HotplugBuilder::new()
            .vendor_id(SWITCH_VID)
            .product_id(SWITCH_PID)
            .enumerate(true)
            .register(context, Box::new(HotplugHandler { inner: data }))
            .expect("We where able to successfully wrap the context");

        loop {
            // blocks thread
            context.handle_events(None).unwrap();
        }
    } else {
        panic!("libusb hotplug API unsupported");
    }
}
