use std::fs;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Result;
use rcm_lib::{Payload, Rcm};

#[derive(Parser)]
struct Context {
    /// Path to the payload file
    payload: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let context = Context::parse();

    let payload_bytes = fs::read(context.payload)?;
    let payload = Payload::new(&payload_bytes);

    let mut switch = Rcm::new(false)?;
    switch.read_device_id();
    switch.write(&payload.data)?;
    switch.switch_to_highbuf()?;

    println!("Smashing the stack!");

    // we expect a timeout
    let err = switch.trigger_controlled_memcopy().unwrap_err();
    println!("Done, yay!, should be timeout: {}", err);

    Ok(())
}
