#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::path::PathBuf;
use std::{env, fs};

use clap::StructOpt;
use color_eyre::eyre::{Context, Result};
use favorites::Favorites;
use tegra_rcm::{Error, Payload, Rcm};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

mod cli;
mod favorites;
#[cfg(feature = "gui")]
mod gui;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    color_eyre::install()?;

    // check if we should start the gui, rn we start with env var set, or platform = windows
    #[cfg(feature = "gui")]
    check_gui_mode()?;

    let args = Cli::parse();

    set_log_level(args.verbose);

    match args.command {
        Commands::Execute {
            payload,
            favorite,
            wait,
        } => execute(payload, favorite, wait)?,
        Commands::Device {} => device()?,
        Commands::List => list()?,
        Commands::Add { payload } => add(payload)?,
        Commands::Remove { favorite } => remove(favorite)?,
        #[cfg(feature = "gui")]
        Commands::Gui {} => gui::gui()?,
    }
    Ok(())
}

/// sets the log level
fn set_log_level(verbosity: u8) {
    let filter = EnvFilter::builder();
    let filter = match verbosity {
        1 => filter.with_default_directive(LevelFilter::INFO.into()),
        2 => filter.with_default_directive(LevelFilter::DEBUG.into()),
        3 => filter.with_default_directive(LevelFilter::TRACE.into()),
        _ => filter.with_default_directive(LevelFilter::WARN.into()),
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter.from_env_lossy())
        .init();
}

fn execute(payload: String, favorite: bool, wait: bool) -> Result<()> {
    let payload_path = if favorite {
        let favorites = Favorites::new()?;
        if let Some(dir) = favorites.get(&payload)? {
            dir.path()
        } else {
            println!("Failed to execute favorite: `{}` not found", &payload); // TODO: should we exit with 1?
            return Ok(());
        }
    } else {
        PathBuf::from(&payload)
    };

    let payload_bytes = fs::read(&payload_path)
        .wrap_err_with(|| format!("Failed to read payload from: {}", &payload))?;
    let pay = Payload::new(&payload_bytes)?;

    let mut switch = Rcm::new(wait)?;
    switch.init()?;
    println!("Smashing the stack!");

    // We need to read the device id first
    let _ = switch.read_device_id()?;
    switch.execute(&pay)?;

    println!("Done!");
    Ok(())
}

fn device() -> Result<()> {
    let switch = Rcm::new(false);

    let err = match switch {
        Ok(_) => {
            println!("[✓] Switch is RCM mode and connected");
            return Ok(());
        }
        Err(e) => e,
    };

    match err {
        Error::SwitchNotFound => println!("[x] Switch in RCM mode not found"),
        Error::AccessDenied => {
            switch.wrap_err_with(|| "USB permission error\nSee \"https://github.com/budde25/switcheroo#linux-permission-denied-error\" to troubleshoot".to_string())?;
        }
        _ => return Err(err.into()),
    };

    Ok(())
}

fn list() -> Result<()> {
    let favorites = Favorites::new()?;
    let list: Vec<_> = favorites.list()?.filter_map(|x| x.ok()).collect();

    if list.is_empty() {
        println!("No favorites");
    } else {
        for entry in list {
            println!("{}", entry.file_name().to_string_lossy());
        }
    }

    Ok(())
}

fn add(payload: PathBuf) -> Result<()> {
    let favorites = Favorites::new()?;
    favorites.add(&payload, true)?;
    println!(
        "Successfully added favorite: `{}`",
        &payload.file_name().unwrap().to_string_lossy()
    );
    Ok(())
}

fn remove(favorite: String) -> Result<()> {
    let favorites = Favorites::new()?;
    match favorites.remove(&favorite)? {
        true => println!("Successfully removed favorite: `{}`", &favorite),
        false => println!("Failed to remove favorite: `{}` not found", &favorite), // TODO: should we exit with 1?
    }

    Ok(())
}

#[cfg(feature = "gui")]
fn check_gui_mode() -> Result<()> {
    // FIXME: only gui mode on windows
    #[cfg(target_os = "windows")]
    launch_gui()?;

    match env::var_os("SWITCHEROO_GUI_ONLY") {
        None => return Ok(()),
        Some(gui_only) => {
            if gui_only == "0" {
                launch_gui()?;
            }
        }
    };
    Ok(())
}

#[cfg(feature = "gui")]
fn launch_gui() -> Result<()> {
    set_log_level(3);
    gui::gui()
}
