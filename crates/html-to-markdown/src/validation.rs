//! Input validation module for HTML to Markdown conversion.
//!
//! Provides validation functions to detect and reject binary data,
//! corrupted input, and other non-text content.

use crate::error::{ConversionError, Result};

const BINARY_SCAN_LIMIT: usize = 8192;
const BINARY_CONTROL_RATIO: f64 = 0.3;
const BINARY_UTF16_NULL_RATIO: f64 = 0.2;
const BINARY_NUL_RATIO: f64 = 0.01;
const BINARY_NUL_MAX: usize = 8;

const BINARY_MAGIC_PREFIXES: &[(&[u8], &str)] = &[
    (b"\x1F\x8B", "gzip-compressed data"),
    (b"\x28\xB5\x2F\xFD", "zstd-compressed data"),
    (b"PK\x03\x04", "zip archive"),
    (b"PK\x05\x06", "zip archive"),
    (b"PK\x07\x08", "zip archive"),
    (b"%PDF-", "PDF data"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Utf16Encoding {
    BomLe,
    BomBe,
    NoBomLe,
    NoBomBe,
}

/// Validate HTML input and reject binary/corrupted data.
///
/// # Errors
///
/// Returns `ConversionError::InvalidInput` if the input is detected as binary,
/// compressed, or contains too many control characters.
#[allow(clippy::cast_precision_loss)]
pub fn validate_input(html: &str) -> Result<()> {
    let bytes = html.as_bytes();
    if bytes.is_empty() {
        return Ok(());
    }

    if let Some(label) = detect_binary_magic(bytes) {
        return Err(ConversionError::InvalidInput(format!(
            "binary data detected ({label}); decode/decompress to UTF-8 HTML first"
        )));
    }

    let sample_len = bytes.len().min(BINARY_SCAN_LIMIT);
    let mut control_count = 0usize;
    let mut nul_count = 0usize;

    for &byte in &bytes[..sample_len] {
        if byte == 0 {
            nul_count += 1;
        }
        let is_control = (byte < 0x09) || (0x0E..0x20).contains(&byte);
        if is_control {
            control_count += 1;
        }
    }

    if nul_count > 0 {
        if let Some(encoding) = detect_utf16_encoding(bytes) {
            let label = utf16_label(encoding);
            return Err(ConversionError::InvalidInput(format!(
                "binary data detected ({label}); decode to UTF-8 HTML first"
            )));
        }
        let nul_ratio = nul_count as f64 / sample_len as f64;
        if nul_ratio > BINARY_NUL_RATIO && nul_count > BINARY_NUL_MAX {
            return Err(ConversionError::InvalidInput(
                "binary data detected (excess NUL bytes)".to_string(),
            ));
        }
    }

    if sample_len >= 32 {
        let control_ratio = control_count as f64 / sample_len as f64;
        if control_ratio > BINARY_CONTROL_RATIO {
            return Err(ConversionError::InvalidInput(
                "binary data detected (excess control bytes)".to_string(),
            ));
        }
    }

    Ok(())
}

fn detect_binary_magic(bytes: &[u8]) -> Option<&'static str> {
    for (prefix, label) in BINARY_MAGIC_PREFIXES {
        if bytes.starts_with(prefix) {
            return Some(*label);
        }
    }
    None
}

#[allow(clippy::cast_precision_loss)]
pub fn detect_utf16_encoding(bytes: &[u8]) -> Option<Utf16Encoding> {
    if bytes.len() >= 2 {
        if bytes.starts_with(b"\xFF\xFE") {
            return Some(Utf16Encoding::BomLe);
        }
        if bytes.starts_with(b"\xFE\xFF") {
            return Some(Utf16Encoding::BomBe);
        }
    }

    let sample_len = bytes.len().min(BINARY_SCAN_LIMIT);
    if sample_len < 4 {
        return None;
    }

    let mut nul_count = 0usize;
    let mut even_nul_count = 0usize;
    let mut odd_nul_count = 0usize;

    for (idx, &byte) in bytes[..sample_len].iter().enumerate() {
        if byte == 0 {
            nul_count += 1;
            if idx % 2 == 0 {
                even_nul_count += 1;
            } else {
                odd_nul_count += 1;
            }
        }
    }

    if nul_count < 2 {
        return None;
    }

    let nul_ratio = nul_count as f64 / sample_len as f64;
    if nul_ratio < BINARY_UTF16_NULL_RATIO {
        return None;
    }

    let dominant_ratio = (even_nul_count.max(odd_nul_count) as f64) / nul_count as f64;
    if dominant_ratio < 0.9 {
        return None;
    }

    if odd_nul_count >= even_nul_count {
        Some(Utf16Encoding::NoBomLe)
    } else {
        Some(Utf16Encoding::NoBomBe)
    }
}

fn utf16_label(encoding: Utf16Encoding) -> &'static str {
    match encoding {
        Utf16Encoding::BomLe => "UTF-16LE BOM",
        Utf16Encoding::BomBe => "UTF-16BE BOM",
        Utf16Encoding::NoBomLe | Utf16Encoding::NoBomBe => "UTF-16 data without BOM",
    }
}
