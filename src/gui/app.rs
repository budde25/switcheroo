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

        let Some(data) = self.selected_data.payload_data() else {
            return None;
        };

        match data {
            Ok(payload) => return Some((switch.clone(), payload)),
            Err(e) => {
                error!("payload load: {e}");
                return None;
            }
        };
    }

    fn main_tab(&mut self, ctx: &Context) {
        self.selected_data.render(ctx);

        CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.add_space(5.0);
                self.payload_window(ui);
                ui.add_space(5.0);
                ui.separator();

                // Buttons
                ui.horizontal(|ui| self.payload_buttons(ui));
            });

            ui.centered_and_justified(|ui| self.switch_image(ui));
        });
    }

    fn payload_window(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.selected_data.render_payload_name(ui);
        });
    }

    fn payload_buttons(&mut self, ui: &mut Ui) {
        if ui
            .button(RichText::new("📂").size(50.0))
            .on_hover_text("Load payload from file")
            .clicked()
        {
            if let Err(e) = self.selected_data.file_picker() {
                self.toast.error(e.to_string());
            }
        }

        if ui
            .add_enabled(
                self.selected_data.can_favorite(),
                Button::new(RichText::new("♥").size(50.0)),
            )
            .on_hover_text("Add currently loaded payload to favorites")
            .clicked()
        {
            if let Err(e) = self.selected_data.favorite() {
                self.toast.error(format!("Error adding favorite: {}", e));
            }
        }

        let executable = self.executable();
        if self.switch == SwitchData::Done {
            if ui
                .button(RichText::new("↺").size(50.0))
                .on_hover_text("Reset status")
                .clicked()
            {
                self.switch = SwitchData::None;
            }
        } else if ui
            .add_enabled(
                executable.is_some(),
                Button::new(RichText::new("🚀").size(50.0)),
            )
            .on_hover_text("Inject loaded payload")
            .clicked()
        {
            let (switch, payload) = executable.expect("device is executable");

            fn execute(mut switch: Switch, payload: &Payload, toast: &mut Toasts) {
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

            execute(switch, payload.payload(), &mut self.toast);
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Ok(switch_res) = self.recv.try_recv() {
            if self.switch != SwitchData::Done {
                match switch_res {
                    Ok(s) => self.switch = SwitchData::Available(s),
                    Err(SwitchError::SwitchNotFound) => self.switch = SwitchData::None,
                    Err(e) => {
                        self.toast.error(e.to_string());
                    }
                }
            }
        }

        self.toast.show(ctx);
        self.main_tab(ctx);
        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if let Some(last) = i.raw.dropped_files.last() {
                if let Some(path) = &last.path {
                    match PayloadData::new(Utf8Path::from_path(path).unwrap()) {
                        Ok(payload) => self.selected_data.set_payload(payload),
                        Err(e) => {
                            self.toast.error(e.to_string());
                        }
                    }
                }
            }
        });
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
