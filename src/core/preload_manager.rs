use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::collections::HashSet;
use rayon::prelude::*;
use egui::{ColorImage, Context};
use crate::core::image_cache::ImageCache;
use crate::core::image_loader_factory::ImageLoaderFactory;

pub struct PreloadManager {
    image_receiver: mpsc::Receiver<(PathBuf, ColorImage)>,
    image_sender: mpsc::Sender<(PathBuf, ColorImage)>,
    loading_paths: Arc<Mutex<HashSet<PathBuf>>>,
    preload_range: usize,
    loader_factory: ImageLoaderFactory,
}

impl PreloadManager {
    pub fn new(preload_range: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            image_receiver: receiver,
            image_sender: sender,
            loading_paths: Arc::new(Mutex::new(HashSet::new())),
            preload_range,
            loader_factory: ImageLoaderFactory::new(),
        }
    }

    pub fn preload_images_around_index(
        &self, 
        image_paths: &[PathBuf], 
        current_index: usize,
        image_cache: &ImageCache
    ) {
        if image_paths.is_empty() {
            return;
        }

        let range = self.preload_range;
        let start = current_index.saturating_sub(range);
        let end = (current_index + range + 1).min(image_paths.len());
        
        let mut paths_to_load = Vec::new();
        
        // Priorizar imagen actual
        if current_index < image_paths.len() {
            let current_path = &image_paths[current_index];
            if !image_cache.contains(current_path) {
                paths_to_load.push(current_path.clone());
            }
        }
        
        // Luego las adyacentes
        for i in start..end {
            if i != current_index && i < image_paths.len() {
                let path = &image_paths[i];
                if !image_cache.contains(path) {
                    paths_to_load.push(path.clone());
                }
            }
        }

        if !paths_to_load.is_empty() {
            self.spawn_loading_thread(paths_to_load);
        }
    }

    fn spawn_loading_thread(&self, paths: Vec<PathBuf>) {
        let sender = self.image_sender.clone();
        let loading_paths = Arc::clone(&self.loading_paths);
        let factory = self.loader_factory.clone(); // Usar clone en lugar de crear nueva instancia
        
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
                    
                    let result = factory.load_image(path.clone());
                    
                    // Remover del conjunto de carga
                    {
                        let mut loading = loading_paths.lock().unwrap();
                        loading.remove(&path);
                    }
                    
                    result
                })
                .for_each(|loaded_image| {
                    if sender.send(loaded_image).is_err() {
                        return; // El receptor se ha cerrado
                    }
                });
        });
    }

    pub fn process_loaded_images(&self, ctx: &Context, image_cache: &mut ImageCache) {
        while let Ok((path, color_image)) = self.image_receiver.try_recv() {
            let texture = ctx.load_texture(
                &format!("image_{}", path.display()),
                color_image,
                egui::TextureOptions::LINEAR
            );
            image_cache.insert(path, texture);
        }
    }
}