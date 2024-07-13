use std::sync::mpsc::Receiver;

use crate::switch::SwitchData;

use super::payload::PayloadData;
use super::selected::SelectedData;

use camino::Utf8Path;
use eframe::egui::{Button, CentralPanel, Color32, Context, RichText, Ui};
use egui_notify::Toasts;
use log::error;
use tegra_rcm::{Payload, Switch, SwitchError};

pub struct MyApp {
    pub(crate) switch: SwitchData,
    pub(crate) selected_data: SelectedData,
    pub(crate) toast: Toasts,
    pub(crate) recv: Receiver<Result<Switch, SwitchError>>,
}

impl MyApp {
    // we can execute if we have a payload and rcm is available
    fn executable(&self) -> Option<(Switch, PayloadData)> {
        // we can't be executable unless switch is available and we can get a payload
        let SwitchData::Available(ref switch) = self.switch else {
            return None;
        };

        let payload = self
            .selected_data
            .payload_data()?
            .inspect_err(|e| error!("payload load {e}"))
            .ok()?;

        Some((switch.clone(), payload))
    }

    fn main_tab(&mut self, ctx: &Context) {
        self.selected_data.render(ctx);

        CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.add_space(5.0);
                self.render_payload_window(ui);
                ui.add_space(5.0);
                ui.separator();

                // Buttons
                ui.horizontal(|ui| self.render_payload_buttons(ui));
            });

            ui.centered_and_justified(|ui| self.show_image(ui));
        });
    }

    fn render_payload_window(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.selected_data.render_payload_name(ui);
        });
    }

    fn render_payload_buttons(&mut self, ui: &mut Ui) {
        let file_picker_button = ui
            .button(RichText::new("📂").size(50.0))
            .on_hover_text("Load payload from file");

        let favorite_button = ui
            .add_enabled(
                self.selected_data.can_favorite(),
                Button::new(RichText::new("♥").size(50.0)),
            )
            .on_hover_text("Add currently loaded payload to favorites");

        if file_picker_button.clicked() {
            if let Err(e) = self.selected_data.file_picker() {
                self.toast.error(e.to_string());
            }
        }

        if favorite_button.clicked() {
            if let Err(e) = self.selected_data.favorite() {
                self.toast.error(format!("Error adding favorite: {}", e));
            }
        }

        let executable = self.executable();
        if self.switch == SwitchData::Done {
            let reset_button = ui
                .button(RichText::new("↺").size(50.0))
                .on_hover_text("Reset status");

            if reset_button.clicked() {
                self.switch = SwitchData::None;
            };
        } else {
            let execute_button = ui
                .add_enabled(
                    executable.is_some(),
                    Button::new(RichText::new("🚀").size(50.0)),
                )
                .on_hover_text("Inject loaded payload");

            if execute_button.clicked() {
                let (switch, payload) = executable.expect("device is executable");

                execute_helper(switch, payload.payload(), &mut self.toast);
            }
        }
    }

    fn file_dropper(&mut self, ctx: &Context) {
        // Collect dropped files:
        ctx.input(|i| {
            let Some(last) = i.raw.dropped_files.last() else {
                return;
            };
            let Some(path) = &last.path else { return };
            let payload = PayloadData::new(Utf8Path::from_path(path).expect("valid UTF-8"));

            match payload {
                Ok(payload) => self.selected_data.set_payload(payload),
                Err(e) => {
                    self.toast.error(e.to_string());
                }
            };
        });
    }

    /// update the switch state, without blocking thread
    fn retrieve_switch_state(&mut self) {
        if self.switch == SwitchData::Done {
            return;
        }
        let Some(switch_res) = self.recv.try_iter().last() else {
            return;
        };

        match switch_res {
            Ok(s) => self.switch = SwitchData::Available(s),
            Err(SwitchError::SwitchNotFound) => self.switch = SwitchData::None,
            Err(e) => {
                self.toast.error(e.to_string());
            }
        }
    }
}

fn execute_helper(mut switch: Switch, payload: &Payload, toast: &mut Toasts) {
    let handle = match switch.handle() {
        Ok(handle) => handle,
        Err(e) => {
            toast.error(e.to_string());
            return;
        }
    };

    if let Err(e) = handle.execute(payload) {
        toast.error(e.to_string());
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &Context) {
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.retrieve_switch_state();
        self.toast.show(ctx);
        self.main_tab(ctx);
        preview_files_being_dropped(ctx);

        self.file_dropper(ctx)
    }
}
