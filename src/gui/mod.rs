mod favorites;
mod image;
mod payload;
mod switch;
mod usb;

use self::image::Images;
use eframe::egui::{
    style, widgets, Button, CentralPanel, Color32, Context, RichText, TopBottomPanel, Ui,
};
use favorites::FavoritesData;
use native_dialog::FileDialog;
use payload::PayloadData;
use switch::{State, Switch, SwitchData};

pub fn gui() {
    let switch_data = SwitchData::new();

    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        min_window_size: Some((400.0, 300.0).into()),
        icon_data: Some(image::load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Switcheroo",
        options,
        Box::new(|cc| {
            let mut style = style::Style::default();
            style.visuals = style::Visuals::dark();
            cc.egui_ctx.set_style(style);

            usb::spawn_thread(switch_data.switch(), cc.egui_ctx.clone());

            // We have to do it like this, we need to update the cache when loading up.
            let app = MyApp {
                switch_data,
                payload_data: None,
                images: Images::default(),
                error: None,
                favorites_data: FavoritesData::new(),
            };

            Box::new(app)
        }),
    );
}

struct MyApp {
    switch_data: SwitchData,
    payload_data: Option<PayloadData>,
    favorites_data: FavoritesData,
    images: Images,
    error: Option<tegra_rcm::Error>,
}

impl MyApp {
    // we can execute if we have a payload and rcm is available
    fn is_executable(&self) -> bool {
        if self.error.is_some() {
            return false;
        }

        // we can't be executable in this state
        if self.switch_data.state() != State::Available {
            return false;
        }

        // Finally do we even have a payload
        self.payload_data.is_some()
    }

    fn allow_favoriting(&self) -> bool {
        let mut should_enabled = self.payload_data.is_some();
        if let Some(payload_data) = &self.payload_data {
            let current_loaded_name = payload_data.file_name();

            let already_favorited = self.favorites_data.contains(current_loaded_name);

            should_enabled = !already_favorited;

            if !payload_data.path().exists() {
                should_enabled = false;
            }
        }

        should_enabled
    }

    fn main_tab(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Payload:").size(16.0));
                    if let Some(payload) = &self.payload_data {
                        ui.monospace(
                            RichText::new(payload.file_name())
                                .color(Color32::LIGHT_BLUE)
                                .size(16.0),
                        );
                    } else {
                        ui.monospace(RichText::new("None").size(16.0));
                    }
                });

                // Favorites
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                if self.favorites_data.favorites().is_empty() {
                    ui.label(RichText::new(
                        "You don't seem to have any favorites yet! 😢",
                    ));
                } else {
                    // TODO: find a way cheaper way to iteratre
                    let favorites = self.favorites_data.favorites().to_owned();
                    for entry in favorites {
                        ui.horizontal(|ui| {
                            ui.label(&entry);
                            if ui
                                .button(RichText::new("Load"))
                                .on_hover_text("Load favorite.")
                                .clicked()
                            {
                                match self.favorites_data.payload(&entry) {
                                    Ok(payload) => self.payload_data = Some(payload),
                                    Err(e) => eprintln!("{e}"),
                                };
                            }
                            ui.spacing();
                            if ui
                                .button(RichText::new("Remove"))
                                .on_hover_text("Remove from favorites.")
                                .clicked()
                            {
                                match self.favorites_data.remove(&entry) {
                                    Ok(_) => (),
                                    Err(e) => eprintln!("Unable to remove favorite: {e}"),
                                };
                            }
                        });
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new("📂").size(50.0))
                        .on_hover_text("Load payload from file")
                        .clicked()
                    {
                        if let Some(path) = FileDialog::new().show_open_single_file().unwrap() {
                            match PayloadData::new(&path) {
                                Ok(payload) => self.payload_data = Some(payload),
                                Err(e) => eprintln!("{e}"),
                            }
                        }
                    }

                    if ui
                        .add_enabled(
                            self.allow_favoriting(),
                            Button::new(RichText::new("♥").size(50.0)),
                        )
                        .on_hover_text("Add currently loaded payload to favorites")
                        .clicked()
                    {
                        if let Some(payload_data) = &self.payload_data {
                            self.favorites_data.add(payload_data).unwrap();
                        }
                    }

                    if self.switch_data.state() == State::Done {
                        if ui
                            .button(RichText::new("↺").size(50.0))
                            .on_hover_text("Reset status")
                            .clicked()
                        {
                            self.switch_data.reset_state();
                        }
                    } else if ui
                        .add_enabled(
                            self.is_executable(),
                            Button::new(RichText::new("💉").size(50.0)),
                        )
                        .on_hover_text("Inject loaded payload")
                        .clicked()
                    {
                        let payload = self
                            .payload_data
                            .as_ref()
                            .expect("Is executable, therefore payload must exist")
                            .payload();
                        if let Err(e) = self.switch_data.execute(payload) {
                            self.error = Some(e)
                        }
                    }
                });
            });

            if let Err(e) = self.switch_data.update_state() {
                self.error = Some(e);
            }

            // check for changes
            self.favorites_data.update(false);

            if let Some(e) = &self.error {
                create_error_from_error(ui, e);
            }

            ui.centered_and_justified(|ui| {
                match self.switch_data.state() {
                    State::Available => {
                        self.images.connected.show_max_size(ui, ui.available_size())
                    }
                    State::NotAvailable => {
                        self.images.not_found.show_max_size(ui, ui.available_size())
                    }
                    State::Done => self.images.done.show_max_size(ui, ui.available_size()),
                };
            });
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Title
                ui.label(RichText::new("Switcheroo").size(24.0).strong());

                ui.separator();
                widgets::global_dark_light_mode_switch(ui);
            })
        });

        self.main_tab(ctx);

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            // unwrap safe cause we are not empty
            let file = ctx.input().raw.dropped_files.last().unwrap().clone();
            if let Some(path) = file.path {
                match PayloadData::new(&path) {
                    Ok(payload) => self.payload_data = Some(payload),
                    Err(e) => eprintln!("{e}"), // TODO:
                }
            }
        }
    }
}

/// Creates a basic error string
fn create_error(ui: &mut Ui, error: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Error:").color(Color32::RED).size(18.0));
        ui.monospace(RichText::new(error).color(Color32::RED).size(18.0));
    });
}

fn create_error_from_error(ui: &mut Ui, error: &tegra_rcm::Error) {
    // if let Some(err) = error.downcast_ref() {
    match error {
        tegra_rcm::Error::SwitchNotFound => (),
        tegra_rcm::Error::AccessDenied => {
            create_error(
                ui,
                "USB permission error, see the following to troubleshoot",
            );
            ui.hyperlink("https://github.com/budde25/switcheroo#linux-permission-denied-error");
        }
        #[cfg(target_os = "windows")]
        tegra_rcm::Error::WindowsWrongDriver(i) => {
            create_error(
            ui,
            &format!(
                "Wrong USB driver installed, expected libusbK but found `{}`, see the following to troubleshoot",
                i),
            );
            ui.hyperlink("https://github.com/budde25/switcheroo#windows-wrong-driver-error");
        }
        e => create_error(ui, &e.to_string()),
    };
    //} else {
    create_error(ui, &error.to_string())
    //}
}

/// Preview hovering files
fn preview_files_being_dropped(ctx: &Context) {
    use eframe::egui::{Align2, Id, LayerId, Order, TextStyle};

    if !ctx.input().raw.hovered_files.is_empty() {
        let mut text = "Dropping payload:\n\n".to_owned();
        for file in &ctx.input().raw.hovered_files {
            if let Some(path) = &file.path {
                text += &path.as_os_str().to_string_lossy();
            } else if !file.mime.is_empty() {
                text += &file.mime;
            } else {
                text += "???";
            }
        }

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.input().screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}
