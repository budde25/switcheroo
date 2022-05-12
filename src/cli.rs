use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    #[clap(short, long, parse(from_occurrences))]
    pub verbose: usize,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Executes a payload
    Execute {
        /// Path to the payload file
        payload: PathBuf,

        /// Wait for device to be connected
        #[clap(short, long)]
        wait: bool,
    },
    /// Checks if a Switch in RCM mode is detected
    Device,

    /// Opens the GUI
    #[cfg(feature = "gui")]
    Gui,
}
