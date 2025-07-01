use eframe::egui;
use egui::{TextureHandle, ColorImage};
use rfd::FileDialog;
use image::ImageReader;
use std::path::{PathBuf, Path};
use std::fs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use rayon::prelude::*;

    const APP_NAME: &str = "Visor de Imágenes";
    const APP_VERSION: &str = "1.1";
    const APP_AUTHOR: &str = "Noé Montoro García";

#[derive(Clone)]
struct LoadedImage {
    texture: TextureHandle,
    path: PathBuf,
}

struct ImageCache {
    textures: HashMap<PathBuf, TextureHandle>,
    max_cache_size: usize,
    access_order: Vec<PathBuf>,
}

impl ImageCache {
    fn new(max_size: usize) -> Self {
        Self {
            textures: HashMap::new(),
            max_cache_size: max_size,
            access_order: Vec::new(),
        }
    }

    fn get(&mut self, path: &Path) -> Option<&TextureHandle> {
        if let Some(texture) = self.textures.get(path) {
            if let Some(pos) = self.access_order.iter().position(|p| p == path) {
                let path = self.access_order.remove(pos);
                self.access_order.push(path);
            }
            Some(texture)
        } else {
            None
        }
    }

    fn insert(&mut self, path: PathBuf, texture: TextureHandle) {
        while self.textures.len() >= self.max_cache_size && !self.access_order.is_empty() {
            let oldest = self.access_order.remove(0);
            self.textures.remove(&oldest);
        }

        self.textures.insert(path.clone(), texture);
        self.access_order.push(path);
    }

    fn clear(&mut self) {
        self.textures.clear();
        self.access_order.clear();
    }

    fn contains(&self, path: &Path) -> bool {
        self.textures.contains_key(path)
    }
}

struct ImageViewerApp {
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
            preload_range: 5, // Aumentado el rango de precarga
            image_receiver: receiver,
            image_sender: sender,
            loading_paths: Arc::new(Mutex::new(std::collections::HashSet::new())),
            thread_pool_size: num_cpus::get().max(8), // Usar todos los cores disponibles
            show_about: false,
        }
    }
}

impl ImageViewerApp {
    fn load_image_optimized(path: PathBuf) -> Option<(PathBuf, ColorImage)> {
        match ImageReader::open(&path) {
            Ok(reader) => {
                match reader.with_guessed_format() {
                    Ok(reader) => {
                        match reader.decode() {
                            Ok(img) => {
                                // Redimensionar más agresivamente para mejor rendimiento
                                let img = if img.width() > 1920 || img.height() > 1080 {
                                    // Usar filtro más rápido para mejor rendimiento
                                    img.resize(1920, 1080, image::imageops::FilterType::Triangle)
                                } else {
                                    img
                                };
                                
                                let rgba_img = img.to_rgba8();
                                let size = [rgba_img.width() as usize, rgba_img.height() as usize];
                                let color_image = ColorImage::from_rgba_unmultiplied(size, rgba_img.as_raw());
                                
                                Some((path, color_image))
                            }
                            Err(e) => {
                                eprintln!("Error al decodificar la imagen {:?}: {}", path, e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error al determinar formato de {:?}: {}", path, e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Error al abrir la imagen {:?}: {}", path, e);
                None
            }
        }
    }

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
                    
                    let result = Self::load_image_optimized(path.clone());
                    
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
                    if ui.button("📂 Abrir").clicked() {
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

                    if ui.add_enabled(can_go_prev, egui::Button::new("⬅️ Anterior")).clicked() {
                        self.previous_image(ctx);
                    }

                    if ui.add_enabled(can_go_next, egui::Button::new("➡️ Siguiente")).clicked() {
                        self.next_image(ctx);
                    }

                    ui.separator();

                    if ui.button("🖥️ Pantalla completa (F11)").clicked() {
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
                        ui.label("📂 Abre una imagen para comenzar");
                        ui.label("Usa las flechas ← → o Espacio para navegar");
                        ui.label("F11 para pantalla completa");
                    } else {
                        ui.label("⏳ Cargando imagen...");
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

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_drag_and_drop(true)
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Ok(Box::new(ImageViewerApp::default()))),
    )
}