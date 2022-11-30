use std::sync::atomic::AtomicBool;

use color_eyre::Result;
use eframe::egui::panel::Side;
use eframe::egui::{
    global_dark_light_mode_switch, Button, Grid, Layout, RichText, SidePanel, TextStyle, Ui,
};
use eframe::emath::Align;
use eframe::epaint::Color32;
use notify::{RecursiveMode, Watcher};

use tracing::{info, warn};

use crate::favorites::{Favorite, Favorites};

use super::payload::PayloadData;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Selected {
    Favorited(Favorite),
    None,
}

static CHECK_UPDATE: AtomicBool = AtomicBool::new(false);

pub struct FavoritesData {
    selected: Selected,
    cache: Favorites,
}

impl FavoritesData {
    /// Create a new FavoritesData which is a wrapper around the favorites and a cache
    pub fn new() -> Self {
        let favorites = Favorites::new()
            .expect("Failed to read favorite directory, are we missing permission?");

        let mut fav = Self {
            selected: Selected::None,
            cache: favorites,
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
                if let Err(e) = fsw.watch(Favorites::directory(), RecursiveMode::Recursive) {
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
        let Ok(favorites) = Favorites::new() else {
            eprintln!("Failed to read favorite directory, are we missing permissions?");
            return;
        };

        self.cache = favorites;

        if let Selected::Favorited(favorite) = &self.selected {
            if !self.contains(favorite.name()) {
                self.selected = Selected::None;
            }
        }
    }

    pub fn set_selected_none(&mut self) {
        self.selected = Selected::None;
    }

    pub fn payload(&self) -> Option<PayloadData> {
        match &self.selected {
            Selected::Favorited(favorite) => Some(favorite.read_payload_data().unwrap()), // FIXME: handle error
            Selected::None => None,
        }
    }

    /// Get the favorites (from the cache) does not access the disk
    fn favorites(&self) -> &[Favorite] {
        self.cache.list()
    }

    /// Removes a payload from the favorites (then updates the cache)
    fn remove(&mut self, favorite: &Favorite) -> bool {
        let res = self.cache.remove(favorite);
        self.update_cache();
        if let Selected::Favorited(fav) = &self.selected {
            if fav == favorite {
                self.selected = Selected::None;
            }
        }
        match res {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Add a payload to the favorites (then updates the cache)
    pub fn add(&mut self, payload_data: &PayloadData) -> Result<()> {
        self.cache.add(payload_data.path(), true)?;
        self.update_cache();
        self.selected =
            Selected::Favorited(self.cache.get(payload_data.file_name()).unwrap().to_owned());
        Ok(())
    }

    pub fn contains(&self, file_name: &str) -> bool {
        self.cache.get(file_name).is_some()
    }

    pub fn render(&mut self, ctx: &eframe::egui::Context) -> (bool, bool) {
        let (mut removed, mut selected) = (false, false);
        SidePanel::new(Side::Left, "Favorites").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Favorites").text_style(TextStyle::Heading));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    global_dark_light_mode_switch(ui);
                });
            });
            ui.separator();

            if self.favorites().is_empty() {
                ui.label(RichText::new("Nothing here..."));
                return;
            }

            (removed, selected) = self.render_grid(ui);
        });
        return (removed, selected);
    }

    fn render_grid(&mut self, ui: &mut Ui) -> (bool, bool) {
        let mut selected = false;
        let mut removed = false;
        Grid::new("favorites").show(ui, |ui| {
            // TODO: find a way cheaper way to iterate
            for entry in self.favorites().to_owned() {
                ui.horizontal(|ui| {
                    match self.render_entry(&entry, ui) {
                        (rem, sel) => {
                            if rem {
                                removed = true;
                            }
                            if sel {
                                selected = true;
                            }
                        }
                    };
                });
                ui.end_row();
            }
            if removed {
                self.update(true);
                if let Selected::Favorited(favorite) = &self.selected {
                    if self.contains(favorite.name()) {
                        removed = false;
                    }
                }
            }
        });
        return (removed, selected);
    }

    fn render_entry(&mut self, entry: &Favorite, ui: &mut Ui) -> (bool, bool) {
        let mut selected = false;
        let button = ui.selectable_value(
            &mut self.selected,
            Selected::Favorited(entry.to_owned()),
            entry.name(),
        );
        if button.clicked() {
            selected = true;
        }
        ui.add_space(20.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let remove_button = Button::new("🗑");
            let remove_resp = ui.add(remove_button).on_hover_text("Remove from favorites");

            if remove_resp.clicked() {
                if self.remove(entry) {
                    return (true, selected);
                };
            }
            return (false, selected);
        })
        .inner
    }
}
