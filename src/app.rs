
use crate::ui::viewer::ImageViewerApp;
use crate::config::config::*;

pub fn run() {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Ok(Box::new(ImageViewerApp::default()))),
    ).unwrap();
}
