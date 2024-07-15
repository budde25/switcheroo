use anyhow::Result;
use camino::Utf8PathBuf;
use eframe::egui::{
    global_dark_light_mode_switch, Button, Context, Layout, RichText, ScrollArea, SidePanel,
    TextStyle, Ui, Visuals,
};
use eframe::emath::Align;
use eframe::epaint::Color32;
use rfd::FileDialog;

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
    fn rich_text(&self, ctx: &Context) -> RichText {
        let text = match self {
            Selected::None => return RichText::new("None").size(16.0),
            Selected::Payload(p) => RichText::new(p.file_stem()),
            Selected::Favorite(f) => RichText::new(f.stem()),
        }
        .size(16.0);

        if ctx.style().visuals == Visuals::dark() {
            text.color(Color32::LIGHT_BLUE)
        } else {
            text.color(Color32::DARK_BLUE)
        }
    }
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

#[derive(Debug, Default)]
pub struct SelectedData {
    selected: Selected,
    cache: Favorites,
}

impl SelectedData {
    /// Create a new `FavoritesData` which is a wrapper around the favorites and a cache
    pub fn new() -> Self {
        Self::default()
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

    /// Is there selected data
    pub fn is_some(&self) -> bool {
        self.selected != Selected::None
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

    pub fn file_picker(&mut self) -> Result<bool> {
        let file_picker = FileDialog::new().add_filter("binary", &["bin"]);

        let Some(file) = file_picker.pick_file() else {
            return Ok(false);
        };

        let path = Utf8PathBuf::from_path_buf(file).unwrap();
        let payload = PayloadData::new(path)?;

        self.set_payload(payload);

        Ok(true)
    }

    pub fn render_payload_name(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Payload:").size(16.0));
        ui.monospace(self.selected.rich_text(ui.ctx()));
    }

    pub fn render(&mut self, ctx: &eframe::egui::Context) {
        SidePanel::left("favorites")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Favorites").text_style(TextStyle::Heading));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        global_dark_light_mode_switch(ui);
                        let refresh_button =
                            ui.button("🔄").on_hover_text("Refresh favorites list");
                        if refresh_button.clicked() {
                            self.update()
                        }
                    });
                });
                ui.separator();

                if self.favorites().count() == 0 {
                    ui.label(RichText::new("No favorites"));
                    return;
                }

                self.render_grid(ui);
            });
    }

    fn render_grid(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            // TODO: find a way cheaper way to iterate
            // TODO: Remove once false positive is resolved
            for entry in self.favorites().cloned().collect::<Vec<_>>() {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.selected,
                        Selected::Favorite(entry.clone()),
                        entry.stem(),
                    );
                    ui.add_space(16.0);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let remove_button = Button::new("🗑");
                        if ui
                            .add(remove_button)
                            .on_hover_text("Remove from favorites")
                            .clicked()
                        {
                            self.remove(&entry);
                        }
                    });
                });
                ui.end_row();
            }
        });
    }
}
