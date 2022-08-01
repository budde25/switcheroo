mod image;
mod usb;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use color_eyre::eyre::Result;

use self::image::{load_icon, Images};
use super::favorites::Favorites;
use eframe::egui::{
    style, widgets, Button, CentralPanel, Color32, Context, RichText, TopBottomPanel, Ui,
};
use native_dialog::FileDialog;
use tegra_rcm::{Error, Payload, Rcm};

type ThreadSwitchResult = Arc<Mutex<Result<Rcm, Error>>>;

pub fn gui() -> Result<()> {
    let rcm = Arc::new(Mutex::new(Rcm::new(false)));

    let images = Images::default();

    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        min_window_size: Some((400.0, 300.0).into()),
        icon_data: Some(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Switcheroo",
        options,
        Box::new(|cc| {
            let mut style = style::Style::default();
            style.visuals = style::Visuals::dark();
            cc.egui_ctx.set_style(style);

            usb::spawn_thread(rcm.clone(), cc.egui_ctx.clone());

            // We have to do it like this, we need to update the cache when loading up.
            let mut app = MyApp {
                switch: rcm,
                payload_data: None,
                images,
                state: State::NotAvailable,
                error: None,
                //tab: Tab::Main,
                favorites: Favorites::new().ok(),
                favorites_cache: vec![],
            };

            app.update_favorite_cache();

            Box::new(app)
        }),
    );
}

struct MyApp {
    switch: Arc<Mutex<Result<Rcm, Error>>>,
    payload_data: Option<PayloadData>,
    images: Images,
    state: State,
    error: Option<Error>,
    //tab: Tab,
    favorites: Option<Favorites>,
    favorites_cache: Vec<PathBuf>,
}

impl MyApp {
    // we can execute if we have a payload and rcm is available
    fn executable(&self) -> bool {
        if self.error.is_some() {
            return false;
        }

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

    // get the payload if its available
    fn payload(&self) -> Option<&Payload> {
        if let Some(payload_data) = &self.payload_data {
            if let Ok(payload) = &payload_data.payload {
                return Some(payload);
            } else {
                return None;
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
                Ok(rcm) => {
                    if let Err(e) = rcm.validate() {
                        self.error = Some(e)
                    }
                    self.state = State::Available;
                }
                Err(e) => {
                    if *e != Error::SwitchNotFound {
                        self.error = Some(e.clone())
                    }
                    self.state = State::NotAvailable;
                }
            }
        }
    }

    fn update_favorite_cache(&mut self) {
        if let Some(favorites) = &self.favorites {
            match favorites.list() {
                Ok(list) => {
                    self.favorites_cache = list.filter_map(std::result::Result::ok).map(|e| e.path()).collect();
                }
                Err(_) => eprintln!(
                    "Failed to read favorite directory, are we possibly missing permissions?"
                ),
            }
        }
    }

    fn main_tab(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.add_space(10.0);
                if let Some(payload_data) = &self.payload_data {
                    match &payload_data.payload {
                        Ok(_) => {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Payload:").size(16.0));
                                ui.monospace(
                                    RichText::new(payload_data.file_name())
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

                // If favorites could be initialized
                if self.favorites.as_ref().is_some() {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    let favorites = self.favorites_cache.clone();

                    if favorites.is_empty() {
                        ui.label(RichText::new(
                            "You don't seem to have any favorites yet! 😢",
                        ));
                    } else {
                        for entry in favorites {
                            // We should be safe to unwrap, list should only contain paths to files.
                            let file_name = entry.file_name().unwrap();

                            ui.horizontal(|ui| {
                                ui.label(&*file_name.to_string_lossy());
                                if ui
                                    .button(RichText::new("Load"))
                                    .on_hover_text("Load favorite.")
                                    .clicked()
                                {
                                    self.payload_data = PayloadData::from_path(&entry);
                                }
                                if ui
                                    .button(RichText::new("Remove"))
                                    .on_hover_text("Remove from favorites.")
                                    .clicked()
                                {
                                    self.favorites
                                        .as_ref()
                                        .unwrap()
                                        .remove(&file_name.to_string_lossy())
                                        .unwrap();
                                    self.update_favorite_cache()
                                }
                            });
                        }
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
                            self.payload_data = PayloadData::from_path(&path);
                        }
                    }

                    if let Some(favorites) = self.favorites.as_ref() {
                        let mut should_enabled = self.payload_data.is_some();

                        if let Some(payload_data) = &self.payload_data {
                            let current_loaded_name =
                                payload_data.path.file_name().unwrap().to_string_lossy();

                            let already_favorited = favorites
                                .get(&current_loaded_name)
                                .unwrap_or(None)
                                .is_some();

                            should_enabled = !already_favorited;

                            if !payload_data.path.exists() {
                                should_enabled = false;
                            }
                        }

                        if ui
                            .add_enabled(should_enabled, Button::new(RichText::new("♥").size(50.0)))
                            .on_hover_text("Add currently loaded payload to favorites")
                            .clicked()
                        {
                            if let Some(payload_data) = &self.payload_data {
                                favorites.add(&payload_data.path, true).unwrap();
                                self.update_favorite_cache();
                            }
                        }
                    }

                    if self.state == State::Done {
                        if ui
                            .button(RichText::new("↺").size(50.0))
                            .on_hover_text("Reset status")
                            .clicked()
                        {
                            self.state = State::NotAvailable
                        }
                    } else if ui
                        .add_enabled(
                            self.executable(),
                            Button::new(RichText::new("💉").size(50.0)),
                        )
                        .on_hover_text("Inject loaded payload")
                        .clicked()
                    {
                        // we are safe to unwrap because we can only get the payload if we are executable
                        let payload = self.payload().unwrap();
                        if let Ok(mut res) = self.switch.try_lock() {
                            // TODO: fix race condition
                            let rcm = &mut *res;
                            match rcm {
                                Ok(switch) => match execute(switch, payload) {
                                    Ok(_) => self.state = State::Done,
                                    Err(e) => self.error = Some(e),
                                },
                                Err(e) => self.error = Some(e.clone()),
                            }
                        }
                    }
                });
            });

            self.check_change_state();

            if let Some(ref e) = self.error {
                create_error_from_error(ui, e.clone());
            }

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
    }

    /*
    fn payload_manager(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Todo");
        });
    }
    */
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    NotAvailable,
    Available,
    Done,
}

/*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Main,
    PayloadManager
}
*/

#[derive(Debug)]
struct PayloadData {
    payload: Result<Payload, Error>,
    path: PathBuf,
}

impl PayloadData {
    /// Makes a payload from a given file path
    /// returns None on an error
    pub fn from_path(path: &Path) -> Option<Self> {
        let file = std::fs::read(&path);
        if let Ok(data) = file {
            let payload_data = PayloadData {
                path: path.to_owned(),
                payload: Payload::new(&data),
            };
            return Some(payload_data);
        }
        None
    }

    fn file_name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
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

                /*
                ui.separator();
                ui.selectable_value(&mut self.tab, Tab::Main,"Main");
                ui.selectable_value(&mut self.tab, Tab::PayloadManager,"Payload Manager");
                */
            })
        });

        /*
        match self.tab {
            Tab::Main => self.main_tab(ctx),
            Tab::PayloadManager => self.payload_manager(ctx),
        }
        */

        self.main_tab(ctx);

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            // unwrap safe cause we are not empty
            let file = ctx.input().raw.dropped_files.last().unwrap().clone();
            if let Some(path) = file.path {
                self.payload_data = PayloadData::from_path(&path);
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

fn create_error_from_error(ui: &mut Ui, error: Error) {
    match error {
        Error::SwitchNotFound => (),
        Error::AccessDenied => {
            create_error(
                ui,
                "USB permission error, see the following to troubleshoot",
            );
            ui.hyperlink("https://github.com/budde25/switcheroo#linux-permission-denied-error");
        }
        #[cfg(target_os = "windows")]
        Error::WindowsWrongDriver(i) => {
            create_error(
            ui,
            &format!(
                "Wrong USB driver installed, expected libusbK but found `{}`, see the following to troubleshoot",
                i),
            );
            ui.hyperlink("https://github.com/budde25/switcheroo#windows-wrong-driver-error");
        }
        _ => create_error(ui, &error.to_string()),
    };
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

/// Executes a payload returning any errors
fn execute(switch: &mut Rcm, payload: &Payload) -> Result<(), Error> {
    // its ok if it gets init more than once, it skips previous inits
    switch.init()?;

    // We need to read the device id first
    let _ = switch.read_device_id()?;
    switch.execute(payload)?;
    Ok(())
}
