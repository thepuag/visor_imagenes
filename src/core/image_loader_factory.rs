use egui::ColorImage;
use std::path::PathBuf;
use crate::core::image_format::{ImageFormat, VectorFormat};

pub trait ImageLoader: Send + Sync {
    fn load(&self, path: &PathBuf) -> Result<ColorImage, LoadError>;
    fn supports_format(&self, format: &ImageFormat) -> bool;
}

#[derive(Debug)]
pub enum LoadError {
    IoError(std::io::Error),
    DecodeError(String),
    FormatError(String),
    UnsupportedFormat,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::IoError(e) => write!(f, "Error de E/S: {}", e),
            LoadError::DecodeError(msg) => write!(f, "Error de decodificación: {}", msg),
            LoadError::FormatError(msg) => write!(f, "Error de formato: {}", msg),
            LoadError::UnsupportedFormat => write!(f, "Formato no soportado"),
        }
    }
}

impl std::error::Error for LoadError {}

pub struct RasterImageLoader;

impl ImageLoader for RasterImageLoader {
    fn load(&self, path: &PathBuf) -> Result<ColorImage, LoadError> {
        use image::ImageReader;
        
        let reader = ImageReader::open(path)
            .map_err(LoadError::IoError)?;
            
        let reader = reader.with_guessed_format()
            .map_err(|e| LoadError::FormatError(e.to_string()))?;
            
        let img = reader.decode()
            .map_err(|e| LoadError::DecodeError(e.to_string()))?;
        
        // Redimensionar para mejor rendimiento
        let img = if img.width() > 1920 || img.height() > 1080 {
            img.resize(1920, 1080, image::imageops::FilterType::Triangle)
        } else {
            img
        };
        
        let rgba_img = img.to_rgba8();
        let size = [rgba_img.width() as usize, rgba_img.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, rgba_img.as_raw());
        
        Ok(color_image)
    }

    fn supports_format(&self, format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::Raster(_))
    }
}

pub struct SvgImageLoader;

impl ImageLoader for SvgImageLoader {
    fn load(&self, path: &PathBuf) -> Result<ColorImage, LoadError> {
        // Aquí implementarías la carga de SVG
        // Puedes usar bibliotecas como resvg o usvg
        // Por ahora, un placeholder:
        
        let _svg_content = std::fs::read_to_string(path)
            .map_err(LoadError::IoError)?;
            
        // TODO: Implementar renderizado SVG a ColorImage
        // Esto requiere una biblioteca como resvg
        
        Err(LoadError::UnsupportedFormat)
    }

    fn supports_format(&self, format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::Vector(VectorFormat::Svg))
    }
}

pub struct ImageLoaderFactory {
    // Usamos Arc para compartir de forma segura entre threads
    loaders: std::sync::Arc<Vec<Box<dyn ImageLoader>>>,
}

impl ImageLoaderFactory {
    pub fn new() -> Self {
        Self {
            loaders: std::sync::Arc::new(vec![
                Box::new(RasterImageLoader),
                Box::new(SvgImageLoader),
            ]),
        }
    }

    pub fn load_image(&self, path: PathBuf) -> Option<(PathBuf, ColorImage)> {
        let format = ImageFormat::from_path(&path)?;
        
        for loader in self.loaders.iter() {
            if loader.supports_format(&format) {
                match loader.load(&path) {
                    Ok(color_image) => return Some((path, color_image)),
                    Err(e) => {
                        eprintln!("Error cargando {}: {}", path.display(), e);
                        return None;
                    }
                }
            }
        }
        
        None
    }
}

impl Clone for ImageLoaderFactory {
    fn clone(&self) -> Self {
        Self {
            loaders: std::sync::Arc::clone(&self.loaders),
        }
    }
}