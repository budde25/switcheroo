use std::sync::{Arc, Mutex};

use tegra_rcm::{Error, Payload, Rcm};

#[derive(Debug, Clone)]
pub struct Switch(pub Arc<Mutex<Result<Rcm, Error>>>);

impl Switch {
    /// Create a new switch device
    /// This is proteced by a mutex and arc so it is thread safe
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Rcm::new(false))))
    }

    /// Executes a payload returning any errors
    pub fn execute(&mut self, payload: &Payload) -> Result<(), Error> {
        let mut guard = self.0.lock().expect("Lock should not be poisoned");

        let switch = match &mut *guard {
            Ok(a) => a,
            Err(e) => return Err(e.clone()),
        };

        // its ok if it gets init more than once, it skips previous inits
        switch.init()?;

        // We need to read the device id first
        let _ = switch.read_device_id()?;
        switch.execute(payload)?;

        Ok(())
    }
}
