use eframe::egui::{include_image, IconData, Ui};

use super::app::MyApp;

impl MyApp {
    pub fn show_image(&self, ui: &mut Ui) {
        let src = match self.switch {
            crate::switch::SwitchData::None => include_image!("images/not_found.svg"),
            crate::switch::SwitchData::Available(_) => include_image!("images/connected.svg"),
            crate::switch::SwitchData::Done => include_image!("images/done.svg"),
        };
        ui.image(src);
    }
}

pub fn load_icon() -> IconData {
    const ICON: &[u8; 16_975] = include_bytes!("../../extra/logo/dev.budd.Switcheroo.png");

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(ICON)
            .expect("icon to load from memory")
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
