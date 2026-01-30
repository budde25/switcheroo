use crate::Result;

use crate::rcm::protocol;
use crate::usb::{Device, SwitchDevice};
use crate::Payload;
use protocol::RcmHandle;

/// A Switch device in RCM (Recovery Mode)
///
/// This represents a Nintendo Switch device connected in RCM mode,
/// ready to receive and execute payloads via the Fusée Gelée exploit.
#[derive(Debug, Clone)]
pub struct Switch {
    switch: SwitchDevice,
}

impl Switch {
    /// Create a new Switch from an existing SwitchDevice
    ///
    /// This is used internally by the hotplug system.
    /// Should not have its interface claimed yet.
    pub(crate) fn new(device: SwitchDevice) -> Self {
        Self { switch: device }
    }

    /// Find and connect to a Switch device in RCM mode
    ///
    /// This will search for a connected Switch device in RCM mode.
    /// Returns an error if no device is found.
    pub fn find() -> Result<Self> {
        let device = SwitchDevice::find_device()?;
        Ok(Self { switch: device })
    }

    /// Execute a payload on the Switch
    ///
    /// This will send the payload to the Switch and trigger the exploit.
    /// The payload is executed and the Switch will boot into the payload.
    ///
    /// This method consumes the Switch as the device connection is closed
    /// after execution.
    pub fn execute(mut self, payload: &Payload) -> Result<()> {
        let handle = self.handle()?;
        handle.execute_payload(payload)
    }

    /// Initialize a handle for internal operations
    fn handle(&mut self) -> Result<RcmHandle> {
        let handle = self.switch.init()?;
        Ok(RcmHandle::new(handle))
    }
}
