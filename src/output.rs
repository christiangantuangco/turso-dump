use std::time::Duration;
use turso::Value;

const BLOB_HEX_LIMIT: usize = 24;

#[must_use]
pub(crate) fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Integer(number) => number.to_string(),
        Value::Real(number) => format_real(*number),
        Value::Text(text) => text.clone(),
        Value::Blob(bytes) => format_blob(bytes),
    }
}

#[must_use]
pub(crate) fn format_elapsed(elapsed: Duration) -> String {
    let millis = elapsed.as_secs_f64() * 1000.0;
    if millis < 1000.0 {
        format!("{millis:.2}ms")
    } else {
        format!("{:.3}s", elapsed.as_secs_f64())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Alignment {
    Left,
    Right,
}

#[must_use]
pub(crate) fn is_numeric_value(value: &Value) -> bool {
    matches!(value, Value::Integer(_) | Value::Real(_))
}

#[must_use]
pub(crate) fn render_table(
    columns: &[String],
    rows: &[Vec<String>],
    alignments: &[Alignment],
) -> String {
    if columns.is_empty() {
        return String::new();
    }

    let headers: Vec<String> = columns.iter().map(|column| escape_cell(column)).collect();
    let body: Vec<Vec<String>> = rows
        .iter()
        .map(|row| row.iter().map(|cell| escape_cell(cell)).collect())
        .collect();

    let mut widths: Vec<usize> = headers.iter().map(|header| display_width(header)).collect();
    for row in &body {
        for (index, cell) in row.iter().enumerate() {
            if index < widths.len() {
                widths[index] = widths[index].max(display_width(cell));
            }
        }
    }

    let mut out = String::new();
    out.push_str(&data_row(&headers, &widths, &[]));
    out.push_str(&separator(&widths));
    for row in &body {
        out.push_str(&data_row(row, &widths, alignments));
    }
    out
}

fn separator(widths: &[usize]) -> String {
    let mut out = String::new();
    for (index, width) in widths.iter().enumerate() {
        if index > 0 {
            out.push('+');
        }
        out.push_str(&"-".repeat(width + 2));
    }
    out.push('\n');
    out
}

fn data_row(cells: &[String], widths: &[usize], alignments: &[Alignment]) -> String {
    let mut out = String::new();
    for (index, width) in widths.iter().enumerate() {
        if index > 0 {
            out.push('|');
        }
        let empty = String::new();
        let cell = cells.get(index).unwrap_or(&empty);
        let padding = width.saturating_sub(display_width(cell));
        let alignment = alignments.get(index).copied().unwrap_or(Alignment::Left);
        out.push(' ');
        match alignment {
            Alignment::Right => {
                out.push_str(&" ".repeat(padding));
                out.push_str(cell);
            }
            Alignment::Left => {
                out.push_str(cell);
                out.push_str(&" ".repeat(padding));
            }
        }
        out.push(' ');
    }
    out.push('\n');
    out
}

fn display_width(text: &str) -> usize {
    text.chars().count()
}

fn escape_cell(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn format_real(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn format_blob(bytes: &[u8]) -> String {
    if bytes.len() > BLOB_HEX_LIMIT {
        return format!("<blob {} bytes>", bytes.len());
    }

    let mut out = String::with_capacity(bytes.len() * 2 + 3);
    out.push_str("x'");
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out.push('\'');
    out
}
