use polars::prelude::*;
use polars::prelude::NullValues;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::sync::Arc;
use unicode_normalization::UnicodeNormalization;

pub fn detect_delimiter(path: &str) -> io::Result<u8> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut header = String::new();
    if reader.read_line(&mut header)? == 0 {
        // Empty file, default to comma
        return Ok(b',');
    }

    let mut comma = 0usize;
    let mut semi = 0usize;
    let mut tab = 0usize;

    for &b in header.as_bytes() {
        match b {
            b',' => comma += 1,
            b';' => semi += 1,
            b'\t' => tab += 1,
            _ => {}
        }
    }

    let delim = if semi > comma && semi > tab {
        b';'
    } else if tab > comma {
        b'\t'
    } else {
        b','
    };

    Ok(delim)
}

pub fn read_csv_normalized(path: &str) -> Result<(DataFrame, u8), String> {
    let delim = detect_delimiter(path)
        .map_err(|e| format!("Failed to detect delimiter for '{}': {e}", path))?;

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {e}", path))?;

    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content).to_string();

    let mut lines = content.lines();
    let header_line = lines
        .next()
        .ok_or_else(|| format!("CSV file '{}' appears to be empty", path))?;
    let delim_ch = delim as char;
    let headers: Vec<&str> = header_line.split(delim_ch).collect();

    let fields: Vec<(PlSmallStr, DataType)> = headers
        .into_iter()
        .map(|name| (PlSmallStr::from_str(name), DataType::String))
        .collect();

    let schema: Schema = fields.into_iter().collect();
    let schema_ref: SchemaRef = Arc::new(schema);

    let options = CsvReadOptions::default()
        .with_has_header(true)
        .with_schema(Some(schema_ref))
        .map_parse_options(move |po| {
            let po = po
                .with_separator(delim)
                .with_truncate_ragged_lines(true)
                .with_null_values(Some(NullValues::AllColumnsSingle("".into())));

            if delim == b';' {
                po.with_decimal_comma(true)
            } else {
                po
            }
        });

    let cursor = std::io::Cursor::new(content.into_bytes());
    let reader = options.into_reader_with_file_handle(cursor);

    let df = reader
        .finish()
        .map_err(|e| format!("Failed to read CSV '{}': {e}", path))?;

    let df = translate_french_months(df)
        .map_err(|e| format!("Failed to translate French dates in '{}': {e}", path))?;

    Ok((df, delim))
}

/// French to english month pairs
const FRENCH_MONTHS: &[(&str, &str)] = &[
    ("janvier", "Jan"),
    ("janv", "Jan"),
    ("jan", "Jan"),
    ("fevrier", "Feb"),
    ("fevr", "Feb"),
    ("fev", "Feb"),
    ("mars", "Mar"),
    ("mar", "Mar"),
    ("avril", "Apr"),
    ("avr", "Apr"),
    ("mai", "May"),
    ("juin", "Jun"),
    ("juillet", "Jul"),
    ("juil", "Jul"),
    ("jul", "Jul"),
    ("aout", "Aug"),
    ("septembre", "Sep"),
    ("sept", "Sep"),
    ("sep", "Sep"),
    ("octobre", "Oct"),
    ("oct", "Oct"),
    ("novembre", "Nov"),
    ("nov", "Nov"),
    ("decembre", "Dec"),
    ("dec", "Dec"),
];

fn strip_accents(s: &str) -> String {
    s.nfd().filter(|c| !unicode_normalization::char::is_combining_mark(*c)).collect()
}

// Translates months
fn translate_french_months(df: DataFrame) -> Result<DataFrame, PolarsError> {
    let mut df = df;
    let col_names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();

    for col_name in &col_names {
        let col = df.column(col_name)?;
        if col.dtype() != &DataType::String {
            continue;
        }

        let str_col = col.str()?;
        let translated: Vec<Option<String>> = str_col
            .into_iter()
            .map(|opt_val| opt_val.map(|val| translate_french_month_in_value(val)))
            .collect();
        let translated = StringChunked::from_iter(translated.into_iter());

        df.with_column(translated.into_series().with_name(PlSmallStr::from_str(col_name)))?;
    }

    Ok(df)
}

fn translate_french_month_in_value(val: &str) -> String {
    for sep in &["-", "/", " "] {
        let parts: Vec<&str> = val.split(*sep).collect();
        if parts.len() >= 3 {
            let mut changed = false;
            let new_parts: Vec<String> = parts.iter().map(|part| {
                let normalized = strip_accents(&part.to_lowercase());
                let normalized = normalized.trim_end_matches('.');
                for &(fr, en) in FRENCH_MONTHS {
                    if normalized == fr {
                        changed = true;
                        return en.to_string();
                    }
                }
                part.to_string()
            }).collect();

            if changed {
                return new_parts.join(sep);
            }
        }
    }

    val.to_string()
}

#[derive(Debug)]
pub enum SampleBarcodeStatus {
    // All rows have both sample and barcode filled
    Complete,
    // No rows have any sample or barcode data
    Empty,
    // Some rows have data
    Incomplete { missing_rows: Vec<usize> },
}

// Checks the status of sample and barcode
pub fn check_sample_barcode_status(df: &DataFrame) -> PolarsResult<SampleBarcodeStatus> {

    if df.height() == 0 {
        return Ok(SampleBarcodeStatus::Empty);
    }

    let has_sample_col = df.get_column_names().iter().any(|c| c.as_str() == "sample");
    let has_barcode_col = df.get_column_names().iter().any(|c| c.as_str() == "barcode");

    if !has_sample_col || !has_barcode_col {
        return Ok(SampleBarcodeStatus::Empty);
    }

    let sample_col = df.column("sample")?.str()?;
    let barcode_col = df.column("barcode")?.str()?;

    let mut has_any_data = false;
    let mut missing_rows: Vec<usize> = Vec::new();

    for (idx, (sample, barcode)) in sample_col.into_iter().zip(barcode_col.into_iter()).enumerate() {
        let sample_empty = sample.map(|s| s.trim().is_empty()).unwrap_or(true);
        let barcode_empty = barcode.map(|s| s.trim().is_empty()).unwrap_or(true);

        if !sample_empty || !barcode_empty {
            has_any_data = true;
        }

        if sample_empty || barcode_empty {
            missing_rows.push(idx + 1);
        }
    }

    if !has_any_data {
        // All rows are empty
        Ok(SampleBarcodeStatus::Empty)
    } else if missing_rows.is_empty() {
        // All rows are complete
        Ok(SampleBarcodeStatus::Complete)
    } else {
        // Some rows have data, some are incomplete
        Ok(SampleBarcodeStatus::Incomplete { missing_rows })
    }
}
