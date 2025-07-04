
use std::path::PathBuf;

pub struct NavigationManager {
    image_paths: Vec<PathBuf>,
    current_index: usize,
}

impl NavigationManager {
    pub fn new() -> Self {
        Self {
            image_paths: Vec::new(),
            current_index: 0,
        }
    }

    pub fn set_images(&mut self, paths: Vec<PathBuf>, initial_index: usize) {
        self.image_paths = paths;
        self.current_index = initial_index.min(self.image_paths.len().saturating_sub(1));
    }

    pub fn previous(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            false
        }
    }

    pub fn next(&mut self) -> bool {
        if self.current_index + 1 < self.image_paths.len() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    pub fn can_go_previous(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_go_next(&self) -> bool {
        self.current_index + 1 < self.image_paths.len()
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn current_path(&self) -> Option<&PathBuf> {
        self.image_paths.get(self.current_index)
    }

    pub fn image_paths(&self) -> &[PathBuf] {
        &self.image_paths
    }

    pub fn total_images(&self) -> usize {
        self.image_paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.image_paths.is_empty()
    }

    pub fn clear(&mut self) {
        self.image_paths.clear();
        self.current_index = 0;
    }
}