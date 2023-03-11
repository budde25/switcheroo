use eframe::IconData;
use egui_extras::RetainedImage;

pub struct Images {
    pub not_found: RetainedImage,
    pub connected: RetainedImage,
    pub done: RetainedImage,
}

impl Images {
    pub fn load() -> Self {
        let not_found = Self::load_image("Rcm Not Found", include_bytes!("images/not_found.svg"));
        let connected = Self::load_image("Rcm Connected", include_bytes!("images/connected.svg"));
        let done = Self::load_image("Rcm Complete", include_bytes!("images/done.svg"));
        Self {
            not_found,
            connected,
            done,
        }
    }

    fn load_image(debug_name: &'static str, image_bytes: &'static [u8]) -> RetainedImage {
        RetainedImage::from_svg_bytes(debug_name, image_bytes).expect("Image should be valid svg")
    }

    pub fn load_icon() -> IconData {
        const ICON: &[u8; 15567] = include_bytes!("../../extra/logo/io.ebudd.Switcheroo.png");

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

impl Default for Images {
    fn default() -> Self {
        Self::load()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_images() {
        fn load(debug_name: &'static str, image_bytes: &'static [u8]) -> RetainedImage {
            RetainedImage::from_svg_bytes(debug_name, image_bytes)
                .expect("Image should be valid svg")
        }

        let not_found = load("Rcm Not Found", include_bytes!("images/not_found.svg"));
        let connected = load("Rcm Connected", include_bytes!("images/connected.svg"));
        let done = load("Rcm Complete", include_bytes!("images/done.svg"));

        assert!(not_found.debug_name() == "Rcm Not Found");
        assert!(connected.debug_name() == "Rcm Connected");
        assert!(done.debug_name() == "Rcm Complete");
    }
}
