//! RFC 4180 compliant Delimited Text (CSV / TSV) parser and emitter.
//!
//! Supports customizable delimiters, quote characters, delimiter sniffing,
//! multi-line quoted fields, automatic type inference, and bidirectional
//! mapping to the universal [`Value`] model.

use crate::error::{BabbelError, Location};
use crate::io::traits::IDestination;
use crate::model::Value;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Configuration options for delimited text (CSV / TSV) parsing and serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvOptions {
    /// Column delimiter character (default: `,`).
    pub delimiter: char,
    /// Quoting character (default: `"`).
    pub quote: char,
    /// Optional escape character for quotes. When `None`, quotes are escaped by doubling them (`""`), per RFC 4180.
    pub escape: Option<char>,
    /// Whether the first line is treated as column headers (default: `true`).
    pub has_header: bool,
    /// Whether to trim unquoted leading and trailing whitespace from fields (default: `true`).
    pub trim: bool,
    /// Whether to infer primitive types (booleans, integers, floats, nulls) for unquoted values (default: `true`).
    pub infer_types: bool,
    /// Optional comment character (e.g. `#`). Lines starting with this character outside quotes are ignored.
    pub comment: Option<char>,
    /// Line terminator to use when serializing (default: `"\r\n"` per RFC 4180).
    pub line_terminator: &'static str,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: ',',
            quote: '"',
            escape: None,
            has_header: true,
            trim: true,
            infer_types: true,
            comment: None,
            line_terminator: "\r\n",
        }
    }
}

impl CsvOptions {
    /// Creates options configured for Tab-Separated Values (TSV).
    pub fn tsv() -> Self {
        Self {
            delimiter: '\t',
            ..Default::default()
        }
    }
}

/// Automatically detects the most probable delimiter in a delimited text sample.
///
/// Tests `,`, `\t`, `;`, and `|`, prioritizing delimiters with consistent column counts across lines.
pub fn sniff_delimiter(sample: &str) -> char {
    let candidates = [',', '\t', ';', '|'];
    let lines: Vec<&str> = sample
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .take(10)
        .collect();

    if lines.is_empty() {
        return ',';
    }

    let mut best_delim = ',';
    let mut best_score: i64 = -1;

    for &candidate in &candidates {
        let mut prev_count: Option<usize> = None;
        let mut consistent = true;
        let mut total_occurrences = 0;

        for line in &lines {
            // Count candidate occurrences outside quotes
            let mut count = 0;
            let mut in_quotes = false;
            let mut chars = line.chars().peekable();

            while let Some(c) = chars.next() {
                if c == '"' {
                    in_quotes = !in_quotes;
                } else if c == candidate && !in_quotes {
                    count += 1;
                }
            }

            if count == 0 {
                consistent = false;
                break;
            }

            total_occurrences += count;
            match prev_count {
                Some(prev) if prev != count => {
                    consistent = false;
                }
                _ => {}
            }
            prev_count = Some(count);
        }

        if total_occurrences > 0 {
            let score = (total_occurrences as i64) + if consistent { 1000 } else { 0 };
            if score > best_score {
                best_score = score;
                best_delim = candidate;
            }
        }
    }

    best_delim
}

#[derive(Debug, Clone)]
struct RawField {
    content: String,
    was_quoted: bool,
}

/// Parses a CSV or TSV string into a universal [`Value`].
///
/// When `options.has_header` is `true`, returns a `Value::Array` of `Value::Object` rows.
/// When `options.has_header` is `false`, returns a `Value::Array` of `Value::Array` rows.
///
/// # Examples
/// ```
/// use babbel_core::csv::{parse_csv, CsvOptions};
///
/// let input = "id,name,active\n1,Alice,true\n2,Bob,false";
/// let value = parse_csv(input, &CsvOptions::default()).unwrap();
/// assert!(value.as_array().is_some());
/// ```
pub fn parse_csv(input: &str, options: &CsvOptions) -> Result<Value, BabbelError> {
    let raw_rows = parse_raw_records(input, options)?;

    if raw_rows.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }

    if options.has_header {
        let headers: Vec<String> = raw_rows[0]
            .iter()
            .enumerate()
            .map(|(idx, field)| {
                let name = field.content.trim();
                if name.is_empty() {
                    format!("column_{}", idx + 1)
                } else {
                    name.to_string()
                }
            })
            .collect();

        let mut rows = Vec::with_capacity(raw_rows.len().saturating_sub(1));

        for raw_row in raw_rows.into_iter().skip(1) {
            // Skip trailing empty record
            if raw_row.len() == 1 && raw_row[0].content.is_empty() && !raw_row[0].was_quoted {
                continue;
            }

            let mut obj_fields = Vec::with_capacity(headers.len());
            for (idx, header) in headers.iter().enumerate() {
                let cell_val = if let Some(field) = raw_row.get(idx) {
                    convert_field(field, options)
                } else {
                    Value::Null
                };
                obj_fields.push((header.clone(), cell_val));
            }
            rows.push(Value::Object(obj_fields));
        }

        Ok(Value::Array(rows))
    } else {
        let mut rows = Vec::with_capacity(raw_rows.len());

        for raw_row in raw_rows {
            if raw_row.len() == 1 && raw_row[0].content.is_empty() && !raw_row[0].was_quoted {
                continue;
            }
            let row_values = raw_row
                .into_iter()
                .map(|f| convert_field(&f, options))
                .collect();
            rows.push(Value::Array(row_values));
        }

        Ok(Value::Array(rows))
    }
}

/// Parses the input into rows of fields, handling RFC 4180 quotes, doubled quotes, and newlines in quotes.
fn parse_raw_records(input: &str, options: &CsvOptions) -> Result<Vec<Vec<RawField>>, BabbelError> {
    let mut rows: Vec<Vec<RawField>> = Vec::new();
    let mut current_row: Vec<RawField> = Vec::new();
    let mut current_field = String::new();
    let mut was_quoted = false;
    let mut in_quotes = false;

    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut idx = 0;
    let mut line = 1;
    let mut col = 1;

    while idx < len {
        let c = chars[idx];

        if in_quotes {
            // Check for escape character or doubled quote
            if let Some(esc) = options.escape {
                if c == esc && idx + 1 < len {
                    let next = chars[idx + 1];
                    current_field.push(next);
                    idx += 2;
                    col += 2;
                    continue;
                }
            }

            if c == options.quote {
                if idx + 1 < len && chars[idx + 1] == options.quote && options.escape.is_none() {
                    // RFC 4180 doubled quote: "" -> "
                    current_field.push(options.quote);
                    idx += 2;
                    col += 2;
                    continue;
                } else {
                    // Closing quote
                    in_quotes = false;
                    idx += 1;
                    col += 1;
                    continue;
                }
            } else {
                current_field.push(c);
                if c == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                idx += 1;
                continue;
            }
        } else {
            // Outside quotes
            if let Some(comment_char) = options.comment {
                if current_row.is_empty() && current_field.is_empty() && c == comment_char {
                    // Skip till end of line
                    while idx < len && chars[idx] != '\n' {
                        idx += 1;
                    }
                    if idx < len && chars[idx] == '\n' {
                        idx += 1;
                        line += 1;
                        col = 1;
                    }
                    continue;
                }
            }

            if c == options.quote && current_field.trim().is_empty() {
                // Opening quote
                in_quotes = true;
                was_quoted = true;
                current_field.clear();
                idx += 1;
                col += 1;
                continue;
            } else if c == options.delimiter {
                // End of field
                let val = if options.trim && !was_quoted {
                    current_field.trim().to_string()
                } else {
                    core::mem::take(&mut current_field)
                };
                current_row.push(RawField {
                    content: val,
                    was_quoted,
                });
                current_field.clear();
                was_quoted = false;
                idx += 1;
                col += 1;
                continue;
            } else if c == '\r' || c == '\n' {
                // End of record
                let val = if options.trim && !was_quoted {
                    current_field.trim().to_string()
                } else {
                    core::mem::take(&mut current_field)
                };
                current_row.push(RawField {
                    content: val,
                    was_quoted,
                });
                current_field.clear();
                was_quoted = false;

                rows.push(core::mem::take(&mut current_row));

                if c == '\r' && idx + 1 < len && chars[idx + 1] == '\n' {
                    idx += 2;
                } else {
                    idx += 1;
                }
                line += 1;
                col = 1;
                continue;
            } else {
                current_field.push(c);
                idx += 1;
                col += 1;
                continue;
            }
        }
    }

    if in_quotes {
        let loc = Location::new(line, col, idx);
        return Err(BabbelError::eof("Unclosed quote in CSV input")
            .with_span(crate::error::Span::new(loc, loc)));
    }

    // Flush final field and record if present
    if !current_field.is_empty() || was_quoted || !current_row.is_empty() {
        let val = if options.trim && !was_quoted {
            current_field.trim().to_string()
        } else {
            current_field
        };
        current_row.push(RawField {
            content: val,
            was_quoted,
        });
        rows.push(current_row);
    }

    Ok(rows)
}

fn convert_field(field: &RawField, options: &CsvOptions) -> Value {
    if field.was_quoted {
        return Value::String(field.content.clone());
    }

    let text = if options.trim {
        field.content.trim()
    } else {
        &field.content
    };

    if !options.infer_types {
        return Value::String(text.to_string());
    }

    if text.is_empty() || text.eq_ignore_ascii_case("null") || text == "~" {
        return Value::Null;
    }

    if text.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if text.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }

    if let Ok(i) = text.parse::<i128>() {
        return Value::Integer(i);
    }

    if (text.contains('.') || text.contains('e') || text.contains('E'))
        && let Ok(f) = text.parse::<f64>()
        && f.is_finite()
    {
        return Value::Float(f);
    }

    Value::String(text.to_string())
}

/// Serializes a [`Value`] to delimited text (CSV / TSV) string.
pub fn emit_csv(value: &Value, options: &CsvOptions) -> Result<String, BabbelError> {
    let mut dest = crate::io::StringDestination::default();
    emit_csv_to(value, options, &mut dest)?;
    Ok(dest.into_string())
}

/// Serializes a [`Value`] to delimited text writing into an [`IDestination`] sink.
pub fn emit_csv_to(
    value: &Value,
    options: &CsvOptions,
    destination: &mut dyn IDestination,
) -> Result<(), BabbelError> {
    match value {
        Value::Array(records) => {
            if records.is_empty() {
                return Ok(());
            }

            // Check if array contains Objects (structured records) or Arrays (flat rows)
            if let Some(Value::Object(_)) = records.first() {
                // Collect unique headers in order of appearance
                let mut headers: Vec<String> = Vec::new();
                for record in records {
                    if let Value::Object(fields) = record {
                        for (k, _) in fields {
                            if !headers.contains(k) {
                                headers.push(k.clone());
                            }
                        }
                    }
                }

                if options.has_header {
                    for (i, header) in headers.iter().enumerate() {
                        if i > 0 {
                            destination.add_byte(options.delimiter as u8);
                        }
                        emit_field(header, options, destination);
                    }
                    destination.add_bytes(options.line_terminator);
                }

                for record in records {
                    if let Value::Object(fields) = record {
                        for (i, header) in headers.iter().enumerate() {
                            if i > 0 {
                                destination.add_byte(options.delimiter as u8);
                            }
                            let cell = fields.iter().find(|(k, _)| k == header).map(|(_, v)| v);
                            match cell {
                                Some(v) => emit_cell_value(v, options, destination),
                                None => {}
                            }
                        }
                        destination.add_bytes(options.line_terminator);
                    }
                }
            } else {
                for record in records {
                    if let Value::Array(cells) = record {
                        for (i, cell) in cells.iter().enumerate() {
                            if i > 0 {
                                destination.add_byte(options.delimiter as u8);
                            }
                            emit_cell_value(cell, options, destination);
                        }
                        destination.add_bytes(options.line_terminator);
                    }
                }
            }
        }
        Value::Object(fields) => {
            if options.has_header {
                for (i, (header, _)) in fields.iter().enumerate() {
                    if i > 0 {
                        destination.add_byte(options.delimiter as u8);
                    }
                    emit_field(header, options, destination);
                }
                destination.add_bytes(options.line_terminator);
            }
            for (i, (_, val)) in fields.iter().enumerate() {
                if i > 0 {
                    destination.add_byte(options.delimiter as u8);
                }
                emit_cell_value(val, options, destination);
            }
            destination.add_bytes(options.line_terminator);
        }
        _ => {
            emit_cell_value(value, options, destination);
            destination.add_bytes(options.line_terminator);
        }
    }

    Ok(())
}

fn emit_cell_value(val: &Value, options: &CsvOptions, destination: &mut dyn IDestination) {
    match val {
        Value::Null => {}
        Value::Bool(b) => destination.add_bytes(if *b { "true" } else { "false" }),
        Value::Integer(i) => destination.add_bytes(&format!("{}", i)),
        Value::Float(f) => destination.add_bytes(&format!("{}", f)),
        Value::String(s) => emit_field(s, options, destination),
        Value::Bytes(_) => emit_field("<bytes>", options, destination),
        Value::Array(_) | Value::Object(_) => emit_field("<complex>", options, destination),
    }
}

fn emit_field(s: &str, options: &CsvOptions, destination: &mut dyn IDestination) {
    let quote_needed = s.contains(options.delimiter)
        || s.contains(options.quote)
        || s.contains('\n')
        || s.contains('\r')
        || s.starts_with(' ')
        || s.ends_with(' ');

    if quote_needed {
        destination.add_byte(options.quote as u8);
        for c in s.chars() {
            if c == options.quote {
                if let Some(esc) = options.escape {
                    destination.add_byte(esc as u8);
                    destination.add_byte(c as u8);
                } else {
                    // Double quote
                    destination.add_byte(options.quote as u8);
                    destination.add_byte(options.quote as u8);
                }
            } else {
                let mut buf = [0u8; 4];
                destination.add_bytes(c.encode_utf8(&mut buf));
            }
        }
        destination.add_byte(options.quote as u8);
    } else {
        destination.add_bytes(s);
    }
}

// ==========================================
// Zero-Allocation Streaming Pull Parser
// ==========================================

/// Zero-allocation streaming CSV pull parser for microcontrollers and embedded systems.
///
/// Iterates over CSV rows without dynamic heap allocation ($O(1)$ stack memory).
#[derive(Debug, Clone)]
pub struct CsvPullParser<'a> {
    input: &'a str,
    pos: usize,
    delimiter: char,
    quote: char,
}

impl<'a> CsvPullParser<'a> {
    /// Creates a new streaming CSV pull parser over a borrowed string slice.
    pub fn new(input: &'a str, options: &CsvOptions) -> Self {
        Self {
            input,
            pos: 0,
            delimiter: options.delimiter,
            quote: options.quote,
        }
    }

    /// Pulls the next record (row) from the CSV stream.
    /// Returns `None` when end of input is reached.
    pub fn next_record(&mut self) -> Option<CsvRecord<'a>> {
        if self.pos >= self.input.len() {
            return None;
        }

        let start = self.pos;
        let bytes = self.input.as_bytes();
        let mut in_quotes = false;

        while self.pos < bytes.len() {
            let b = bytes[self.pos];
            if b == self.quote as u8 {
                in_quotes = !in_quotes;
                self.pos += 1;
            } else if !in_quotes && (b == b'\n' || b == b'\r') {
                let row_slice = &self.input[start..self.pos];
                if b == b'\r' && self.pos + 1 < bytes.len() && bytes[self.pos + 1] == b'\n' {
                    self.pos += 2;
                } else {
                    self.pos += 1;
                }
                return Some(CsvRecord {
                    raw: row_slice,
                    delimiter: self.delimiter,
                    quote: self.quote,
                });
            } else {
                self.pos += 1;
            }
        }

        if start < self.input.len() {
            let row_slice = &self.input[start..self.pos];
            Some(CsvRecord {
                raw: row_slice,
                delimiter: self.delimiter,
                quote: self.quote,
            })
        } else {
            None
        }
    }
}

/// A borrowed CSV record (row) referencing slices in the original input buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsvRecord<'a> {
    raw: &'a str,
    delimiter: char,
    quote: char,
}

impl<'a> CsvRecord<'a> {
    /// Returns the raw unparsed row slice.
    pub fn raw(&self) -> &'a str {
        self.raw
    }

    /// Returns an iterator over the fields in this record without allocating.
    pub fn fields(&self) -> CsvFieldsIter<'a> {
        CsvFieldsIter {
            raw: self.raw,
            pos: 0,
            delimiter: self.delimiter,
            quote: self.quote,
        }
    }
}

/// Zero-allocation iterator over fields within a [`CsvRecord`].
#[derive(Debug, Clone)]
pub struct CsvFieldsIter<'a> {
    raw: &'a str,
    pos: usize,
    delimiter: char,
    quote: char,
}

impl<'a> Iterator for CsvFieldsIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.raw.len() {
            return None;
        }
        if self.pos == self.raw.len() {
            self.pos += 1;
            return Some("");
        }

        let start = self.pos;
        let bytes = self.raw.as_bytes();
        let mut in_quotes = false;

        while self.pos < bytes.len() {
            let b = bytes[self.pos];
            if b == self.quote as u8 {
                in_quotes = !in_quotes;
                self.pos += 1;
            } else if !in_quotes && b == self.delimiter as u8 {
                let field = &self.raw[start..self.pos];
                self.pos += 1; // skip delimiter
                return Some(Self::clean_field(field, self.quote));
            } else {
                self.pos += 1;
            }
        }

        let field = &self.raw[start..self.pos];
        self.pos += 1; // mark EOF
        Some(Self::clean_field(field, self.quote))
    }
}

impl<'a> CsvFieldsIter<'a> {
    fn clean_field(s: &'a str, quote: char) -> &'a str {
        let trimmed = s.trim();
        let quote_b = quote as u8;
        if trimmed.len() >= 2
            && trimmed.as_bytes()[0] == quote_b
            && trimmed.as_bytes()[trimmed.len() - 1] == quote_b
        {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_basic_parsing() {
        let input = "id,name,score\n1,Alice,98.5\n2,Bob,85";
        let val = parse_csv(input, &CsvOptions::default()).unwrap();
        let rows = val.as_array().unwrap();
        assert_eq!(rows.len(), 2);

        if let Value::Object(row1) = &rows[0] {
            assert_eq!(row1[0], ("id".to_string(), Value::Integer(1)));
            assert_eq!(row1[1], ("name".to_string(), Value::String("Alice".into())));
            assert_eq!(row1[2], ("score".to_string(), Value::Float(98.5)));
        } else {
            panic!("Expected object row");
        }
    }

    #[test]
    fn test_csv_rfc4180_quotes_and_newlines() {
        let input = "\"id\",\"description\"\n1,\"Line 1\nLine 2\"\n2,\"Quoted \"\"value\"\"\"";
        let val = parse_csv(input, &CsvOptions::default()).unwrap();
        let rows = val.as_array().unwrap();
        assert_eq!(rows.len(), 2);

        if let Value::Object(row1) = &rows[0] {
            assert_eq!(
                row1[1],
                (
                    "description".to_string(),
                    Value::String("Line 1\nLine 2".into())
                )
            );
        }
        if let Value::Object(row2) = &rows[1] {
            assert_eq!(
                row2[1],
                (
                    "description".to_string(),
                    Value::String("Quoted \"value\"".into())
                )
            );
        }
    }

    #[test]
    fn test_tsv_parsing() {
        let input = "col1\tcol2\nfoo\t123";
        let val = parse_csv(input, &CsvOptions::tsv()).unwrap();
        let rows = val.as_array().unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_sniff_delimiter() {
        let csv_sample = "name,age,city\nJohn,30,New York\nJane,25,Boston";
        assert_eq!(sniff_delimiter(csv_sample), ',');

        let tsv_sample = "name\tage\tcity\nJohn\t30\tNew York\nJane\t25\tBoston";
        assert_eq!(sniff_delimiter(tsv_sample), '\t');

        let semi_sample = "name;age;city\nJohn;30;New York\nJane;25;Boston";
        assert_eq!(sniff_delimiter(semi_sample), ';');
    }

    #[test]
    fn test_emit_csv_roundtrip() {
        let input = "id,name\r\n1,Alice\r\n2,Bob\r\n";
        let val = parse_csv(input, &CsvOptions::default()).unwrap();
        let emitted = emit_csv(&val, &CsvOptions::default()).unwrap();
        assert_eq!(emitted, input);
    }

    #[test]
    fn test_csv_pull_parser() {
        let csv_data = "temp,humidity,sensor\n21.5,45,\"living room\"\n22.0,46,bedroom\n";
        let mut parser = CsvPullParser::new(csv_data, &CsvOptions::default());

        let row1 = parser.next_record().unwrap();
        let fields1: Vec<&str> = row1.fields().collect();
        assert_eq!(fields1, vec!["temp", "humidity", "sensor"]);

        let row2 = parser.next_record().unwrap();
        let fields2: Vec<&str> = row2.fields().collect();
        assert_eq!(fields2, vec!["21.5", "45", "living room"]);

        let row3 = parser.next_record().unwrap();
        let fields3: Vec<&str> = row3.fields().collect();
        assert_eq!(fields3, vec!["22.0", "46", "bedroom"]);

        assert!(parser.next_record().is_none());
    }
}
