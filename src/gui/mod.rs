#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod image;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use color_eyre::eyre::Result;

use egui::{Button, Color32, Context, RichText, Ui};
use image::Images;
use tegra_rcm::{Error, Payload, Rcm};

type ThreadSwitchResult = Arc<Mutex<Result<Rcm, Error>>>;

pub fn gui() -> Result<()> {
    let rcm = Arc::new(Mutex::new(Rcm::new(false)));

    let images = Images::default();

    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        icon_data: None,
        ..Default::default()
    };

    eframe::run_native(
        "Switcheroo",
        options,
        Box::new(|cc| {
            spawn_rcm_check_thread(rcm.clone(), cc.egui_ctx.clone(), Duration::from_secs(1));

            Box::new(MyApp {
                switch: rcm,
                payload_data: None,
                images,
                state: State::NotAvailable,
            })
        }),
    );
}

struct MyApp {
    switch: Arc<Mutex<Result<Rcm, Error>>>,
    payload_data: Option<PayloadData>,
    images: Images,
    state: State,
}

impl MyApp {
    // we can execute if we have a payload and rcm is available
    fn executable(&self) -> bool {
        // we can't be excutable in this state
        match self.state {
            State::NotAvailable => return false,
            State::Available => (),
            State::Done => return false,
        };

        // if we have a payload
        if let Some(payload_data) = &self.payload_data {
            return payload_data.payload.is_ok();
        };
        false
    }

    // get the payload its available
    fn payload(&self) -> Option<&Payload> {
        if let Some(payload_data) = &self.payload_data {
            if let Ok(payload) = &payload_data.payload {
                Some(payload)
            } else {
                None
            };
        };
        None
    }

    /// Check if we need to change our current state
    fn check_change_state(&mut self) {
        if self.state == State::Done {
            return;
        }

        let arc = self.switch.try_lock();
        if let Ok(lock) = arc {
            let res = &*lock;
            match res {
                Ok(_) => self.state = State::Available,
                Err(_) => self.state = State::NotAvailable,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    NotAvailable,
    Available,
    Done,
}

#[derive(Debug)]
struct PayloadData {
    payload: Result<Payload, Error>,
    picked_path: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title
            ui.label(RichText::new("Switcheroo").size(30.0).strong());
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.add_space(10.0);
                if let Some(payload_data) = &self.payload_data {
                    match payload_data.payload {
                        Ok(_) => {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Payload:").size(16.0));
                                ui.monospace(
                                    RichText::new(&payload_data.picked_path)
                                        .color(Color32::BLUE)
                                        .size(16.0),
                                );
                            });
                        }
                        Err(e) => create_error(ui, &e.to_string()),
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Payload:").size(16.0));
                        ui.monospace(RichText::new("None").size(16.0));
                    });
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new("Select Payload").size(18.0))
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.payload_data = make_payload_data(&path);
                            }
                        }

                        if ui
                            .add_enabled(
                                self.executable(),
                                Button::new(RichText::new("Execute").size(18.0)),
                            )
                            .clicked()
                        {
                            // we are safe to unwrap because we can only get the payload if we are executable
                            let payload = self.payload().unwrap();
                            if let Ok(mut res) = self.switch.try_lock() {
                                // TODO: fix race condition
                                if let Ok(switch) = &mut *res {
                                    match execute(switch, payload) {
                                        Ok(_) => self.state = State::Done,
                                        Err(e) => create_error(ui, &e.to_string()),
                                    }
                                }
                            }
                        }
                    });
                });
            });

            ui.centered_and_justified(|ui| {
                match self.state {
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

        self.check_change_state();
        preview_files_being_dropped(ctx);

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            // unwrap safe cause we are not empty
            let file = ctx.input().raw.dropped_files.last().unwrap().clone();
            if let Some(path) = file.path {
                self.payload_data = make_payload_data(&path);
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

/// Preview hovering files
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;

    if !ctx.input().raw.hovered_files.is_empty() {
        let mut text = "Dropping payload:\n".to_owned();
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

/// Executes a payload returning any errors
fn execute(switch: &mut Rcm, payload: &Payload) -> Result<(), Error> {
    // its ok if it gets init more than once, it skips previous inits
    switch.init()?;
    println!("Smashing the stack!");

    // We need to read the device id first
    let _ = switch.read_device_id()?;
    switch.execute(payload)?;
    Ok(())
}

/// Makes a payload from a given file path
/// returns None on an error
fn make_payload_data(path: &Path) -> Option<PayloadData> {
    let file = std::fs::read(&path);
    if let Ok(data) = file {
        let payload_data = PayloadData {
            picked_path: path.display().to_string(),
            payload: Payload::new(&data),
        };
        return Some(payload_data);
    }
    None
}

/// Creates a backgroud thread that reports if there is a switch in RCM mode detected
fn spawn_rcm_check_thread(tswitch: ThreadSwitchResult, ctx: Context, refresh: Duration) {
    thread::spawn(move || loop {
        {
            let lock = tswitch.lock();
            if let Ok(mut inner) = lock {
                let new = Rcm::new(false);
                *inner = new;
                ctx.request_repaint();
            }
        }
        thread::sleep(refresh);
    });
}
