use crate::ui::viewer::ImageViewerApp;
use crate::config::config::*;

pub fn run() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    let result = eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            let mut app = ImageViewerApp::default();
            app.init(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );

    // Manejo de errores silencioso en release
    if let Err(_e) = result {
        #[cfg(debug_assertions)]
        eprintln!("Error al iniciar la aplicación: {}", _e);
        
        // En release, salimos silenciosamente
        #[cfg(not(debug_assertions))]
        std::process::exit(1);
    }
}