use eframe::egui::Image;
use eframe::{egui, IconData};

pub struct Images<'a> {
    pub not_found: Image<'a>,
    pub connected: Image<'a>,
    pub done: Image<'a>,
}

impl<'a> Images<'a> {
    pub fn load() -> Self {
        let not_found = Image::new(egui::include_image!("images/not_found.svg"));
        let connected = Image::new(egui::include_image!("images/connected.svg"));
        let done = Image::new(egui::include_image!("images/done.svg"));
        //let not_found = Self::load_image("Rcm Not Found", include_bytes!("images/not_found.svg"));
        //let connected = Self::load_image("Rcm Connected", include_bytes!("images/connected.svg"));
        //let done = Self::load_image("Rcm Complete", include_bytes!("images/done.svg"));
        Self {
            not_found,
            connected,
            done,
        }
    }

    // fn load_image(debug_name: &'static str, image_bytes: &'static [u8]) -> RetainedImage {
    //     RetainedImage::from_svg_bytes(debug_name, image_bytes).expect("Image should be valid svg")
    // }

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
}

impl<'a> Default for Images<'a> {
    fn default() -> Self {
        Self::load()
    }
}
