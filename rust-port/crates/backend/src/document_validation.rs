const DEFAULT_MAX_BYTES: usize = 25 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentValidationVerdict {
    pub normalized_mime_type: Option<String>,
    pub scanner_status: &'static str,
}

pub fn validate_uploaded_document(
    original_name: &str,
    declared_mime_type: Option<&str>,
    bytes: &[u8],
) -> Result<DocumentValidationVerdict, String> {
    if bytes.is_empty() {
        return Err("Uploaded document is empty.".into());
    }
    if bytes.len() > DEFAULT_MAX_BYTES {
        return Err(format!(
            "Uploaded document exceeds the {} MB enterprise limit.",
            DEFAULT_MAX_BYTES / 1024 / 1024
        ));
    }

    let extension = original_name
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if matches!(
        extension.as_str(),
        "exe" | "dll" | "bat" | "cmd" | "com" | "ps1" | "sh" | "js" | "html" | "htm" | "php"
    ) {
        return Err(format!(
            "{} is blocked by the enterprise document upload policy.",
            extension
        ));
    }

    let sniffed = sniff_mime_type(bytes);
    let declared = declared_mime_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());
    if let (Some(declared), Some(sniffed)) = (declared.as_deref(), sniffed)
        && !mime_types_compatible(declared, sniffed)
    {
        return Err(format!(
            "Declared MIME type {} does not match detected {} content.",
            declared, sniffed
        ));
    }

    Ok(DocumentValidationVerdict {
        normalized_mime_type: sniffed.map(str::to_string).or(declared),
        scanner_status: "policy_clean_scanner_pending",
    })
}

fn sniff_mime_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"%PDF-") {
        Some("application/pdf")
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']) {
        Some("image/png")
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else {
        None
    }
}

fn mime_types_compatible(declared: &str, sniffed: &str) -> bool {
    declared == sniffed
        || (declared == "image/jpg" && sniffed == "image/jpeg")
        || (declared == "application/octet-stream" && sniffed.starts_with("image/"))
        || (declared == "application/octet-stream" && sniffed == "application/pdf")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blocked_extensions() {
        let error = validate_uploaded_document("invoice.exe", None, b"payload")
            .expect_err("executable uploads should be blocked");
        assert!(error.contains("blocked"));
    }

    #[test]
    fn rejects_declared_mime_mismatch() {
        let error = validate_uploaded_document(
            "photo.pdf",
            Some("application/pdf"),
            &[0xff, 0xd8, 0xff, 0x00],
        )
        .expect_err("mismatched MIME should fail");
        assert!(error.contains("does not match"));
    }

    #[test]
    fn accepts_pdf_and_returns_scanner_hook_status() {
        let verdict = validate_uploaded_document("pod.pdf", Some("application/pdf"), b"%PDF-1.7")
            .expect("valid PDF should pass");
        assert_eq!(
            verdict.normalized_mime_type.as_deref(),
            Some("application/pdf")
        );
        assert_eq!(verdict.scanner_status, "policy_clean_scanner_pending");
    }
}
