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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    NotAvailable,
    Available,
    Done,
}

#[derive(Debug, Clone)]
pub struct SwitchData {
    switch: Switch,
    state: State,
}

impl SwitchData {
    /// Create some new Switch Data
    pub fn new() -> Self {
        Self {
            switch: Switch::new(),
            state: State::NotAvailable,
        }
    }

    /// Check if we need to change our current state
    pub fn update_state(&mut self) -> Result<State, Error> {
        if self.state == State::Done {
            return Ok(self.state);
        }

        let guard = self.switch.0.lock().expect("Lock should not be poisoned");

        match &*guard {
            Ok(rcm) => {
                match rcm.validate() {
                    Ok(_) => self.state = State::Available,
                    Err(e) => {
                        self.state = State::NotAvailable;
                        return Err(e);
                    }
                }

                Ok(self.state)
            }
            Err(e) => {
                self.state = State::NotAvailable;
                if e != &tegra_rcm::Error::SwitchNotFound {
                    return Err(e.clone());
                }
                Ok(self.state)
            }
        }
    }

    pub fn execute(&mut self, payload: &Payload) -> Result<(), Error> {
        match self.switch.execute(payload) {
            Ok(_) => self.state = State::Done,
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn reset_state(&mut self) {
        self.state = State::NotAvailable;
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn switch(&self) -> Switch {
        self.switch.clone()
    }
}
