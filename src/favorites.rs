use anyhow::{bail, Result};
use camino::{Utf8Path, Utf8PathBuf};
use log::{error, warn};
use once_cell::sync::Lazy;
use std::collections::BTreeSet;
use std::fs;
use tegra_rcm::Payload;

use crate::error::AddPath;

static FAVORITES_PATH: Lazy<Utf8PathBuf> = Lazy::new(|| {
    let mut favorites_dir = dirs::data_dir().expect("System data directory exists");
    favorites_dir.push("switcheroo");
    favorites_dir.push("favorites");

    fs::create_dir_all(&favorites_dir)
        .expect("Permission to create switcheroo favorites directory");
    Utf8PathBuf::from_path_buf(favorites_dir).expect("Favorites directory is valid UTF-8")
});

/// Favorite payloads
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorites {
    list: BTreeSet<Favorite>,
}

impl Default for Favorites {
    fn default() -> Self {
        Self::new()
    }
}

impl Favorites {
    /// Create a new Favorites this points to the OS's <data_dir>/switcheroo/favorites folder and creates it if it does not exist
    pub fn new() -> Self {
        let mut list = BTreeSet::new();
        let dir = Utf8Path::read_dir_utf8(Self::directory()).expect("Favorites directory exists");
        for dir in dir
            .filter_map(|x| match x {
                Ok(dir) => Some(dir),
                Err(e) => {
                    warn!("Error parsing favorite: {}", e);
                    None
                }
            })
            .filter(|x| x.path().extension() == Some("bin"))
        {
            list.insert(Favorite::new(dir.file_name()));
        }

        Self { list }
    }

    /// Get an iterator to the favorites, these will be sorted by name
    pub fn iter(&self) -> impl Iterator<Item = &Favorite> {
        self.list.iter()
    }

    /// Add a payload to the favorites directory, if `check_valid` is true, we will make sure that the payload parses correctly (but slower)
    pub fn add(&mut self, payload_path: &Utf8Path, check_valid: bool) -> Result<Favorite> {
        if check_valid {
            // ensure we have been passed a valid payload
            let payload_bytes = fs::read(payload_path).map_err(|x| x.with_path(payload_path))?;
            let _ = Payload::new(&payload_bytes)?;
        }

        let Some(file_name) = payload_path.file_name() else {
            bail!("Path provided is not a file: {:?}", payload_path)
        };

        fs::copy(payload_path, Self::directory().join(file_name))?;
        let favorite = Favorite::new(file_name);
        self.list.insert(favorite.clone());
        Ok(favorite)
    }

    /// Get a Favorite, if None, did not find one
    pub fn get(&self, favorite: &str) -> Option<&Favorite> {
        self.list.iter().find(|x| x.name() == favorite.trim())
    }

    pub fn remove(&mut self, favorite: &Favorite) -> Result<()> {
        let utf8_path = favorite.path();
        let path = utf8_path.as_std_path();
        #[cfg(feature = "gui")]
        if let Err(e) = trash::delete(path) {
            error!("could not trash files: {}", e);
            fs::remove_file(path).map_err(|x| x.with_path(path))?;
        };
        #[cfg(not(feature = "gui"))]
        fs::remove_file(path).map_err(|x| x.with_path(path))?;
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
    pub fn directory() -> &'static Utf8Path {
        &FAVORITES_PATH
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Favorite {
    name: Box<str>,
}

impl Favorite {
    fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().trim().to_string().into_boxed_str(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn read(&self) -> Result<Payload> {
        Ok(Payload::read(&*self.path())?)
    }

    pub fn path(&self) -> Box<Utf8Path> {
        Favorites::directory()
            .to_owned()
            .join(self.name.as_ref())
            .into_boxed_path()
    }
}
