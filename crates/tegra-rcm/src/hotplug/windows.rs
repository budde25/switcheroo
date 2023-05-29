use libusbk::{has_hotplug, Device, DeviceHandle, Hotplug, HotplugBuilder};

use crate::device::{SwitchDevice, SWITCH_PID, SWITCH_VID};
use crate::{Actions, Switch};

impl Hotplug for HotplugHandler {
    /// Gets called whenever a new usb device arrives
    fn device_arrived(&mut self, device: Device) {
        // if this is not Ok, it probably got unplugged really fast
        if let Ok(dev) = device.open() {
            let switch_device = SwitchDevice::with_device_handle(dev);
            Switch::with_device(switch_device).unwrap();
            self.inner.arrives(rcm);
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
