use color_eyre::eyre::{bail, Result, WrapErr};
use std::fs::ReadDir;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use tegra_rcm::Payload;

/// Favorite payloads
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorites {
    path: PathBuf,
}

impl Favorites {
    /// Create a new Favorites this points to the OS's <data_dir>/switcheroo/favorites folder and creates it if it does not exist
    pub fn new() -> Result<Self> {
        let data_dir = match dirs::data_dir() {
            Some(dir) => dir,
            None => bail!("Failed to load application data_dir"),
        };

        let favorites = data_dir.join(data_dir.join("switcheroo/favorites"));

        if !favorites.is_dir() {
            fs::create_dir_all(&favorites)?;
        }

        Ok(Self { path: favorites })
    }

    /// Get an iterator of the files in the directory
    pub fn list(&self) -> Result<ReadDir> {
        let paths = fs::read_dir(&self.path)?;
        Ok(paths)
    }

    /// Add a payload to the favorites directory, if `check_valid` is true, we will make sure that the payload parses correctly (but slower)
    pub fn add(&self, payload: &Path, check_valid: bool) -> Result<()> {
        if check_valid {
            // ensure we have been passed a valid payload
            let payload_bytes = fs::read(payload)
                .wrap_err_with(|| format!("Failed to read payload from: {}", &payload.display()))?;
            let _ = Payload::new(&payload_bytes)?;
        } else if !payload.is_file() {
            bail!("Path provided is not a file: {}", payload.display())
        }

        // unwrap is safe as we checked above
        let file_name = payload.file_name().unwrap().to_string_lossy().to_string();
        fs::copy(payload, self.path.join(file_name))?;

        Ok(())
    }

    /// Get the `DirEntry` of a favorite, if None, did not find one
    pub fn get(&self, favorite: &str) -> Result<Option<DirEntry>> {
        let list = self.list()?;
        Ok(list
            .into_iter()
            .filter_map(std::result::Result::ok)
            .find(|x| x.file_name().to_string_lossy() == favorite))
    }

    /// Returns true if it successfully removed the favorite, false otherwise
    pub fn remove(&self, favorite: &str) -> Result<bool> {
        let favorite = favorite.trim(); // make sure we don't hav whitespace interfere
        let found = self.get(favorite)?;

        if let Some(file) = found {
            fs::remove_file(file.path())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// The actual favorites directory on the file system
    pub fn directory(&self) -> &Path {
        &self.path
    }
}
