use color_eyre::eyre::{bail, Result, WrapErr};
use log::warn;
use once_cell::sync::Lazy;
use std::collections::BTreeSet;
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
    list: BTreeSet<Favorite>,
}

impl Favorites {
    /// Create a new Favorites this points to the OS's <data_dir>/switcheroo/favorites folder and creates it if it does not exist
    pub fn new() -> Self {
        let mut list = BTreeSet::new();
        for entry in fs::read_dir(Self::directory())
            .expect("Favorites directory exists")
            .flatten()
        {
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                warn!("Favorite file name is not a valid UTF-8 string: {:?} in directory {}", file_name, Self::directory().display());
                continue;
            };
            list.insert(Favorite::new(file_name));
        }

        Self { list }
    }

    /// Get an iterator to the favorites, these will be sorted by name
    pub fn iter(&self) -> impl Iterator<Item = &Favorite> {
        self.list.iter()
    }

    /// Add a payload to the favorites directory, if `check_valid` is true, we will make sure that the payload parses correctly (but slower)
    pub fn add<'a>(&mut self, payload_path: &'a Path, check_valid: bool) -> Result<&'a str> {
        if check_valid {
            // ensure we have been passed a valid payload
            let payload_bytes = fs::read(payload_path).wrap_err_with(|| {
                format!("Failed to read payload from path: {:?}", &payload_path)
            })?;
            let _ = Payload::new(&payload_bytes)?;
        }

        let Some(payload) = payload_path.file_name() else {
            bail!("Path provided is not a file: {:?}", payload_path)
        };

        let Some(file_name) = payload.to_str() else {
            bail!("file name is not a valid UTF-8 string")
        };

        fs::copy(payload_path, Self::directory().join(file_name))?;
        self.list.insert(Favorite::new(file_name));
        Ok(file_name)
    }

    /// Get a Favorite, if None, did not find one
    pub fn get(&self, favorite: &str) -> Option<&Favorite> {
        self.list.iter().find(|x| x.name() == favorite.trim())
    }

    pub fn remove(&mut self, favorite: &Favorite) -> Result<()> {
        fs::remove_file(favorite.path())?;
        self.list.retain(|x| x != favorite);
        Ok(())
    }

    /// Returns true if it successfully removed the favorite, false otherwise
    pub fn remove_str(&mut self, favorite_str: &str) -> Result<()> {
        let Some(favorite) = self.get(favorite_str) else {
            bail!("Failed to remove, favorite not found: {}", favorite_str)
        };
        self.remove(&favorite.to_owned())
    }

    /// The actual favorites directory on the file system
    pub fn directory() -> &'static Path {
        &FAVORITES_PATH
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Favorite {
    name: Box<str>,
}

impl Favorite {
    fn new(name: &str) -> Self {
        Self {
            name: name.trim().to_string().into_boxed_str(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn read(&self) -> Result<Payload> {
        let payload_bytes = fs::read(self.path())
            .wrap_err_with(|| format!("Failed to read payload from: {:?}", &self.path()))?;
        Ok(Payload::new(&payload_bytes)?)
    }

    fn path(&self) -> Box<Path> {
        Favorites::directory()
            .to_owned()
            .join(self.name.as_ref())
            .into_boxed_path()
    }
}
