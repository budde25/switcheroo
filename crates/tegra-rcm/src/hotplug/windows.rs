use libusbk::DeviceHandle;

use crate::device::{Device, SwitchDevice, SWITCH_PID, SWITCH_VID};
use crate::{Actions, Rcm};

impl Rcm {
    /// Create a new Rcm object from an existing DeviceHandle
    /// Should not have its interface claimed yet
    fn with_device_handle(device: DeviceHandle) -> Self {
        Self::with_device(SwitchDevice::with_device_handle(device))
    }
}

struct HotplugHandler {
    inner: Box<dyn Actions>,
}
unsafe impl Send for HotplugHandler {}

impl libusbk::Hotplug for HotplugHandler {
    /// Gets called whenever a new usb device arrives
    fn device_arrived(&mut self, device: Device) {
        if let Ok(device_desc) = device.device_descriptor() {
            if device_desc.vendor_id() == SWITCH_VID && device_desc.product_id() == SWITCH_PID {
                // if this is not Ok, it probably got unplugged really fast
                if let Ok(dev) = device.open() {
                    let rcm = Rcm::with_device_handle(dev);
                    self.inner.arrives(rcm);
                }
            }
        }
    }

    /// Gets called whenever a usb device leaves
    fn device_left(&mut self, device: Device) {
        if let Ok(device_desc) = device.device_descriptor() {
            if device_desc.vendor_id() == SWITCH_VID && device_desc.product_id() == SWITCH_PID {
                self.inner.leaves();
            }
        }
    }
}

/// create a hotplug setup, this blocks
pub fn create_hotplug(data: Box<dyn Actions>) {
    if libusk::has_hotplug() {
        let context = rusb::GlobalContext::default();

        let _hotplug = HotplugBuilder::new()
            .enumerate(true)
            .register(&context, Box::new(HotplugHandler { inner: data }))
            .expect("We where able to successfully wrap the context");

        loop {
            // blocks thread
            context.handle_events(None).unwrap();
        }
    } else {
        panic!("libusbK hotplug API unsupported");
    }
}
