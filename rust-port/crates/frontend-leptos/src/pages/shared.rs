pub fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" | "selected" | "delivered" | "active" | "approved" | "verified" => {
            "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;"
        }
        "high" | "warning" | "pending" | "deferred" | "retrying" | "queued" | "paused" => {
            "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;"
        }
        "danger" | "failed" | "rejected" | "urgent" | "dead_letter" | "disabled" => {
            "background:#ffe4e6;padding:0.25rem 0.6rem;border-radius:999px;color:#be123c;"
        }
        "info" | "primary" | "replay_queued" => {
            "background:#e0f2fe;padding:0.25rem 0.6rem;border-radius:999px;color:#0369a1;"
        }
        _ => "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;",
    }
}

pub fn parse_required_u64(value: &str, field: &str) -> Result<u64, String> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("Enter a valid {} before running this action.", field))
}

pub fn parse_optional_i64(value: &str) -> Result<Option<i64>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        trimmed
            .parse::<i64>()
            .map(Some)
            .map_err(|_| format!("{} is not a valid whole number.", value))
    }
}

pub fn human_file_size(bytes: Option<u64>) -> String {
    match bytes {
        Some(value) if value >= 1024 * 1024 => format!("{:.1} MB", value as f64 / 1024.0 / 1024.0),
        Some(value) if value >= 1024 => format!("{:.1} KB", value as f64 / 1024.0),
        Some(value) => format!("{} B", value),
        None => "Size not recorded".into(),
    }
}

pub fn split_comma_values(value: String) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

pub const MUTED_TEXT_STYLE: &str = "color:#64748b;";
pub const FIELD_LABEL_STYLE: &str = "display:grid;gap:0.35rem;";
pub const FIELD_INPUT_STYLE: &str =
    "padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;";
pub const TABLE_HEAD_CELL_STYLE: &str = "text-align:left;padding:0.75rem;";
pub const TABLE_CELL_STYLE: &str = "padding:0.75rem;";
pub const TABLE_OVERFLOW_STYLE: &str = "overflow:auto;";
pub const TABLE_HEADER_STYLE: &str = "background:#f8fafc;";
pub const ROW_BORDER_STYLE: &str = "border-top:1px solid #e5e7eb;";
pub const PANEL_SCROLL_STYLE: &str = "padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;";
