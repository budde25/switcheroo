#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use indicatif::{ProgressBar, ProgressStyle};
use run::RunCommand;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;

mod cli;
mod error;
mod favorites;
#[cfg(feature = "gui")]
mod gui;
mod run;
mod switch;
mod usb;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    // check if we should start the gui, rn we start with env var set, or platform = windows
    #[cfg(feature = "gui")]
    launch_gui_only_mode();

    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(convert_filter(cli.verbose.log_level_filter()))
        .init();

    match cli.command {
        Commands::Execute(cmd) => cmd.run()?,
        Commands::Device(cmd) => cmd.run()?,
        Commands::List(cmd) => cmd.run()?,
        Commands::Add(cmd) => cmd.run()?,
        Commands::Remove(cmd) => cmd.run()?,
        #[cfg(feature = "gui")]
        Commands::Gui(cmd) => cmd.run()?,
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

/// Launch the gui if we are in gui only mode
/// Most commonly by checking the env variable
/// SWITCHEROO_GUI_ONLY is set to "0"
#[cfg(feature = "gui")]
fn launch_gui_only_mode() {
    let Some(gui_only) = std::env::var_os("SWITCHEROO_GUI_ONLY") else {
        return;
    };

    if gui_only == "0" {
        gui::gui();
    }
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
    pb.set_message("Searching for Switch in RCM mode");
    pb
}
