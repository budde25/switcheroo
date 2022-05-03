use clap::IntoApp;
use clap_complete::generate_to;
use clap_complete::Shell::{Bash, Fish, Zsh};
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let shells = vec![Bash, Fish, Zsh];

    for shell in shells {
        let mut cmd = Cli::command();
        let path = generate_to(
            shell,
            &mut cmd,     // We need to specify what generator to use
            "switcheroo", // We need to specify the bin name manually
            &outdir,      // We need to specify where to write to
        )?;
        println!("cargo:warning=completion file is generated: {:?}", path);
    }

    Ok(())
}
