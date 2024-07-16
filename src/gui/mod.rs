mod app;
mod error;
mod image;
mod payload;
mod selected;

use std::sync::mpsc::Receiver;

use crate::switch::SwitchData;

use self::error::InitError;

use eframe::egui::{style, Context, ViewportBuilder};
use egui_notify::Toasts;
use selected::SelectedData;
use tegra_rcm::{Switch, SwitchError};

const APP_NAME: &str = "Switcheroo";

pub fn gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_min_inner_size([450.0, 300.0])
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

            if let Err(e) = tegra_rcm::check_env() {
                return Ok(Box::new(InitError::new(e)));
            }

            let switch = match Switch::find() {
                Ok(a) => SwitchData::Available(a),
                Err(SwitchError::SwitchNotFound) => SwitchData::None,
                Err(e) => return Ok(Box::new(InitError::new(e))),
            };

            let recv = spawn_thread_context(cc.egui_ctx.clone());

            // We have to do it like this, we need to update the cache when loading up.
            let app = app::MyApp {
                switch,
                selected_data: SelectedData::new(),
                toast: Toasts::default(),
                recv,
            };

            Ok(Box::new(app))
        }),
    )
}

/// Spawn a separate thread
pub fn spawn_thread_context(ctx: Context) -> Receiver<Result<Switch, SwitchError>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || tegra_rcm::create_hotplug(tx, Some(move || ctx.request_repaint())));
    rx
}
