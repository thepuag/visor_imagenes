
use std::path::{Path, PathBuf};
use std::fs;
use crate::core::image_format::ImageFormat;

pub struct FileManager;

impl FileManager {
    pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut image_paths: Vec<PathBuf> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .map(|e| e.path())
            .filter(|p| ImageFormat::is_supported(p))
            .collect();
            
        image_paths.sort();
        Ok(image_paths)
    }

    pub fn find_index_of_file(paths: &[PathBuf], target: &Path) -> Option<usize> {
        paths.iter().position(|p| p == target)
    }

    pub fn get_supported_file_filter() -> (&'static str, Vec<&'static str>) {
        ("Imagen", ImageFormat::get_supported_extensions())
    }
}