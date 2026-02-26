use polars::prelude::*;
use std::collections::HashMap;

/// Converts a well ID to a 0-based index (0-95 for a 96-well plate)
pub fn well_to_index(well: &str) -> Option<usize> {
    let well = well.trim();
    if well.len() < 2 {
        return None;
    }

    let row_char = well.chars().next()?.to_ascii_uppercase();
    let col_str = &well[1..];

    let row = match row_char {
        'A' => 0,
        'B' => 1,
        'C' => 2,
        'D' => 3,
        'E' => 4,
        'F' => 5,
        'G' => 6,
        'H' => 7,
        _ => return None,
    };

    let col: usize = col_str.parse().ok()?;
    if col < 1 || col > 12 {
        return None;
    }

    Some(row * 12 + (col - 1))
}

// converts a 0-based index to a well ID
pub fn index_to_well(idx: usize) -> Option<String> {
    if idx >= 96 {
        return None;
    }
    let row = idx / 12;
    let col = (idx % 12) + 1;
    let row_char = (b'A' + row as u8) as char;
    Some(format!("{}{}", row_char, col))
}

// converts a well ID to a barcode string
pub fn well_to_barcode(well: &str) -> Option<String> {
    let idx = well_to_index(well)?;
    Some(format!("barcode{:02}", idx + 1))
}

// Applies plate map entries to a DataFrame by ADDING new rows
// returns the updated DataFrame with new rows appended
pub fn apply_plate_map_to_dataframe(
    df: DataFrame,
    plate_entries: &HashMap<String, (String, String)>,
) -> Result<DataFrame, String> {
    println!("[plate_map] apply_plate_map_to_dataframe called");
    println!("[plate_map] plate_entries count: {}", plate_entries.len());

    if plate_entries.is_empty() {
        println!("[plate_map] No entries to apply");
        return Ok(df);
    }

    let column_names: Vec<String> = df.get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    println!("[plate_map] DataFrame columns: {:?}", column_names);
    println!("[plate_map] Original DataFrame height: {}", df.height());

    // Sort entries by barcode for consistent ordering
    let mut sorted_entries: Vec<(&String, &(String, String))> = plate_entries.iter().collect();
    sorted_entries.sort_by(|a, b| {
        let barcode_a = &(a.1).1;
        let barcode_b = &(b.1).1;
        barcode_a.cmp(barcode_b)
    });

    // Build new rows
    let num_new_rows = sorted_entries.len();

    // Create vectors for each column
    let mut new_columns: HashMap<String, Vec<String>> = HashMap::new();

    // Initialize all columns with empty strings
    for col_name in &column_names {
        new_columns.insert(col_name.clone(), vec![String::new(); num_new_rows]);
    }

    // Fill in the sample, barcode, and Well values
    for (row_idx, (well_id, (sample, barcode))) in sorted_entries.iter().enumerate() {
        println!("[plate_map] Adding row {}: well='{}', sample='{}', barcode='{}'",
            row_idx, well_id, sample, barcode);

        if let Some(col) = new_columns.get_mut("sample") {
            col[row_idx] = sample.to_string();
        }
        if let Some(col) = new_columns.get_mut("barcode") {
            col[row_idx] = barcode.to_string();
        }
        if let Some(col) = new_columns.get_mut("Well") {
            col[row_idx] = well_id.to_string();
        }
    }

    // Create a new DataFrame from the new rows
    let mut new_series: Vec<Column> = Vec::new();
    for col_name in &column_names {
        if let Some(values) = new_columns.get(col_name) {
            let series = Series::new(PlSmallStr::from_str(col_name), values.clone());
            new_series.push(series.into_column());
        }
    }

    let new_df = DataFrame::new(new_series)
        .map_err(|e| format!("Failed to create new DataFrame: {e}"))?;

    println!("[plate_map] New rows DataFrame height: {}", new_df.height());

    // Concatenate the original DataFrame with the new rows
    let result = if df.height() == 0 {
        println!("[plate_map] Original DataFrame empty, using new rows only");
        new_df
    } else {
        println!("[plate_map] Appending new rows to existing DataFrame");
        df.vstack(&new_df)
            .map_err(|e| format!("Failed to append rows: {e}"))?
    };

    println!("[plate_map] Final DataFrame height: {}", result.height());

    Ok(result)
}
