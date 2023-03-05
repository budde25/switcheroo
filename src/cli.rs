use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "switcheroo", author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Executes a provided payload
    Execute {
        /// Path to the payload file, or a favorite if flag is passed
        payload: String,

        /// Use a favorite payload
        #[arg(short, long)]
        favorite: bool,

        /// Wait for device to be connected
        #[arg(short, long)]
        wait: bool,
    },
    /// Checks if a Switch in RCM mode is detected
    Device,

    /// Lists favorites
    List,

    /// Add a favorite
    Add {
        /// Path to the payload file
        payload: PathBuf,
    },

    /// Remove a favorite
    Remove { favorite: String },

    /// Opens the Graphical User Interface
    #[cfg(feature = "gui")]
    Gui,
}
