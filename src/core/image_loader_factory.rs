use egui::ColorImage;
use image::ImageReader;
use std::path::PathBuf;
use egui::TextureHandle;
use crate::i18n::es::{ERROR_DECODE, ERROR_FORMAT, ERROR_OPEN};

#[derive(Clone)]
struct LoadedImage {
    texture: TextureHandle,
    path: PathBuf,
}

pub fn load_image_optimized(path: PathBuf) -> Option<(PathBuf, ColorImage)> {
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
                                eprintln!("{} {:?}: {}", ERROR_DECODE, path, e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {:?}: {}", ERROR_FORMAT, path, e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("{} {:?}: {}", ERROR_OPEN, path, e);
                None
            }
        }
    }
