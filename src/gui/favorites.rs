use std::sync::atomic::AtomicBool;

use color_eyre::Result;
use eframe::egui::panel::Side;
use eframe::egui::{Button, Grid, RichText, SidePanel, TextStyle};
use notify::{RecursiveMode, Watcher};

use tracing::{info, warn};

use crate::favorites::Favorites;

use super::payload::PayloadData;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Selected {
    Favorited(String),
    None,
}

static CHECK_UPDATE: AtomicBool = AtomicBool::new(false);

pub struct FavoritesData {
    fav: Selected,
    favorites: Favorites,
    cache: Vec<String>,
    payload: Option<PayloadData>,
}

impl FavoritesData {
    /// Create a new FavoritesData which is a wrapper around the favorites and a cache
    pub fn new() -> Self {
        let favorites = Favorites::new()
            .expect("Failed to read favorite directory, are we missing permission?");

        let mut fav = Self {
            fav: Selected::None,
            favorites,
            cache: Vec::new(),
            payload: None,
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
                warn!("File system notifications are not available. There will be no immediate feedback to favorites directory changes.\n{:?}", e)
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

    pub fn payload(&self) -> Option<PayloadData> {
        self.payload.clone()
    }

    /// Get the favorites (from the cache) does not access the disk
    fn favorites(&self) -> &[String] {
        &self.cache
    }

    fn make_payload(&self) -> Option<PayloadData> {
        let Selected::Favorited(favorite) = &self.fav else { return None };

        let path = self.favorites.directory().join(favorite);

        let Ok(payload) = PayloadData::new(&path) else { return None };
        Some(payload)
    }

    /// Removes a payload from the favorites (then updates the cache)
    ///
    // fn remove(&mut self, file_name: &str) -> Result<bool> {
    //     let res = self.favorites.remove(file_name);
    //     self.update_cache();
    //     res
    // }

    /// Add a payload to the favorites (then updates the cache)
    pub fn add(&mut self, payload_data: &PayloadData) -> Result<()> {
        let res = self.favorites.add(payload_data.path(), true);
        self.update_cache();
        res
    }

    pub fn contains(&self, file_name: &str) -> bool {
        self.cache.iter().any(|x| x.as_str() == file_name)
    }

    pub fn render(&mut self, ctx: &eframe::egui::Context) {
        SidePanel::new(Side::Left, "Favorites").show(ctx, |ui| {
            ui.label(RichText::new("Favorites").text_style(TextStyle::Heading));
            ui.separator();

            if self.favorites().is_empty() {
                ui.label(RichText::new(
                    "You don't seem to have any favorites yet! 😢",
                ));
            } else {
                Grid::new("favorites").show(ui, |ui| {
                    let mut update = false;
                    // TODO: find a way cheaper way to iteratre
                    for entry in self.favorites().to_owned() {
                        ui.horizontal(|ui| {
                            let button = ui.selectable_value(
                                &mut self.fav,
                                Selected::Favorited(entry.clone()),
                                &entry,
                            );
                            if button.clicked() {
                                self.payload = self.make_payload();
                            }
                            ui.add_space(20.0);
                            ui.with_layout(
                                eframe::egui::Layout::right_to_left(eframe::emath::Align::Center),
                                |ui| {
                                    let remove_button = Button::new("🗑");
                                    let remove_resp = ui
                                        .add(remove_button)
                                        .on_hover_text("Remove from favorites");

                                    if remove_resp.clicked() {
                                        match self.favorites.remove(&entry) {
                                            Ok(_) => update = true,
                                            Err(e) => eprintln!("Unable to remove favorite: {e}"),
                                        };
                                    }
                                },
                            );
                        });
                        ui.end_row();
                    }
                    if update {
                        self.update(true);
                    }
                });
            }
        });
    }
}
