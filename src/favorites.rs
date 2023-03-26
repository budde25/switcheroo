use color_eyre::eyre::{bail, Result, WrapErr};
use once_cell::sync::Lazy;
use std::fs;
use std::path::{Path, PathBuf};
use tegra_rcm::Payload;

static FAVORITES_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut favorites_dir = dirs::data_dir().expect("System data directory exists");
    favorites_dir.push("switcheroo");
    favorites_dir.push("favorites");

    fs::create_dir_all(&favorites_dir)
        .expect("Permission to create switcheroo favorites directory");
    favorites_dir
});

/// Favorite payloads
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorites {
    list: Vec<Favorite>,
}

impl Favorites {
    /// Create a new Favorites this points to the OS's <data_dir>/switcheroo/favorites folder and creates it if it does not exist
    pub fn new() -> Result<Self> {
        let mut list = Vec::new();
        for entry in fs::read_dir(Self::directory())?.flatten() {
            let file_name = entry.file_name();
            list.push(Favorite::new(
                file_name.to_str().expect("A valid UTF-8 String"),
            ));
        }
        list.sort();

        Ok(Self { list })
    }

    /// Get an iterator of the files in the directory
    pub fn list(&self) -> &[Favorite] {
        &self.list
    }

    /// Add a payload to the favorites directory, if `check_valid` is true, we will make sure that the payload parses correctly (but slower)
    pub fn add(&self, payload_path: &Path, check_valid: bool) -> Result<()> {
        if check_valid {
            // ensure we have been passed a valid payload
            let payload_bytes = fs::read(payload_path).wrap_err_with(|| {
                format!("Failed to read payload from: {}", &payload_path.display())
            })?;
            let _ = Payload::new(&payload_bytes)?;
        }

        let Some(payload) = payload_path.file_name() else {
            bail!("Path provided is not a file: {}", payload_path.display())
        };

        let Some(file_name) = payload.to_str() else {
            bail!("Payload is not a valid UTF-8 string")
        };

        fs::copy(payload_path, Self::directory().join(file_name))?;
        Ok(())
    }

    /// Get a Favorite, if None, did not find one
    pub fn get(&self, favorite: &str) -> Option<&Favorite> {
        self.list.iter().find(|x| x.name() == favorite.trim())
    }

    /// Returns true if it successfully removed the favorite, false otherwise
    pub fn remove(&mut self, favorite: &Favorite) -> Result<()> {
        fs::remove_file(favorite.path())?;
        self.list.retain(|x| x != favorite);
        Ok(())
    }

    /// The actual favorites directory on the file system
    pub fn directory() -> &'static Path {
        &FAVORITES_PATH
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Favorite {
    name: String,
}

impl Favorite {
    fn new(name: &str) -> Self {
        Self {
            name: name.trim().to_owned(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn read(&self) -> Result<Payload> {
        let payload_bytes = fs::read(self.path())
            .wrap_err_with(|| format!("Failed to read payload from: {}", &self.path().display()))?;
        Ok(Payload::new(&payload_bytes)?)
    }

    fn path(&self) -> PathBuf {
        Favorites::directory().to_owned().join(&self.name)
    }
}
