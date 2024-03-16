use console::{style, Emoji};
use tegra_rcm::{Payload, Switch, SwitchError};

use crate::cli::{Add, Device, Execute, Gui, List, Remove};
use crate::error::Error;
use crate::usb::spawn_thread;
use crate::{favorites::Favorites, spinner};

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

        let success_msg = format!("{}Payload executed!", EMOJI_ROCKET);

        if !self.wait {
            let switch = Switch::find();
            let Ok(mut switch) = switch else {
                println!("{}Switch in RCM mode not found", EMOJI_NOT_FOUND);
                return Ok(());
            };
            let handle = switch.handle()?;
            handle.execute(&payload)?;
            println!("{success_msg}");
        } else {
            let _spinner = spinner();

            let switch = Switch::find();
            if let Ok(mut switch) = switch {
                switch.handle()?.execute(&payload)?;
                return Ok(());
            }

            let rx = spawn_thread();
            while let Ok(switch) = rx.recv() {
                match switch {
                    Ok(mut switch) => {
                        switch.handle()?.execute(&payload)?;
                        return Ok(());
                    }
                    Err(SwitchError::SwitchNotFound) => (),
                    Err(e) => return Err(e.into()),
                }
            }
        }
        Ok(())
    }
}

impl RunCommand for Device {
    fn run(self) -> Result<(), CliError> {
        if !self.wait {
            match Switch::find() {
                Ok(_) => println!("{}Switch is in RCM mode and connected", EMOJI_FOUND),
                Err(SwitchError::SwitchNotFound) => {
                    println!("{}Switch in RCM mode not found", EMOJI_NOT_FOUND)
                }
                Err(e) => return Err(e.into()),
            }
        } else {
            let _spinner = spinner();

            let switch = Switch::find();
            if let Ok(mut switch) = switch {
                switch.handle()?;
                println!("{}Switch is in RCM mode and connected", EMOJI_FOUND);
                return Ok(());
            }

            let rx = spawn_thread();
            while let Ok(switch) = rx.recv() {
                match switch {
                    Ok(mut switch) => {
                        switch.handle()?;
                        println!("{}Switch is in RCM mode and connected", EMOJI_FOUND);
                        return Ok(());
                    }
                    Err(SwitchError::SwitchNotFound) => (),
                    Err(e) => return Err(e.into()),
                }
            }
        }
        Ok(())
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
        println!("Successfully added favorite: {}", style(file.name()).cyan());
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
        crate::gui::gui().expect("GUI is able to be started");
        Ok(())
    }
}
