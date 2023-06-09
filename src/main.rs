#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use camino::{Utf8Path, Utf8PathBuf};

use clap::Parser;
use color_eyre::eyre::{bail, Result};
use favorites::Favorites;
use tegra_rcm::{Payload, Switch};

mod cli;
mod favorites;
#[cfg(feature = "gui")]
mod gui;

use cli::{Cli, Commands};

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
        Commands::List => _ = list()?,
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
        let favorites = Favorites::new()?;
        let Some(fav) = favorites.get(&favorite) else {
            bail!("Failed to execute favorite: `{}` not found", &favorite);
        };
        fav.read()?
    } else {
        Payload::read(&path)?
    };

    let mut switch = Switch::new()?;
    while wait && switch.is_some() {
        switch = Switch::new()?;
    }
    if let Some(switch) = switch {
        switch.execute(&payload)?;
    } else {
        bail!("Switch not found")
    }

    println!("Done!");
    Ok(())
}

fn device(wait: bool) -> Result<()> {
    let switch = Switch::new()?;
    if switch.is_none() {
        println!("[x] Switch in RCM mode not found");
        return Ok(());
    };

    println!("[✓] Switch is RCM mode and connected");

    Ok(())
}

/// Prints the favorites to stdout
/// Errors on trying to read from the favorites directory
/// Returns the number of entries
fn list() -> Result<usize> {
    let favorites = Favorites::new()?;
    let list = favorites.list();

    if list.is_empty() {
        println!("No favorites");
        return Ok(0);
    }

    for entry in list {
        println!("{}", entry.name());
    }

    Ok(list.len())
}

fn add(payload: &Utf8Path) -> Result<()> {
    let favorites = Favorites::new()?;
    favorites.add(payload.as_std_path(), true)?;
    println!(
        "Successfully added favorite: `{}`",
        &payload.file_name().unwrap()
    );
    Ok(())
}

fn remove(favorite: &str) -> Result<()> {
    let mut favorites = Favorites::new()?;

    let Some(fav) = favorites.get(favorite) else {
        bail!("Failed to remove favorite: `{}` not found", &favorite);
    };
    let fav = fav.clone();

    favorites.remove(&fav)?;
    println!("Successfully removed favorite: `{}`", &fav.name());
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
