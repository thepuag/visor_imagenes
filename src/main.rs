use eframe::egui;
use egui::{TextureHandle, ColorImage};
use rfd::FileDialog;
use image::ImageReader; // Corrección: importar directamente ImageReader
use std::path::{PathBuf, Path};
use std::fs;

    const APP_NAME: &str = "Visor de Imágenes";
    const APP_VERSION: &str = "1.0";
    const APP_AUTHOR: &str = "Noé Montoro García";

struct ImageViewerApp {
    current_image: Option<TextureHandle>,
    image_paths: Vec<PathBuf>,
    current_index: usize,
    fullscreen: bool,
    image_dir: Option<PathBuf>,
    show_about: bool,
}

impl Default for ImageViewerApp {
    fn default() -> Self {
        Self {
            current_image: None,
            image_paths: vec![],
            current_index: 0,
            fullscreen: false,
            image_dir: None,
            show_about: false,
        }
    }
}

impl ImageViewerApp {
    fn load_image(&mut self, ctx: &egui::Context, path: &Path) {
        // Corrección: manejar correctamente los diferentes tipos de error
        match ImageReader::open(path) {
            Ok(reader) => {
                match reader.decode() {
                    Ok(img) => {
                        let size = [img.width() as usize, img.height() as usize];
                        let color_image = ColorImage::from_rgba_unmultiplied(
                            size,
                            &img.to_rgba8(),
                        );
                        self.current_image = Some(ctx.load_texture(
                            "image",
                            color_image,
                            egui::TextureOptions::default(),
                        ));
                    }
                    Err(_) => {
                        eprintln!("Error al decodificar la imagen: {:?}", path);
                    }
                }
            }
            Err(_) => {
                eprintln!("Error al abrir la imagen: {:?}", path);
            }
        }
    }

    fn load_images_from_folder(&mut self, ctx: &egui::Context, path: &Path) {
        if let Some(parent) = path.parent() {
            self.image_paths = fs::read_dir(parent)
                .unwrap()
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| {
                    if let Some(ext) = p.extension() {
                        matches!(
                            ext.to_str().unwrap_or("").to_lowercase().as_str(),
                            "jpg" | "jpeg" | "png" | "bmp"
                        )
                    } else {
                        false
                    }
                })
                .collect();
            self.image_paths.sort();
            self.current_index = self.image_paths.iter().position(|p| p == path).unwrap_or(0);
            // Corrección: separar el acceso inmutable del mutable
            if !self.image_paths.is_empty() {
                let path_to_load = self.image_paths[self.current_index].clone();
                self.load_image(ctx, &path_to_load);
            }
            self.image_dir = Some(parent.to_path_buf());
        }
    }

    fn next_image(&mut self, ctx: &egui::Context) {
        if !self.image_paths.is_empty() && self.current_index + 1 < self.image_paths.len() {
            self.current_index += 1;
            // Corrección: separar el acceso inmutable del mutable
            let path = self.image_paths[self.current_index].clone();
            self.load_image(ctx, &path);
        }
    }

    fn previous_image(&mut self, ctx: &egui::Context) {
        if self.current_index > 0 {
            self.current_index -= 1;
            // Corrección: separar el acceso inmutable del mutable
            let path = self.image_paths[self.current_index].clone();
            self.load_image(ctx, &path);
        }
    }
}

impl eframe::App for ImageViewerApp {
    // Corrección: eliminar el método name() que no existe en eframe::App

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Corrección: usar set_fullscreen en el viewport
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.fullscreen {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            self.fullscreen = false;
        }

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📂 Abrir imagen").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Imagen", &["png", "jpg", "jpeg", "bmp"])
                        .pick_file() 
                    {
                        self.load_images_from_folder(ctx, &path);
                    }
                }

                if ui.button("⬅️ Anterior").clicked() {
                    self.previous_image(ctx);
                }

                if ui.button("➡️ Siguiente").clicked() { 
                    self.next_image(ctx);
                }

                if ui.button("🖥️ Pantalla completa").clicked() {
                    // Corrección: usar ViewportCommand para pantalla completa
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    self.fullscreen = true;
                }

                if ui.button("ℹ️ Acerca de").clicked() {
                    self.show_about = true;
                }
                if self.show_about {
                    egui::Window::new("Acerca de")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ctx, |ui| {
                            ui.label(format!("{} v{}", APP_NAME, APP_VERSION));
                            ui.label(format!("Autor: {}", APP_AUTHOR));
                            ui.separator();
                            ui.label("Visor de Imágenes hecho en Rust con eframe y egui.");
                            
                            if ui.button("Cerrar").clicked() {
                                self.show_about = false;
                }
             });
}

            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.current_image {
                let available_size = ui.available_size();
                let image_size = texture.size_vec2();
                let scale = (available_size.x / image_size.x)
                    .min(available_size.y / image_size.y)
                    .min(1.0);
                
                // Corrección: usar fit_to_exact_size o un enfoque diferente para el escalado
                let scaled_size = image_size * scale;
                ui.add(
                    egui::Image::from_texture(texture)
                        .fit_to_exact_size(scaled_size)
                        .corner_radius(egui::CornerRadius::ZERO)
                );
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Abre una imagen para comenzar");
                });
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        // Corrección: eliminar drag_and_drop_support que no existe
        viewport: egui::ViewportBuilder::default()
            .with_drag_and_drop(true), // Esto es para drag and drop
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        // Corrección: devolver Ok() como espera la función
        Box::new(|_cc| Ok(Box::new(ImageViewerApp::default()))),
    )
}
