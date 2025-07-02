use std::collections::HashMap;
use std::path::{PathBuf, Path};
use egui::TextureHandle;

pub struct ImageCache {
    textures: HashMap<PathBuf, TextureHandle>,
    max_cache_size: usize,
    access_order: Vec<PathBuf>,
}

impl ImageCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            textures: HashMap::new(),
            max_cache_size: max_size,
            access_order: Vec::new(),
        }
    }

    pub fn get(&mut self, path: &Path) -> Option<&TextureHandle> {
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

    pub fn insert(&mut self, path: PathBuf, texture: TextureHandle) {
        while self.textures.len() >= self.max_cache_size && !self.access_order.is_empty() {
            let oldest = self.access_order.remove(0);
            self.textures.remove(&oldest);
        }

        self.textures.insert(path.clone(), texture);
        self.access_order.push(path);
    }

    pub fn clear(&mut self) {
        self.textures.clear();
        self.access_order.clear();
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.textures.contains_key(path)
    }
}
