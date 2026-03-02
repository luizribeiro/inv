use qrcode::render::unicode;
use qrcode::QrCode;

use crate::error::{AppError, Result};

pub fn render(payload: &str) -> Result<String> {
    if payload.trim().is_empty() {
        return Err(AppError::Validation(
            "QR payload must not be empty".to_string(),
        ));
    }

    let code = QrCode::new(payload.as_bytes()).map_err(|error| {
        AppError::Validation(format!("failed to generate terminal QR code: {error}"))
    })?;

    Ok(code.render::<unicode::Dense1x2>().quiet_zone(true).build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_is_deterministic_for_same_payload() {
        let payload = "https://example.com/shortcut";

        let first = render(payload).expect("render should succeed");
        let second = render(payload).expect("render should succeed");

        assert_eq!(first, second);
    }

    #[test]
    fn render_contains_multiple_lines_and_block_glyphs() {
        let output = render("https://example.com/shortcut").expect("render should succeed");

        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() > 8, "terminal QR should span multiple lines");

        let block_count = output
            .chars()
            .filter(|ch| matches!(ch, '█' | '▀' | '▄' | '▓' | '▌' | '▐'))
            .count();

        assert!(
            block_count > 20,
            "terminal QR should include visible block glyphs"
        );
    }

    #[test]
    fn render_rejects_empty_payload() {
        let error = render("   ").expect_err("empty payload must fail");
        assert!(matches!(error, AppError::Validation(_)));
    }
}
