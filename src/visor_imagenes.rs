
use eframe::{egui, epi};
use egui::{TextureHandle, ColorImage};
use rfd::FileDialog;
use image::{io::Reader as ImageReader, DynamicImage};
use std::path::{PathBuf, Path};
use std::fs;

struct ImageViewerApp {
    current_image: Option<TextureHandle>,
    image_paths: Vec<PathBuf>,
    current_index: usize,
    fullscreen: bool,
    image_dir: Option<PathBuf>,
}

impl Default for ImageViewerApp {
    fn default() -> Self {
        Self {
            current_image: None,
            image_paths: vec![],
            current_index: 0,
            fullscreen: false,
            image_dir: None,
        }
    }
}

impl ImageViewerApp {
    fn load_image(&mut self, ctx: &egui::Context, path: &Path) {
        if let Ok(img) = ImageReader::open(path).and_then(|r| r.decode()) {
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
            self.load_image(ctx, &self.image_paths[self.current_index]);
            self.image_dir = Some(parent.to_path_buf());
        }
    }

    fn next_image(&mut self, ctx: &egui::Context) {
        if self.current_index + 1 < self.image_paths.len() {
            self.current_index += 1;
            self.load_image(ctx, &self.image_paths[self.current_index]);
        }
    }

    fn previous_image(&mut self, ctx: &egui::Context) {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.load_image(ctx, &self.image_paths[self.current_index]);
        }
    }
}

impl epi::App for ImageViewerApp {
    fn name(&self) -> &str {
        "Visor de Imágenes"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut epi::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.fullscreen {
            frame.set_fullscreen(false);
            self.fullscreen = false;
        }

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📂 Abrir imagen").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("Imagen", &["png", "jpg", "jpeg", "bmp"]).pick_file() {
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
                    frame.set_fullscreen(true);
                    self.fullscreen = true;
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
                ui.image(texture, image_size * scale);
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
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(
        "Visor de Imágenes",
        options,
        Box::new(|_cc| Box::new(ImageViewerApp::default())),
    )
}
