use eframe::egui::Ui;
use eframe::{egui, IconData};

use super::MyApp;

impl MyApp {
    pub fn switch_image(&self, ui: &mut Ui) {
        let src = match self.switch_data.state() {
            crate::switch::State::NotAvailable => egui::include_image!("images/not_found.svg"),
            crate::switch::State::Available => egui::include_image!("images/connected.svg"),
            crate::switch::State::Done => egui::include_image!("images/done.svg"),
        };
        ui.image(src);
    }
}

pub fn load_icon() -> IconData {
    const ICON: &[u8; 16_975] = include_bytes!("../../extra/logo/io.ebudd.Switcheroo.png");

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(ICON)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}
