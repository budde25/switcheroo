use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::Shell::{Bash, Fish, Zsh};
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::{env, process};

include!("src/args.rs");

fn main() -> Result<(), Error> {
    let mut outdir = match env::var_os("CARGO_MANIFEST_DIR") {
        None => {
            eprintln!("CARGO_MANIFEST_DIR not defined");
            process::exit(1);
        }
        Some(outdir) => PathBuf::from(outdir),
    };

    app_dir(&outdir)?;
    outdir.push("extra");

    let mut cmd = Args::command();

    let completions_out = outdir.join("completions");
    let man_out = outdir.join("man");

    let shells = [Bash, Fish, Zsh];
    for shell in shells {
        let _path = generate_to(
            shell,
            &mut cmd,         // We need to specify what generator to use
            "switcheroo",     // We need to specify the bin name manually
            &completions_out, // We need to specify where to write to
        )?;
    }

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Vec::default();
    man.render(&mut buffer)?;

    std::fs::write(man_out.join("switcheroo.1"), buffer)?;

    Ok(())
}

/// Doesn't support another target dir
fn app_dir(path: &Path) -> Result<(), Error> {
    let target = path.join("target");
    let bin = target.join("AppDir").join("usr").join("bin");
    fs::create_dir_all(bin)?;
    Ok(())
}
