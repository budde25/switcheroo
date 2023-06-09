use camino::Utf8PathBuf;
use clap_verbosity_flag::Verbosity;

use clap::{builder::ArgPredicate, Parser, Subcommand, ValueHint};

#[derive(Parser)]
#[command(name = "switcheroo", author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    #[clap(flatten)]
    pub verbose: Verbosity,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Executes a payload on a connected Switch
    Execute {
        /// Path to the payload file
        #[clap(value_hint = ValueHint::FilePath, required = false, required_unless_present = "favorite", default_value_if("favorite", ArgPredicate::IsPresent, "/"))]
        payload: Utf8PathBuf,

        /// Use a favorite payload instead
        #[arg(short, long)]
        favorite: Option<String>,

        /// Wait for device to be connected
        #[arg(short, long)]
        wait: bool,
    },

    /// Checks if a Switch is connected and booted to RCM mode
    Device {
        /// Wait for device to be connected
        #[arg(short, long)]
        wait: bool,
    },

    /// Lists favorite binaries
    List,

    /// Add a favorite binary
    Add {
        /// Path to the binary file
        #[clap(value_hint = ValueHint::FilePath)]
        payload: Utf8PathBuf,
    },

    /// Remove a favorite binary
    Remove { favorite: String },

    /// Opens the graphical user interface
    #[cfg(feature = "gui")]
    Gui,
}
