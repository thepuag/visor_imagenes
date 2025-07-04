use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Raster(RasterFormat),
    Vector(VectorFormat),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterFormat {
    Jpeg,
    Png,
    Bmp,
    Gif,
    Tiff,
    Webp,
    Avif,
    Heic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VectorFormat {
    Svg,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(ImageFormat::Raster(RasterFormat::Jpeg)),
            "png" => Some(ImageFormat::Raster(RasterFormat::Png)),
            "bmp" => Some(ImageFormat::Raster(RasterFormat::Bmp)),
            "gif" => Some(ImageFormat::Raster(RasterFormat::Gif)),
            "tiff" | "tif" => Some(ImageFormat::Raster(RasterFormat::Tiff)),
            "webp" => Some(ImageFormat::Raster(RasterFormat::Webp)),
            "avif" => Some(ImageFormat::Raster(RasterFormat::Avif)),
            "heic" => Some(ImageFormat::Raster(RasterFormat::Heic)),
            "svg" => Some(ImageFormat::Vector(VectorFormat::Svg)),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    pub fn is_supported(path: &Path) -> bool {
        Self::from_path(path).is_some()
    }

    pub fn get_supported_extensions() -> Vec<&'static str> {
        vec![
            "jpg", "jpeg", "png", "bmp", "gif", "tiff", "tif",
            "webp", "avif", "heic", "svg"
        ]
    }
}