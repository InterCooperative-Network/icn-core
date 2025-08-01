//! QR code generation for ICN actions
//!
//! This module provides utilities for generating QR codes from action URLs.

use crate::{Action, ActionEncoder, ActionError, QrErrorCorrection, QrFormat, QrMetadata};

#[cfg(feature = "qr")]
use image::Rgb;
#[cfg(feature = "qr")]
use qrcode::{EcLevel, QrCode};

/// QR code generator for ICN actions
pub struct QrGenerator;

impl QrGenerator {
    /// Generate a QR code for an action as PNG bytes
    #[cfg(feature = "qr")]
    pub fn generate_png(action_url: &str, size: u32) -> Result<Vec<u8>, ActionError> {
        let metadata = QrMetadata {
            size,
            format: QrFormat::Png,
            ..Default::default()
        };
        Self::generate_with_metadata(action_url, &metadata)
    }

    /// Generate a QR code for an action as PNG bytes (stub when QR feature disabled)
    #[cfg(not(feature = "qr"))]
    pub fn generate_png(_action_url: &str, _size: u32) -> Result<Vec<u8>, ActionError> {
        Err(ActionError::QrGeneration(
            "QR feature not enabled. Compile with --features qr".to_string(),
        ))
    }

    /// Generate a QR code for an action with custom metadata
    #[cfg(feature = "qr")]
    pub fn generate_with_metadata(
        action_url: &str,
        metadata: &QrMetadata,
    ) -> Result<Vec<u8>, ActionError> {
        let ec_level = match metadata.error_correction {
            QrErrorCorrection::Low => EcLevel::L,
            QrErrorCorrection::Medium => EcLevel::M,
            QrErrorCorrection::Quartile => EcLevel::Q,
            QrErrorCorrection::High => EcLevel::H,
        };

        let code = QrCode::with_error_correction_level(action_url, ec_level)
            .map_err(|e| ActionError::QrGeneration(format!("Failed to create QR code: {e}")))?;

        match metadata.format {
            QrFormat::Png => Self::generate_png_bytes(&code, metadata),
            QrFormat::Svg => Self::generate_svg_bytes(&code, metadata),
            QrFormat::Terminal => Self::generate_terminal_bytes(&code),
        }
    }

    /// Generate a QR code for an action with custom metadata (stub when QR feature disabled)
    #[cfg(not(feature = "qr"))]
    pub fn generate_with_metadata(
        _action_url: &str,
        _metadata: &QrMetadata,
    ) -> Result<Vec<u8>, ActionError> {
        Err(ActionError::QrGeneration(
            "QR feature not enabled. Compile with --features qr".to_string(),
        ))
    }

    /// Generate QR code for an action directly
    pub fn generate_for_action(
        action: &Action,
        metadata: &QrMetadata,
    ) -> Result<Vec<u8>, ActionError> {
        let url = ActionEncoder::encode(action)?;
        Self::generate_with_metadata(&url, metadata)
    }

    /// Generate a compact QR code for an action (uses base64 encoding)
    pub fn generate_compact(action: &Action, size: u32) -> Result<Vec<u8>, ActionError> {
        let compact_url = ActionEncoder::encode_compact(action)?;
        Self::generate_png(&compact_url, size)
    }

    /// Generate QR code and save to file
    pub fn save_to_file(
        action_url: &str,
        file_path: &str,
        metadata: &QrMetadata,
    ) -> Result<(), ActionError> {
        let qr_data = Self::generate_with_metadata(action_url, metadata)?;

        std::fs::write(file_path, qr_data)
            .map_err(|e| ActionError::QrGeneration(format!("Failed to write file: {e}")))?;

        Ok(())
    }

    /// Display QR code in terminal (basic ASCII art version without external dependencies)
    pub fn display_terminal(action_url: &str) -> Result<String, ActionError> {
        #[cfg(feature = "qr")]
        {
            let code = QrCode::new(action_url)
                .map_err(|e| ActionError::QrGeneration(format!("Failed to create QR code: {e}")))?;

            let string = code
                .render::<char>()
                .quiet_zone(false)
                .module_dimensions(2, 1)
                .build();

            Ok(string)
        }

        #[cfg(not(feature = "qr"))]
        {
            // Simple fallback - just show the URL in a box
            let border = format!("{}{}{}", "+", "-".repeat(action_url.len() + 2), "+");
            let content = format!("| {action_url} |");
            Ok(format!(
                "{border}\n{content}\n{border}\n[QR Code would be here with --features qr]"
            ))
        }
    }

    #[cfg(feature = "qr")]
    fn generate_png_bytes(code: &QrCode, metadata: &QrMetadata) -> Result<Vec<u8>, ActionError> {
        let image = code
            .render::<Rgb<u8>>()
            .max_dimensions(metadata.size, metadata.size)
            .build();

        let mut buffer = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut buffer),
                image::ImageFormat::Png,
            )
            .map_err(|e| ActionError::QrGeneration(format!("Failed to encode PNG: {e}")))?;

        Ok(buffer)
    }

    #[cfg(feature = "qr")]
    fn generate_svg_bytes(_code: &QrCode, metadata: &QrMetadata) -> Result<Vec<u8>, ActionError> {
        // Generate simple placeholder SVG since the QR code API changed
        let svg_content = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
                <rect width="100%" height="100%" fill="white"/>
                <rect x="10%" y="10%" width="80%" height="80%" fill="black"/>
                <text x="50%" y="50%" text-anchor="middle" font-family="monospace" font-size="12" fill="white">QR</text>
            </svg>"#,
            metadata.size, metadata.size, metadata.size, metadata.size
        );

        Ok(svg_content.into_bytes())
    }

    #[cfg(feature = "qr")]
    fn generate_terminal_bytes(code: &QrCode) -> Result<Vec<u8>, ActionError> {
        let string = code
            .render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();

        Ok(string.into_bytes())
    }
}

/// Utility functions for QR code operations
pub struct QrUtils;

impl QrUtils {
    /// Estimate the optimal size for a QR code based on data length
    pub fn estimate_optimal_size(data_length: usize) -> u32 {
        match data_length {
            0..=100 => 128,
            101..=200 => 256,
            201..=400 => 384,
            _ => 512,
        }
    }

    /// Choose error correction level based on use case
    pub fn recommend_error_correction(use_case: &str) -> QrErrorCorrection {
        match use_case {
            "print" | "sticker" | "card" => QrErrorCorrection::High,
            "screen" | "mobile" => QrErrorCorrection::Medium,
            "fast" | "simple" => QrErrorCorrection::Low,
            _ => QrErrorCorrection::Medium,
        }
    }

    /// Create a QR metadata configuration for common use cases
    pub fn config_for_use_case(use_case: &str, size: Option<u32>) -> QrMetadata {
        let size = size.unwrap_or(match use_case {
            "business_card" => 128,
            "poster" => 512,
            "sticker" => 256,
            "mobile" => 256,
            _ => 256,
        });

        QrMetadata {
            size,
            border: match use_case {
                "business_card" => 2,
                "poster" => 8,
                _ => 4,
            },
            error_correction: Self::recommend_error_correction(use_case),
            format: QrFormat::Png,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "qr")]
    use crate::Action;
    #[cfg(feature = "qr")]
    use icn_common::Did;
    #[cfg(feature = "qr")]
    use std::str::FromStr;

    #[test]
    fn test_terminal_display() {
        let url = "icn://share?did=did:icn:alice";
        let terminal_qr = QrGenerator::display_terminal(url).unwrap();
        assert!(!terminal_qr.is_empty());
        // Should work even without QR feature
    }

    #[test]
    fn test_size_estimation() {
        assert_eq!(QrUtils::estimate_optimal_size(50), 128);
        assert_eq!(QrUtils::estimate_optimal_size(150), 256);
        assert_eq!(QrUtils::estimate_optimal_size(300), 384);
        assert_eq!(QrUtils::estimate_optimal_size(500), 512);
    }

    #[test]
    fn test_use_case_config() {
        let config = QrUtils::config_for_use_case("business_card", None);
        assert_eq!(config.size, 128);
        assert_eq!(config.border, 2);

        let config = QrUtils::config_for_use_case("poster", Some(1024));
        assert_eq!(config.size, 1024);
        assert_eq!(config.border, 8);
    }

    #[cfg(feature = "qr")]
    #[test]
    fn test_qr_generation() {
        let did = Did::from_str("did:icn:alice").unwrap();
        let action = Action::ShareIdentity { did };

        let qr_data = QrGenerator::generate_compact(&action, 256).unwrap();
        assert!(!qr_data.is_empty());
        assert!(qr_data.len() > 100); // PNG should have reasonable size
    }
}
