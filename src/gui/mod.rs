mod favorites;
mod image;
mod payload;
mod switch;
mod usb;

use std::rc::Rc;

use self::image::Images;
use eframe::egui::{
    style, Button, CentralPanel, Color32, Context, Direction, Layout, RichText, Ui,
};
use egui_notify::Toasts;
use favorites::FavoritesData;
use payload::PayloadData;
use rfd::FileDialog;
use switch::{State, SwitchData, SwitchDevice};

pub fn gui() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        min_window_size: Some((500.0, 300.0).into()),
        icon_data: Some(Images::load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Switcheroo",
        options,
        Box::new(|cc| {
            let mut style = style::Style::default();
            style.visuals = style::Visuals::dark();
            cc.egui_ctx.set_style(style);

            let Ok(switch_data) = SwitchData::new() else {
                let app = InitError {
                    error: gen_error(&tegra_rcm::SwitchError::LinuxEnv).unwrap(),
                };
                return Box::new(app);
            };

            usb::spawn_thread(switch_data.switch(), cc.egui_ctx.clone());

            // We have to do it like this, we need to update the cache when loading up.
            let app = MyApp {
                switch_data,
                payload_data: None,
                images: Images::load(),
                favorites_data: FavoritesData::new(),
                toast: Toasts::default(),
            };

            Box::new(app)
        }),
    )
    .expect("Window is able to run");
}

struct InitError {
    error: String,
}

impl eframe::App for InitError {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    ui.heading(&self.error);
                    ui.label("Unrecoverable error, please correct this error and relaunch the app");
                })
            });
        });
    }
}

struct MyApp {
    switch_data: SwitchData,
    payload_data: Option<Rc<PayloadData>>,
    favorites_data: FavoritesData,
    images: Images,
    toast: Toasts,
}

impl MyApp {
    // we can execute if we have a payload and rcm is available
    fn is_executable(&self) -> bool {
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
        let (removed, clicked) = self.favorites_data.render(ctx);
        if removed {
            self.payload_data = None;
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.add_space(5.0);
                self.payload_window(ui);
                ui.add_space(5.0);
                ui.separator();

                // Buttons
                ui.horizontal(|ui| self.payload_buttons(ui));
            });

            self.switch_data.update_state();

            if let Some(p) = self.favorites_data.payload() {
                if clicked {
                    self.payload_data = Some(p);
                };
            }

            // check for changes
            self.favorites_data.update(false);

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

    fn payload_window(&mut self, ui: &mut Ui) {
        self.render_payload(ui);
    }

    pub fn render_payload(&mut self, ui: &mut Ui) {
        let mut payload = self.favorites_data.payload();
        if payload.is_none() {
            payload = self.payload_data.clone();
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new("Payload:").size(16.0));

            if let Some(selected) = payload {
                ui.monospace(
                    RichText::new(selected.file_name())
                        .color(Color32::LIGHT_BLUE)
                        .size(16.0),
                );
            } else {
                ui.monospace(RichText::new("None").size(16.0));
            }
        });
    }

    fn payload_buttons(&mut self, ui: &mut Ui) {
        if ui
            .button(RichText::new("📂").size(50.0))
            .on_hover_text("Load payload from file")
            .clicked()
        {
            if let Some(file) = FileDialog::new().add_filter("binary", &["bin"]).pick_file() {
                match PayloadData::new(&file) {
                    Ok(payload) => {
                        self.payload_data = Some(Rc::new(payload));
                        self.favorites_data.set_selected_none();
                    }
                    Err(e) => {
                        self.toast.error(e.to_string());
                    }
                }
            } else {
                eprintln!("File Dialog Error");
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
                Button::new(RichText::new("🚀").size(50.0)),
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
                if let Some(err) = gen_error(&e) {
                    self.toast.error(err);
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.toast.show(ctx);
        self.main_tab(ctx);
        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if let Some(last) = i.raw.dropped_files.last() {
                if let Some(path) = &last.path {
                    match PayloadData::new(path) {
                        Ok(payload) => self.payload_data = Some(Rc::new(payload)),
                        Err(e) => {
                            self.toast.error(e.to_string());
                        }
                    }
                }
            }
        });
    }
}

fn gen_error(error: &tegra_rcm::SwitchError) -> Option<String> {
    match error {
        tegra_rcm::SwitchError::SwitchNotFound => None,
        tegra_rcm::SwitchError::AccessDenied => {
            let link = "https://github.com/budde25/switcheroo#linux-permission-denied-error";
            Some(format!(
                "USB permission error, see the following to troubleshoot\n{link}"
            ))
        }
        tegra_rcm::SwitchError::WindowsWrongDriver(i) => {
            let link = "https://github.com/budde25/switcheroo#windows-wrong-driver-error";
            Some(format!(
                "Wrong USB driver installed, expected libusbK but found `{i}`, see the following to troubleshoot\n{link}"
            ))
        }
        e => Some(e.to_string()),
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &eframe::egui::Context) {
    use eframe::egui::{Align2, Id, LayerId, Order, TextStyle};
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
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
