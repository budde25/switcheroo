use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Context, Result};
use rcm_lib::{Error, Payload, Rcm};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();

    match args.command {
        Commands::Execute { payload, wait } => execute(payload, wait)?,
        Commands::Device {} => device()?,
    }
    Ok(())
}

fn execute(payload: PathBuf, wait: bool) -> Result<()> {
    let payload_bytes = fs::read(&payload)
        .wrap_err_with(|| format!("Failed to read payload from: {}", &payload.display()))?;
    let payload = Payload::new(&payload_bytes)?;

    let mut switch = Rcm::new(wait)?;

    println!("Smashing the stack!");

    // We need to read the device id first
    let _ = switch.read_device_id()?;
    switch.execute(payload)?;

    println!("Done!");
    Ok(())
}

fn device() -> Result<()> {
    let switch = Rcm::new(false);
    if let Err(Error::SwitchNotFound) = switch {
        println!("[x] Switch in RCM mode not found")
    } else {
        switch?;
        println!("[✓] Switch is RCM mode and connected")
    }

    Ok(())
}
