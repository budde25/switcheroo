mod error;
mod image;
mod payload;
mod selected;

use std::sync::mpsc::Receiver;

use crate::switch::SwitchData;

use self::error::gen_error;

use camino::Utf8Path;
use eframe::egui::{style, Button, CentralPanel, Color32, Context, RichText, Ui, ViewportBuilder};
use egui_notify::Toasts;
use payload::PayloadData;
use selected::SelectedData;
use tegra_rcm::{Switch, SwitchError};

const APP_NAME: &str = "Switcheroo";

pub fn gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_min_inner_size([400.0, 300.0])
            .with_drag_and_drop(true)
            .with_icon(image::load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            let mut style = style::Style::default();
            style.visuals = style::Visuals::dark();
            cc.egui_ctx.set_style(style);

            egui_extras::install_image_loaders(&cc.egui_ctx);

            // let Ok(switch) = Switch::find() else {
            //     #[cfg(target_os = "linux")]
            //     return Box::new(error::InitError::new(tegra_rcm::SwitchError::LinuxEnv));
            //     #[cfg(not(target_os = "linux"))]
            //     panic!("Failed to init SwitchData");
            // };

            let recv = spawn_thread_context(cc.egui_ctx.clone());

            // We have to do it like this, we need to update the cache when loading up.
            let app = MyApp {
                switch: SwitchData::None,
                selected_data: SelectedData::new(),
                toast: Toasts::default(),
                recv,
            };

            Box::new(app)
        }),
    )
}

struct MyApp {
    switch: SwitchData,
    selected_data: SelectedData,
    toast: Toasts,
    recv: Receiver<Result<Switch, SwitchError>>,
}

impl MyApp {
    // we can execute if we have a payload and rcm is available
    fn is_executable(&self) -> bool {
        // we can't be executable unless switch is available and we can get a payload
        matches!(self.switch, SwitchData::Available(_)) && self.selected_data.is_some()
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
                self.is_executable(),
                Button::new(RichText::new("🚀").size(50.0)),
            )
            .on_hover_text("Inject loaded payload")
            .clicked()
        {
            let payload = self.selected_data.payload_data().unwrap().unwrap();

            let payload = payload.payload();
            if let SwitchData::Available(mut switch) = self.switch.clone() {
                let handle = switch.handle().unwrap(); // TODO
                handle.execute(payload).unwrap();
            }
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
                    Err(e) => eprintln!("{e}"),
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

/// Spawn a separate thread
pub fn spawn_thread_context(ctx: Context) -> Receiver<Result<Switch, SwitchError>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || tegra_rcm::create_hotplug(tx, Some(move || ctx.request_repaint())));
    rx
}
