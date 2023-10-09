use anyhow::Result;
use eframe::egui::panel::Side;
use eframe::egui::{
    global_dark_light_mode_switch, Button, Grid, Layout, RichText, SidePanel, TextStyle, Ui,
};
use eframe::emath::Align;

use crate::favorites::{Favorite, Favorites};

use super::payload::PayloadData;

#[derive(Debug, Default, PartialEq)]
pub enum Selected {
    #[default]
    None,
    Payload(PayloadData),
    Favorite(Favorite),
}

impl Selected {
    fn payload_data(&self) -> Option<Result<PayloadData>> {
        match self {
            Selected::None => None,
            Selected::Payload(p) => Some(Ok(p.clone())),
            Selected::Favorite(f) => Some(f.read_payload_data().map_err(|x| x.into())),
        }
    }
}

#[derive(Debug)]
pub struct SelectedData {
    selected: Selected,
    cache: Favorites,
}

impl SelectedData {
    /// Create a new `FavoritesData` which is a wrapper around the favorites and a cache
    pub fn new() -> Self {
        Self {
            selected: Selected::None,
            cache: Favorites::new(),
        }
    }

    /// Grab new favorites from the the disk
    fn update(&mut self) {
        self.cache = Favorites::new();

        if let Selected::Favorite(favorite) = &self.selected {
            if !self.contains(favorite.name()) {
                self.selected = Selected::None;
            }
        }
    }

    pub fn payload_data(&self) -> Option<Result<PayloadData>> {
        self.selected.payload_data()
    }

    /// Get the favorites (from the cache) does not access the disk
    fn favorites(&self) -> impl Iterator<Item = &Favorite> {
        self.cache.iter()
    }

    /// Removes a payload from the favorites (then updates the cache)
    fn remove(&mut self, favorite: &Favorite) -> bool {
        let res = self.cache.remove(favorite);
        self.update();
        res.is_ok()
    }

    /// Add a payload to the favorites (then updates the cache)
    pub fn add(&mut self, payload_data: &PayloadData) -> Result<()> {
        let favorite = self.cache.add(payload_data.path(), true)?.clone();
        self.update();
        self.selected = Selected::Favorite(favorite);
        Ok(())
    }

    pub fn contains(&self, file_name: &str) -> bool {
        self.cache.get(file_name).is_some()
    }

    pub fn is_some(&self) -> bool {
        self.selected != Selected::None
    }

    pub fn can_favorite(&self) -> bool {
        if let Selected::Payload(p) = &self.selected {
            return self.cache.get(p.file_name()).is_none();
        }
        false
    }

    pub fn favorite(&mut self) -> Result<bool> {
        let Selected::Payload(payload) = std::mem::take(&mut self.selected) else {
            return Ok(false);
        };
        self.add(&payload)?;
        Ok(true)
    }

    // TODO: put whole picker in here?
    pub fn set_payload(&mut self, payload: PayloadData) {
        self.selected = Selected::Payload(payload);
    }

    pub fn render(&mut self, ctx: &eframe::egui::Context) {
        SidePanel::new(Side::Left, "Favorites").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Favorites").text_style(TextStyle::Heading));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    global_dark_light_mode_switch(ui);
                });
            });
            ui.separator();

            if self.favorites().count() == 0 {
                ui.label(RichText::new("Nothing here..."));
                return;
            }

            self.render_grid(ui);
        });
    }

    fn render_grid(&mut self, ui: &mut Ui) {
        Grid::new("favorites").show(ui, |ui| {
            // TODO: find a way cheaper way to iterate
            // TODO: Remove once false positive is resolved
            for entry in self.favorites().cloned().collect::<Vec<_>>() {
                ui.horizontal(|ui| {
                    self.render_entry(&entry, ui);
                });
                ui.end_row();
            }
        });
    }

    fn render_entry(&mut self, entry: &Favorite, ui: &mut Ui) {
        ui.selectable_value(
            &mut self.selected,
            Selected::Favorite(entry.clone()),
            entry.name(),
        );
        ui.add_space(36.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let remove_button = Button::new("🗑");
            let remove_resp = ui.add(remove_button).on_hover_text("Remove from favorites");

            if remove_resp.clicked() {
                self.remove(entry);
            }
        });
    }
}
