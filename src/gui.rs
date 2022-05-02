#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use color_eyre::eyre::Result;
use eframe;

use rcm_lib::{Error, Payload, Rcm};

pub fn gui() -> Result<()> {
    let rcm = Rcm::new(false);
    let arc = Arc::new(Mutex::new(rcm));
    let arc2 = arc.clone();

    thread::spawn(move || loop {
        {
            let lock = arc.lock();
            if let Ok(mut inner) = lock {
                let new = Rcm::new(false);
                *inner = new;
            }
        }
        thread::sleep(Duration::from_secs(1));
    });

    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };

    eframe::run_native(
        "Switcharoo",
        options,
        Box::new(|_cc| {
            Box::new(MyApp {
                switch: arc2,
                dropped_files: Vec::default(),
                payload_data: None,
                executable: false,
                available: false,
            })
        }),
    );
}

struct MyApp {
    switch: Arc<Mutex<Result<Rcm, Error>>>,
    dropped_files: Vec<egui::DroppedFile>,
    payload_data: Option<PayloadData>,
    executable: bool,
    available: bool,
}

struct PayloadData {
    payload: Result<Payload, Error>,
    picked_path: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Select Payload").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let file = std::fs::read(&path);
                    if let Ok(data) = file {
                        self.payload_data = Some(PayloadData {
                            picked_path: path.display().to_string(),
                            payload: Payload::new(&data),
                        });
                    } else {
                        // TODO: handle error
                    }
                }
            }

            ui.group(|ui| {
                if let Some(payload_data) = &self.payload_data {
                    ui.horizontal(|ui| {
                        if let Ok(_) = &payload_data.payload {
                            ui.label("Payload:");
                            ui.monospace(&payload_data.picked_path);
                        } else {
                            ui.label("Error:");
                            let error = &payload_data.payload.as_ref().unwrap_err().to_string();
                            ui.monospace(error);
                        }
                    });
                    if self.available {
                        self.executable = true;
                    }
                } else {
                    self.executable = false;
                }

                ui.separator();

                // A greyed-out and non-interactive button:
                if ui
                    .add_enabled(self.executable, egui::Button::new("Execute"))
                    .clicked()
                {
                    let payload = self
                        .payload_data
                        .as_ref()
                        .unwrap()
                        .payload
                        .as_ref()
                        .unwrap();
                    if let Ok(mut res) = self.switch.try_lock() {
                        // TODO: fix race condition
                        if let Ok(switch) = &mut *res {
                            match execute(switch, payload) {
                                Ok(_) => (),
                                Err(e) => {
                                    ui.horizontal(|ui| {
                                        ui.label("Error");
                                        ui.monospace(e.to_string());
                                    });
                                }
                            }
                            switch.init().unwrap();
                            let _ = switch.read_device_id().unwrap();
                            switch.execute(payload).unwrap();
                            self.available = false;
                        }
                    }
                }
            });

            if self.available {
                ui.label("Switch is plugged in and available");
            } else {
                ui.label("Switch is unavailable");
            }

            let arc = self.switch.try_lock();
            if let Ok(lock) = arc {
                let res = &*lock;
                match res {
                    Ok(_) => self.available = true,
                    Err(_) => {
                        self.available = false;
                    }
                }
            }

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };
                        if let Some(bytes) = &file.bytes {
                            info += &format!(" ({} bytes)", bytes.len());
                        }
                        ui.label(info);
                    }
                });
            }
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;

    if !ctx.input().raw.hovered_files.is_empty() {
        let mut text = "Dropping files:\n".to_owned();
        for file in &ctx.input().raw.hovered_files {
            if let Some(path) = &file.path {
                text += &format!("\n{}", path.display());
            } else if !file.mime.is_empty() {
                text += &format!("\n{}", file.mime);
            } else {
                text += "\n???";
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

fn execute(switch: &mut Rcm, payload: &Payload) -> Result<(), Error> {
    switch.init()?;
    println!("Smashing the stack!");

    // We need to read the device id first
    let _ = switch.read_device_id()?;
    switch.execute(&payload)?;
    Ok(())
}
