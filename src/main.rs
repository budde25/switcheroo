use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Result;
use color_eyre::Help;
use rcm_lib::{Payload, Rcm};

#[derive(Parser)]
struct Context {
    /// Path to the payload file
    payload: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let context = Context::parse();

    let payload = fs::read(context.payload)?;
    let p_builder = Payload::new(&payload);
    println!("{:?}", p_builder.data);
    let mut file = File::create("rust.bin")?;
    file.write_all(&*p_builder.data)?;
    Ok(())
}
