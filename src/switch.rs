use std::sync::{Arc, Mutex};

use tegra_rcm::{Payload, Switch, SwitchError};

#[derive(Debug, Clone)]
pub struct SwitchThreaded(pub Arc<Mutex<Option<Switch>>>);

impl SwitchThreaded {
    /// Create a new switch device
    /// This is proteced by a mutex and arc so it is thread safe
    pub fn new() -> Result<Self, SwitchError> {
        match Switch::new()? {
            Some(s) => Ok(Self(Arc::new(Mutex::new(Some(s))))),
            None => Ok(Self(Arc::new(Mutex::new(None)))),
        }
    }

    /// Executes a payload returning any errors
    pub fn execute(&mut self, payload: &Payload) -> Result<(), SwitchError> {
        let mut guard = self.0.lock().expect("Lock should not be poisoned");

        let Some(switch) = guard.take() else {
            return Err(SwitchError::SwitchNotFound);
        };

        switch.execute(payload)
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
    switch: SwitchThreaded,
    state: State,
}

impl SwitchData {
    /// Create some new Switch Data
    pub fn new() -> Result<Self, SwitchError> {
        Ok(Self {
            switch: SwitchThreaded::new()?,
            state: State::NotAvailable,
        })
    }

    /// Check if we need to change our current state
    pub fn update_state(&mut self) {
        if self.state == State::Done {
            return;
        }

        let guard = self.switch.0.lock().expect("Lock should not be poisoned");

        match &*guard {
            Some(_) => self.state = State::Available,
            None => self.state = State::NotAvailable,
        }
    }

    pub fn execute(&mut self, payload: &Payload) -> Result<(), SwitchError> {
        self.switch.execute(payload)
    }

    pub fn reset_state(&mut self) {
        self.state = State::NotAvailable;
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn switch(&self) -> SwitchThreaded {
        self.switch.clone()
    }
}
