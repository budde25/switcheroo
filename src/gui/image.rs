use egui_extras::RetainedImage;
use std::thread::{self, JoinHandle};

pub struct Images {
    pub not_found: RetainedImage,
    pub connected: RetainedImage,
    pub done: RetainedImage,
}

impl Default for Images {
    fn default() -> Self {
        // some performance testing showed that this does indeed have a up to almost 3x speedup
        // TODO: still feels like this could be faster
        let not_found_handle = load_image("Rcm Not Found", include_bytes!("images/not_found.svg"));
        let connected_handle = load_image("Rcm Connected", include_bytes!("images/connected.svg"));
        let done_handle = load_image("Rcm Complete", include_bytes!("images/done.svg"));

        let not_found = not_found_handle
            .join()
            .expect("Thread should be able to join");
        let connected = connected_handle
            .join()
            .expect("Thread should be able to join");
        let done = done_handle.join().expect("Thread should be able to join");

        Self {
            not_found,
            connected,
            done,
        }
    }
}

fn load_image(debug_name: &'static str, image_bytes: &'static [u8]) -> JoinHandle<RetainedImage> {
    thread::spawn(move || {
        RetainedImage::from_svg_bytes(debug_name, image_bytes).expect("Image should be valid svg")
    })
}
