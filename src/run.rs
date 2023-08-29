use std::time::Duration;

use console::{style, Emoji};
use tegra_rcm::{Payload, Switch};

use crate::cli::{Add, Device, Execute, Gui, List, Remove};
use crate::error::Error;
use crate::{favorites::Favorites, spinner, switch::SwitchDevice, usb::spawn_thread};

type CliError = Error;

const EMOJI_FOUND: Emoji = Emoji("🟢 ", "");
const EMOJI_NOT_FOUND: Emoji = Emoji("🔴 ", "");
const EMOJI_ROCKET: Emoji = Emoji("🚀 ", "");

pub(crate) trait RunCommand {
    fn run(self) -> Result<(), CliError>;
}

impl RunCommand for Execute {
    fn run(self) -> Result<(), CliError> {
        let payload = if let Some(favorite) = self.favorite {
            let favorites = Favorites::new();
            let Some(fav) = favorites.get(&favorite) else {
                return Err(Error::FavoriteNotFound(favorite.to_owned()));
            };
            fav.read()?
        } else {
            Payload::read(&self.payload)?
        };

        if !self.wait {
            let switch = Switch::new()?;
            let Some(switch) = switch else {
                println!("{}Switch in RCM mode not found", EMOJI_NOT_FOUND);
                return Ok(());
            };
            switch.execute(&payload)?;
            println!("{}Payload executed!", EMOJI_ROCKET);

            Ok(())
        } else {
            let switch = SwitchDevice::new()?;
            let spinner = spinner();
            spawn_thread(
                switch.clone(),
                Box::new(move || {
                    if let Some(s) = switch.0.lock().unwrap().take() {
                        s.execute(&payload)
                            .expect("Execute should have been successful");
                        spinner.finish_with_message(format!("{}Payload executed!", EMOJI_ROCKET));
                        std::process::exit(0)
                    }
                }),
            );

            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

impl RunCommand for Device {
    fn run(self) -> Result<(), CliError> {
        if !self.wait {
            let switch = Switch::new()?;
            if switch.is_none() {
                println!("{}Switch in RCM mode not found", EMOJI_NOT_FOUND);
                return Ok(());
            };

            println!("{}Switch is RCM mode and connected", EMOJI_FOUND);

            Ok(())
        } else {
            let switch = SwitchDevice::new()?;
            let spinner = spinner();
            spawn_thread(
                switch.clone(),
                Box::new(move || {
                    if switch.0.lock().unwrap().is_some() {
                        spinner.finish_and_clear();
                        println!("{}Switch is RCM mode and connected", EMOJI_FOUND);
                        std::process::exit(0)
                    }
                }),
            );

            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

impl RunCommand for List {
    fn run(self) -> Result<(), CliError> {
        let favorites = Favorites::new();

        let mut count = 0;
        for entry in favorites.iter() {
            println!("{}", style(entry.name()));
            count += 1;
        }

        if count == 0 {
            println!("No favorites");
        }
        Ok(())
    }
}

impl RunCommand for Add {
    fn run(self) -> Result<(), CliError> {
        let mut favorites = Favorites::new();
        let file = favorites.add(&self.payload, true)?;
        println!("Successfully added favorite: {}", style(file).cyan());
        Ok(())
    }
}

impl RunCommand for Remove {
    fn run(self) -> Result<(), CliError> {
        let mut favorites = Favorites::new();

        favorites.remove_str(&self.favorite)?;
        println!(
            "Successfully removed favorite: {}",
            style(&self.favorite).cyan()
        );
        Ok(())
    }
}

#[cfg(feature = "gui")]
impl RunCommand for Gui {
    fn run(self) -> Result<(), CliError> {
        crate::gui::gui();
        Ok(())
    }
}
