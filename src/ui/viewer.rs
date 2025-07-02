use egui::{TextureHandle, ColorImage};
use std::path::{PathBuf, Path};
use std::sync::{Arc, Mutex, mpsc};
use crate::core::image_cache::ImageCache;
use crate::core::image_loader_factory::load_image_optimized;
use crate::i18n::es::*;
use crate::config::config::*;
use eframe::egui;
use std::fs;
use std::thread;
use rayon::prelude::*;
use rfd::FileDialog;
// Define la estructura de la aplicación

pub struct ImageViewerApp {
    image_cache: ImageCache,
    image_paths: Vec<PathBuf>,
    current_index: usize,
    fullscreen: bool,
    image_dir: Option<PathBuf>,
    preload_range: usize,
    // Canales para comunicación asíncrona
    image_receiver: mpsc::Receiver<(PathBuf, ColorImage)>,
    image_sender: mpsc::Sender<(PathBuf, ColorImage)>,
    loading_paths: Arc<Mutex<std::collections::HashSet<PathBuf>>>,
    thread_pool_size: usize,
    show_about: bool,
}

impl Default for ImageViewerApp {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            image_cache: ImageCache::new(20), // Aumentado el cache
            image_paths: vec![],
            current_index: 0,
            fullscreen: false,
            image_dir: None,
            preload_range: 10, // Aumentado el rango de precarga
            image_receiver: receiver,
            image_sender: sender,
            loading_paths: Arc::new(Mutex::new(std::collections::HashSet::new())),
            thread_pool_size: num_cpus::get().max(8), // Usar todos los cores disponibles
            show_about: false,
        }
    }
}

impl ImageViewerApp {

    
    fn spawn_image_loading_thread(&self, paths: Vec<PathBuf>) {
        let sender = self.image_sender.clone();
        let loading_paths = Arc::clone(&self.loading_paths);
        
        thread::spawn(move || {
            paths.into_par_iter()
                .filter_map(|path| {
                    // Verificar si ya se está cargando
                    {
                        let mut loading = loading_paths.lock().unwrap();
                        if loading.contains(&path) {
                            return None;
                        }
                        loading.insert(path.clone());
                    }
                    
                    let result = load_image_optimized(path.clone());
                    
                    // Remover del conjunto de carga
                    {
                        let mut loading = loading_paths.lock().unwrap();
                        loading.remove(&path);
                    }
                    
                    result
                })
                .for_each(|loaded_image| {
                    if sender.send(loaded_image).is_err() {
                        // El receptor se ha cerrado, terminar el hilo
                        return;
                    }
                });
        });
    }

    fn preload_images(&mut self, ctx: &egui::Context) {
        if self.image_paths.is_empty() {
            return;
        }

        let current_idx = self.current_index;
        let range = self.preload_range;
        let start = current_idx.saturating_sub(range);
        let end = (current_idx + range + 1).min(self.image_paths.len());
        
        // Priorizar la imagen actual
        let mut paths_to_load = Vec::new();
        
        // Primero la imagen actual
        if current_idx < self.image_paths.len() {
            let current_path = &self.image_paths[current_idx];
            if !self.image_cache.contains(current_path) {
                paths_to_load.push(current_path.clone());
            }
        }
        
        // Luego las adyacentes
        for i in start..end {
            if i != current_idx && i < self.image_paths.len() {
                let path = &self.image_paths[i];
                if !self.image_cache.contains(path) {
                    paths_to_load.push(path.clone());
                }
            }
        }

        if !paths_to_load.is_empty() {
            self.spawn_image_loading_thread(paths_to_load);
        }

        // Procesar imágenes cargadas
        while let Ok((path, color_image)) = self.image_receiver.try_recv() {
            let texture = ctx.load_texture(
                &format!("image_{}", path.display()),
                color_image,
                egui::TextureOptions::LINEAR
            );
            self.image_cache.insert(path, texture);
        }
    }

    fn load_images_from_folder(&mut self, ctx: &egui::Context, path: &Path) {
        if let Some(parent) = path.parent() {
            self.image_cache.clear();
            
            // Leer directorio y filtrar imágenes
            let mut image_paths: Vec<_> = match fs::read_dir(parent) {
                Ok(entries) => {
                    entries
                        .filter_map(|entry| entry.ok())
                        .map(|e| e.path())
                        .filter(|p| {
                            if let Some(ext) = p.extension() {
                                matches!(
                                    ext.to_str().unwrap_or("").to_lowercase().as_str(),
                                    "jpg" | "jpeg" | "png" | "bmp" | "gif" | "tiff" | "webp" | "avif" | "heic"
                                )
                            } else {
                                false
                            }
                        })
                        .collect()
                }
                Err(_) => Vec::new(),
            };
            
            image_paths.sort();
            self.image_paths = image_paths;
            self.current_index = self.image_paths.iter().position(|p| p == path).unwrap_or(0);
            self.image_dir = Some(parent.to_path_buf());
            
            self.preload_images(ctx);
        }
    }

    fn previous_image(&mut self, ctx: &egui::Context) {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.preload_images(ctx);
        }
    }

    fn next_image(&mut self, ctx: &egui::Context) {
        if self.current_index + 1 < self.image_paths.len() {
            self.current_index += 1;
            self.preload_images(ctx);
        }
    }

    fn get_current_texture(&mut self) -> Option<&TextureHandle> {
        if self.current_index < self.image_paths.len() {
            let current_path = &self.image_paths[self.current_index];
            self.image_cache.get(current_path)
        } else {
            None
        }
    }
}




impl eframe::App for ImageViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Solicitar repintado continuo para procesar imágenes cargadas
        ctx.request_repaint();
        

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) && self.fullscreen {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                self.fullscreen = false;
            }
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::Space) {
                self.next_image(ctx);
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.previous_image(ctx);
            }
            if i.key_pressed(egui::Key::F11) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!self.fullscreen));
                self.fullscreen = !self.fullscreen;
            }
        });

        if !self.fullscreen {
            egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(format!("📂 {}", BTN_OPEN)).clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Imagen", &["png", "jpg", "jpeg", "bmp", "gif", "tiff", "webp", "avif", "heic"])
                            .pick_file() 
                        {
                            self.load_images_from_folder(ctx, &path);
                        }
                    }

                    ui.separator();

                    let can_go_prev = self.current_index > 0;
                    let can_go_next = self.current_index + 1 < self.image_paths.len();

                    if ui.add_enabled(can_go_prev, egui::Button::new(format!("⬅️{}", BTN_PREV))).clicked() {
                        self.previous_image(ctx);
                    }

                    if ui.add_enabled(can_go_next, egui::Button::new(format!("➡️ {}", BTN_NEXT))).clicked() {
                        self.next_image(ctx);
                    }

                    ui.separator();

                    if ui.button(format!("🖥️ {}", BTN_FULLSCREEN)).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        self.fullscreen = true;
                    }

                    if ui.button(format!("ℹ️ {}", BTN_ABOUT)).clicked() {
                    self.show_about = true;
                }
                if self.show_about {
                    egui::Window::new(BTN_ABOUT)
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ctx, |ui| {
                            ui.label(format!("{} v{}", APP_NAME, APP_VERSION));
                            ui.label(format!("{}{}", TEXT_AUTHOR, APP_AUTHOR));
                            ui.separator();
                            ui.label(TEXT_INFOAPP);
                            
                            if ui.button(BTN_CLOSE).clicked() {
                                self.show_about = false;
                }
                        });
                    }

                    if !self.image_paths.is_empty() {
                        ui.separator();
                        ui.label(format!("{} / {}", self.current_index + 1, self.image_paths.len()));
                        
                        if let Some(current_path) = self.image_paths.get(self.current_index) {
                            if let Some(filename) = current_path.file_name() {
                                ui.label(filename.to_string_lossy());
                            }
                        }
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = self.get_current_texture() {
                let available_size = ui.available_size();
                let image_size = texture.size_vec2();
                
                let scale = (available_size.x / image_size.x)
                    .min(available_size.y / image_size.y)
                    .min(1.0);
                
                let scaled_size = image_size * scale;
                
                let rect = egui::Rect::from_center_size(
                    ui.available_rect_before_wrap().center(),
                    scaled_size
                );
                
                let response = ui.allocate_rect(rect, egui::Sense::hover());
                egui::Image::from_texture(texture)
                    .fit_to_exact_size(scaled_size)
                    .paint_at(ui, response.rect);
            } else {
                ui.vertical_centered_justified(|ui| {
                    if self.image_paths.is_empty() {
                        ui.label("");
                        ui.label("");
                        ui.label(TEXT_OPENIMG);
                        ui.label(TEXT_ROWSORSPACE);
                        ui.label(TEXT_F11FULLSCREEN);
                    } else {
                        ui.label(TEXT_LOADINGIMG);
                    }
                });
            }
        });

        // Precargar imágenes
        if !self.image_paths.is_empty() {
            self.preload_images(ctx);
        }
    }
}

