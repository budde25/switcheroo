use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Verbosity
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Executes a provided payload
    Execute {
        /// Path to the payload file, or a favorite if flag is passed
        #[clap(action)]
        payload: String,

        /// Use a favorite payload
        #[clap(short, long, action)]
        favorite: bool,

        /// Wait for device to be connected
        #[clap(short, long, action)]
        wait: bool,
    },
    /// Checks if a Switch in RCM mode is detected
    Device,

    /// Lists favorites
    List,

    /// Add a favorite
    Add {
        /// Path to the payload file
        #[clap(action)]
        payload: PathBuf,
    },

    /// Remove a favorite
    Remove {
        #[clap(action)]
        favorite: String,
    },

    /// Opens the Graphical User Interface
    #[cfg(feature = "gui")]
    Gui,
}
