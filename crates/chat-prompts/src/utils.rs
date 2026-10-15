use crate::error::{PromptError, Result};
use base64::{engine::general_purpose, Engine as _};
use endpoints::chat::{
    ChatCompletionAssistantMessage, ChatCompletionUserMessage, ChatCompletionUserMessageContent,
    ContentPart,
};
use image::io::Reader as ImageReader;
use std::io::Cursor;

/// Extract the text content from a user message.
///
/// If the message content is a single text value, it is returned as-is. If it
/// is a list of parts, the text of each [`ContentPart::Text`] part is
/// concatenated (each followed by a newline) and non-text parts are ignored.
pub fn get_text_content_from_user_message(message: &ChatCompletionUserMessage) -> String {
    match message.content() {
        ChatCompletionUserMessageContent::Text(text) => text.to_string(),
        ChatCompletionUserMessageContent::Parts(parts) => {
            let mut content = String::new();
            for part in parts {
                if let ContentPart::Text(text_content) = part {
                    content.push_str(text_content.text());
                    content.push('\n');
                }
            }
            content
        }
    }
}

/// Extract the content from an assistant message.
///
/// The content of an assistant message is optional when `tool_calls` is
/// specified. In that case an empty string is returned; otherwise a
/// [`PromptError::NoAssistantMessage`] error is returned.
pub fn get_content_from_assistant_message(
    message: &ChatCompletionAssistantMessage,
) -> Result<String> {
    match message.content() {
        Some(content) => Ok(content.to_string()),
        // Note that the content is optional if `tool_calls` is specified.
        None => match message.tool_calls().is_some() {
            true => Ok(String::new()),
            false => Err(PromptError::NoAssistantMessage),
        },
    }
}

/// Get the image format from a base64-encoded image string.
pub fn get_image_format(base64_str: &str) -> Result<String> {
    let image_data = match general_purpose::STANDARD.decode(base64_str) {
        Ok(data) => data,
        Err(_) => {
            return Err(PromptError::Operation(
                "Failed to decode base64 string.".to_string(),
            ))
        }
    };

    let format = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()
        .unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    // 1x1 pixel images encoded as base64.
    const PNG_1X1: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4z8AAAAMBAQDJ/pLvAAAAAElFTkSuQmCC";
    const GIF_1X1: &str = "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7";

    #[test]
    fn test_get_image_format_png() {
        assert_eq!(get_image_format(PNG_1X1).unwrap(), "png");
    }

    #[test]
    fn test_get_image_format_gif() {
        assert_eq!(get_image_format(GIF_1X1).unwrap(), "gif");
    }

    #[test]
    fn test_get_image_format_invalid_base64() {
        // `!!!` is not valid base64.
        let err = get_image_format("!!!").unwrap_err();
        assert!(matches!(err, PromptError::Operation(_)));
    }

    #[test]
    fn test_get_image_format_unsupported() {
        // Valid base64 but not a recognizable image format.
        let not_an_image = base64::engine::general_purpose::STANDARD.encode("not an image");
        let err = get_image_format(&not_an_image).unwrap_err();
        assert!(matches!(err, PromptError::Operation(_)));
    }
}
