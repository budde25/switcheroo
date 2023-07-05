#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use camino::{Utf8Path, Utf8PathBuf};
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::{bail, Result};
use favorites::Favorites;
use tegra_rcm::{Payload, Switch};

mod cli;
mod favorites;
#[cfg(feature = "gui")]
mod gui;
mod switch;
mod usb;

use cli::{Cli, Commands};
use usb::spawn_thread;

use crate::switch::SwitchDevice;

const EMOJI_FOUND: Emoji = Emoji("🟢 ", "");
const EMOJI_NOT_FOUND: Emoji = Emoji("🔴 ", "");
const EMOJI_ROCKET: Emoji = Emoji("🚀 ", "");

fn main() -> Result<()> {
    color_eyre::install()?;

    // check if we should start the gui, rn we start with env var set, or platform = windows
    #[cfg(feature = "gui")]
    launch_gui_only_mode();

    let args = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(convert_filter(args.verbose.log_level_filter()))
        .init();

    match args.command {
        Commands::Execute {
            payload,
            favorite,
            wait,
        } => execute(payload, favorite, wait)?,
        Commands::Device { wait } => device(wait)?,
        Commands::List => _ = list(),
        Commands::Add { payload } => add(&payload)?,
        Commands::Remove { favorite } => remove(&favorite)?,
        #[cfg(feature = "gui")]
        Commands::Gui {} => gui::gui(),
    }
    Ok(())
}

fn convert_filter(filter: log::LevelFilter) -> tracing_subscriber::filter::LevelFilter {
    match filter {
        log::LevelFilter::Off => tracing_subscriber::filter::LevelFilter::OFF,
        log::LevelFilter::Error => tracing_subscriber::filter::LevelFilter::ERROR,
        log::LevelFilter::Warn => tracing_subscriber::filter::LevelFilter::WARN,
        log::LevelFilter::Info => tracing_subscriber::filter::LevelFilter::INFO,
        log::LevelFilter::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
        log::LevelFilter::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
    }
}

fn execute(path: Utf8PathBuf, favorite: Option<String>, wait: bool) -> Result<()> {
    let payload = if let Some(favorite) = favorite {
        let favorites = Favorites::new();
        let Some(fav) = favorites.get(&favorite) else {
            bail!("Failed to execute favorite: `{}` not found", &favorite);
        };
        fav.read()?
    } else {
        Payload::read(&path)?
    };

    if !wait {
        let switch = Switch::new()?;
        let Some(switch) = switch else {
            println!("{}Switch in RCM mode not found", EMOJI_NOT_FOUND);
            return Ok(());
        };
        switch.execute(&payload)?;
        println!("{}Payload excuted!", EMOJI_ROCKET);

        Ok(())
    } else {
        let switch = SwitchDevice::new()?;
        let spinner = spinner();
        spawn_thread(
            switch.clone(),
            Box::new(move || {
                if let Some(s) = switch.0.lock().unwrap().take() {
                    s.execute(&payload)
                        .expect("Excute should have been successful");
                    spinner.finish_with_message(format!("{}Payload excuted!", EMOJI_ROCKET));
                    std::process::exit(0)
                }
            }),
        );

        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

fn device(wait: bool) -> Result<()> {
    if !wait {
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

/// Prints the favorites to stdout
/// Errors on trying to read from the favorites directory
/// Returns the number of entries
fn list() -> usize {
    let favorites = Favorites::new();

    let mut count = 0;
    for entry in favorites.iter() {
        println!("{}", style(entry.name()));
        count += 1;
    }

    if count == 0 {
        println!("No favorites");
    }

    count
}

fn add(payload: &Utf8Path) -> Result<()> {
    let favorites = Favorites::new();
    favorites.add(payload.as_std_path(), true)?;
    println!(
        "Successfully added favorite: {}",
        style(&payload.file_name().unwrap()).cyan()
    );
    Ok(())
}

fn remove(favorite: &str) -> Result<()> {
    let mut favorites = Favorites::new();

    let Some(fav) = favorites.get(favorite) else {
        bail!("Failed to remove favorite, not found: {}", style(favorite).red());
    };
    let fav = fav.clone();

    favorites.remove(&fav)?;
    println!(
        "Successfully removed favorite: {}",
        style(&fav.name()).cyan()
    );
    Ok(())
}

/// Launch the gui if we are in gui only mode
/// Most commonly by checking the env variable
/// SWITCHEROO_GUI_ONLY is set to "0"
#[cfg(feature = "gui")]
fn launch_gui_only_mode() {
    let Some(gui_only) = std::env::var_os("SWITCHEROO_GUI_ONLY") else {
        return;
    };

    if gui_only == "0" {
        launch_gui();
    }
}

#[cfg(feature = "gui")]
fn launch_gui() {
    gui::gui();
}

pub fn spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    pb.set_message("Waiting for Switch in RCM mode...");
    pb
}
