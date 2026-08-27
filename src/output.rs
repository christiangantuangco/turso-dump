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

#[must_use]
pub(crate) fn render_table(columns: &[String], rows: &[Vec<String>]) -> String {
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
    out.push_str(&border(&widths, '┌', '┬', '┐'));
    out.push_str(&data_row(&headers, &widths));
    out.push_str(&border(&widths, '├', '┼', '┤'));
    for row in &body {
        out.push_str(&data_row(row, &widths));
    }
    out.push_str(&border(&widths, '└', '┴', '┘'));
    out
}

fn border(widths: &[usize], left: char, middle: char, right: char) -> String {
    let mut out = String::new();
    out.push(left);
    for (index, width) in widths.iter().enumerate() {
        if index > 0 {
            out.push(middle);
        }
        out.push_str(&"─".repeat(width + 2));
    }
    out.push(right);
    out.push('\n');
    out
}

fn data_row(cells: &[String], widths: &[usize]) -> String {
    let mut out = String::new();
    out.push('│');
    for (index, width) in widths.iter().enumerate() {
        let empty = String::new();
        let cell = cells.get(index).unwrap_or(&empty);
        let padding = width.saturating_sub(display_width(cell));
        out.push(' ');
        out.push_str(cell);
        out.push_str(&" ".repeat(padding));
        out.push_str(" │");
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
