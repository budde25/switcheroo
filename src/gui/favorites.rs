use std::sync::atomic::AtomicBool;

use color_eyre::Result;
use notify::{RecursiveMode, Watcher};
use tracing::{info, warn};

use crate::favorites::Favorites;

use super::payload::PayloadData;

static CHECK_UPDATE: AtomicBool = AtomicBool::new(false);

pub struct FavoritesData {
    favorites: Favorites,
    cache: Vec<String>,
}

impl FavoritesData {
    /// Create a new FavoritesData which is a wrapper around the favorites and a cache
    pub fn new() -> Self {
        let favorites = Favorites::new()
            .expect("Failed to read favorite directory, are we missing permission?");

        let mut fav = Self {
            favorites,
            cache: Vec::new(),
        };
        fav.setup_watcher();
        fav.update_cache();
        fav
    }

    fn setup_watcher(&self) {
        // Automatically select the best implementation for your platform.
        let watcher = notify::recommended_watcher(|res| match res {
            Ok(event) => {
                info!("File watcher event: {:?}", event);
                CHECK_UPDATE.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            Err(e) => warn!("File watch error: {:?}", e),
        });

        match watcher {
            Ok(mut fsw) => {
                if let Err(e) = fsw.watch(self.favorites.directory(), RecursiveMode::Recursive) {
                    warn!("File watch error: {:?}", e);
                }
            }
            Err(e) => {
                warn!("File system notifications are not availble. There will be no immediate feedback to favorites directory changes.\n{:?}", e)
            }
        }
    }

    pub fn update(&mut self, force: bool) {
        if force || CHECK_UPDATE.fetch_and(false, std::sync::atomic::Ordering::Relaxed) {
            self.update_cache();
        }
    }

    /// Grab new favorites from the the disk
    fn update_cache(&mut self) {
        match self.favorites.list() {
            Ok(list) => {
                // shouldn't be that long so a short should not be bad
                self.cache = list
                    .filter_map(std::result::Result::ok)
                    .filter_map(|e| {
                        e.path()
                            .file_name()
                            .and_then(|s| s.to_str())
                            .map(|f| f.to_owned())
                    })
                    .collect();
                self.cache.sort();
            }
            Err(_) => {
                eprintln!("Failed to read favorite directory, are we missing permissions?")
            }
        }
    }

    pub fn payload(&self, favorite: &str) -> Result<PayloadData> {
        let path = self.favorites.directory().join(favorite);
        Ok(PayloadData::new(&path)?)
    }

    /// Get the favorites (from the cache) does not access the disk
    pub fn favorites(&self) -> &[String] {
        &self.cache
    }

    /// Removes a payload from the favorites (then updates the cache)
    ///
    pub fn remove(&mut self, file_name: &str) -> Result<bool> {
        let res = self.favorites.remove(file_name);
        self.update_cache();
        res
    }

    /// Add a payload to the favorites (then updates the cache)
    pub fn add(&mut self, payload_data: &PayloadData) -> Result<()> {
        let res = self.favorites.add(&payload_data.path(), true);
        self.update_cache();
        res
    }

    pub fn contains(&self, file_name: &str) -> bool {
        self.cache
            .iter()
            .find(|x| x.as_str() == file_name)
            .is_some()
    }
}
