use crate::error::{PromptError, Result};
use base64::{engine::general_purpose, Engine as _};
use image::io::Reader as ImageReader;
use std::io::Cursor;

/// Get the image format from a base64-encoded image string.
pub fn get_image_format(base64_str: &str) -> Result<String> {
    let image_data = general_purpose::STANDARD.decode(base64_str).map_err(|e| {
        PromptError::Operation(format!("Failed to decode base64 string. Reason: {e}"))
    })?;

    let format = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()
        .map_err(|e| {
            PromptError::Operation(format!("Failed to guess the image format. Reason: {e}"))
        })?
        .format();

    let image_format = match format {
        Some(image::ImageFormat::Png) => "png".to_string(),
        Some(image::ImageFormat::Jpeg) => "jpeg".to_string(),
        Some(image::ImageFormat::Tga) => "tga".to_string(),
        Some(image::ImageFormat::Bmp) => "bmp".to_string(),
        Some(image::ImageFormat::Gif) => "gif".to_string(),
        Some(image::ImageFormat::Hdr) => "hdr".to_string(),
        Some(image::ImageFormat::Pnm) => "pnm".to_string(),
        _ => {
            return Err(PromptError::Operation(
                "Unsupported image format.".to_string(),
            ))
        }
    };

    Ok(image_format)
}
