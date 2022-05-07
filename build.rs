use clap::IntoApp;
use clap_complete::generate_to;
use clap_complete::Shell::{Bash, Fish, Zsh};
use std::fs;
use std::io::Error;
use std::path::Path;
use std::{env, process};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let mut outdir = match env::var_os("CARGO_MANIFEST_DIR") {
        None => {
            eprintln!("CARGO_MANIFEST_DIR not defined");
            process::exit(1);
        }
        Some(outdir) => PathBuf::from(outdir),
    };

    app_dir(&outdir)?;

    // we are gonna push to the extra dir
    outdir.push("extra");
    let mut completions = outdir.clone();
    completions.push("completions");

    let shells = [Bash, Fish, Zsh];
    for shell in shells {
        let mut cmd = Cli::command();
        let _path = generate_to(
            shell,
            &mut cmd,     // We need to specify what generator to use
            "switcheroo", // We need to specify the bin name manually
            &completions, // We need to specify where to write to
        )?;
    }

    Ok(())
}

/// Doesn't support another target dir
fn app_dir(path: &Path) -> Result<(), Error> {
    let target = path.join("target");
    let bin = target.join("AppDir").join("usr").join("bin");
    fs::create_dir_all(&bin)?;
    Ok(())
}
