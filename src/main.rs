use std::fs;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{Context, Result};
use rcm_lib::{Payload, Rcm};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the payload file
    payload: PathBuf,

    /// Wait for device to be connected
    #[clap(short, long)]
    wait: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let payload_bytes = fs::read(&args.payload)
        .wrap_err_with(|| format!("Failed to read payload from: {}", &args.payload.display()))?;
    let payload = Payload::new(&payload_bytes)?;

    let mut switch = Rcm::new(args.wait)?;

    println!("Smashing the stack!");

    // We need to read the device id first
    let _ = switch.read_device_id()?;
    switch.execute(payload)?;

    println!("Done!");
    Ok(())
}
