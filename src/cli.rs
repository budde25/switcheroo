use camino::Utf8PathBuf;
use clap_verbosity_flag::Verbosity;

use clap::builder::Styles;
use clap::{builder::ArgPredicate, Args, Parser, Subcommand, ValueHint};

fn style() -> Styles {
    use clap::builder::styling::*;
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
        .valid(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .invalid(AnsiColor::Yellow.on_default() | Effects::BOLD)
}

#[derive(Debug, Parser)]
#[command(name = "switcheroo", author, version, about, long_about = None)]
#[clap(styles = style())]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub(crate) command: Commands,

    #[clap(flatten)]
    pub(crate) verbose: Verbosity,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Executes a payload on a connected Switch
    Execute(Execute),
    /// Checks if a Switch is connected and booted to RCM mode
    Device(Device),
    /// Lists favorite binaries
    List(List),
    /// Add a favorite binary
    Add(Add),
    /// Remove a favorite binary
    Remove(Remove),
    /// Opens the graphical user interface
    #[cfg(feature = "gui")]
    Gui(Gui),
}

#[derive(Debug, Args)]
pub(crate) struct Execute {
    /// Path to the payload file
    #[clap(value_hint = ValueHint::FilePath, required = false, required_unless_present = "favorite", default_value_if("favorite", ArgPredicate::IsPresent, "/"))]
    pub(crate) payload: Utf8PathBuf,

    /// Use a favorite payload instead
    #[arg(short, long)]
    pub(crate) favorite: Option<String>,

    /// Wait for device to be connected
    #[arg(short, long)]
    pub(crate) wait: bool,
}

#[derive(Debug, Args)]
pub(crate) struct Device {
    /// Wait for device to be connected
    #[arg(short, long)]
    pub(crate) wait: bool,
}

#[derive(Debug, Args)]
pub(crate) struct List;

#[derive(Debug, Args)]
pub(crate) struct Add {
    /// Path to the binary file
    #[clap(value_hint = ValueHint::FilePath)]
    pub(crate) payload: Utf8PathBuf,
}

#[derive(Debug, Args)]
pub(crate) struct Remove {
    pub(crate) favorite: String,
}

#[derive(Debug, Args)]
pub(crate) struct Gui;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
