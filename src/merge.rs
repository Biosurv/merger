use polars::prelude::*;
use regex::Regex;
use std::collections::HashSet;

use crate::template::expected_columns_for_mode;

/// Input parameters for a merge operation
pub struct MergeParams {
    pub mode: String,
    pub action: String,  // "merge" or "update"
    pub overwrite_existing: bool,
    pub run_num: String,
    pub minknow_ver: Option<String>,
    pub pir_ver: String,
    pub seq_date: Option<String>,
    pub fc_id: Option<String>,
    pub fc_uses: String,
    pub fc_pores: Option<String>,
    pub seq_hours: Option<String>,
    pub fasta_date: String,
    pub seq_kit: Option<String>,
    pub rt_date: String,
    pub lab: String,
    pub pos_con: String,
    pub neg_con: String,
    // DDNS-specific
    pub vp1_date: String,
    pub pcr_machine: String,
    pub vp1_pcr_machine: String,
    pub rtpcr_primers: String,
    pub vp1_primers: String,
}

/// Validates date format (yyyy-mm-dd)
pub fn validate_date(val: &str, name: &str) -> Option<String> {
    let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    if !val.is_empty() && !date_regex.is_match(val) {
        Some(format!("Invalid date format for field {}: {}", name, val))
    } else {
        None
    }
}

/// Validates run number format (yyyymmdd_xxx)
pub fn validate_run_number(run_num: &str) -> Option<String> {
    let run_num_regex = Regex::new(r"^\d{8}_\d{3}$").unwrap();
    if !run_num.is_empty() && !run_num_regex.is_match(run_num) {
        Some(format!("Invalid run number format: {run_num} \nExpected yyyymmdd_xxx."))
    } else {
        None
    }
}

/// Renames EpiInfo columns for minION mode
pub fn rename_epiinfo_columns_for_minion(epi_df: &mut DataFrame) -> Result<(), String> {
    let has_col = |df: &DataFrame, name: &str| {
        df.get_column_names().iter().any(|n| n.as_str() == name)
    };

    let rename_pairs = [
        ("DateFinalCellCultureResults", "DateFinalCultureResult"),
        ("DateFinalrRTPCRResults", "DateFinalITDresult"),
        ("FinalITDResult", "ITDResult"),
        ("SequenceName", "SangerSequenceID"),
        ("DateSeqResult", "DateSangerResultGenerated"),
    ];

    for (old, new_) in rename_pairs {
        if has_col(epi_df, old) && !has_col(epi_df, new_) {
            epi_df
                .rename(old, PlSmallStr::from_str(new_))
                .map_err(|e| format!("Failed to rename '{}' → '{}': {e}", old, new_))?;
        }
    }

    Ok(())
}

/// Merges sample_df with epi_df
pub fn merge_with_epiinfo(
    sample_df: DataFrame,
    epi_df: DataFrame,
) -> Result<DataFrame, String> {
    let sample_cols: HashSet<String> = sample_df
        .get_column_names()
        .iter()
        .map(|&s| s.to_string())
        .collect();
    let epi_cols: HashSet<String> = epi_df
        .get_column_names()
        .iter()
        .map(|&s| s.to_string())
        .collect();
    let common_columns: Vec<String> = sample_cols.intersection(&epi_cols).cloned().collect();

    let sample_df = sample_df.drop_many(common_columns);

    let merged = sample_df
        .left_join(&epi_df, ["sample"], ["ICLabID"])
        .map_err(|e| format!("Failed to merge dataframes: {e}"))?;

    // Normalize EPID column
    let df = merged
        .lazy()
        .with_columns([when(col("EPID").is_null())
            .then(col("EpidNumber"))
            .otherwise(col("EPID"))
            .alias("EPID")])
        .collect()
        .map_err(|e| format!("Failed to normalize EPID column: {e}"))?;

    match df.drop("EpidNumber") {
        Ok(df2) => Ok(df2),
        Err(_) => Ok(df),
    }
}

/// Validates that all expected columns are present
pub fn validate_columns(df: &DataFrame, mode: &str) -> Result<(), String> {
    let expected_columns = expected_columns_for_mode(mode);
    let actual_columns: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let missing: Vec<_> = expected_columns
        .iter()
        .filter(|col| !actual_columns.contains(&col.to_string()))
        .cloned()
        .collect();

    if !missing.is_empty() {
        Err(format!(
            "These columns were missing from the samples file for {}: {}\n\nPlease ensure you are using the correct samples.csv template",
            mode,
            missing.join(", ")
        ))
    } else {
        Ok(())
    }
}

/// Converts PCR control selection to Pass/Fail string
pub fn pcr_control_value(selection: &str) -> &'static str {
    match selection {
        "Positive Passed" | "Negative Passed" => "Pass",
        "Positive Failed" | "Negative Failed" => "Fail",
        "Unselected" => "",
        _ => "unknown",
    }
}

/// Helper to fill a column if it's null or empty string
fn fill_if_empty(column: &str, value: &str) -> Expr {
    when(col(column).is_null().or(col(column).eq(lit(""))))
        .then(lit(value))
        .otherwise(col(column))
        .alias(column)
}

/// Conditionally fills a column based on overwrite setting and value
/// - If overwrite is false, returns column as-is
/// - If value is empty, returns column as-is
/// - Otherwise, applies fill_if_empty logic
fn conditional_fill(column: &str, value: &str, overwrite: bool) -> Expr {
    if !overwrite || value.is_empty() {
        col(column).alias(column)
    } else {
        fill_if_empty(column, value)
    }
}

/// Fills run constants into the merged DataFrame
pub fn fill_run_constants(
    merged_df: DataFrame,
    params: &MergeParams,
) -> Result<DataFrame, String> {
    let pos_con = pcr_control_value(&params.pos_con);
    let neg_con = pcr_control_value(&params.neg_con);
    let overwrite = params.overwrite_existing;

    let mut lazy = merged_df
        .clone()
        .lazy()
        .with_columns([
            conditional_fill("RunNumber", &params.run_num, overwrite),
            conditional_fill("MinKNOWSoftwareVersion", params.minknow_ver.as_deref().unwrap_or(""), overwrite),
            conditional_fill("AnalysisPipelineVersion", &params.pir_ver, overwrite),
            conditional_fill("DateSeqRunLoaded", params.seq_date.as_deref().unwrap_or(""), overwrite),
            conditional_fill("FlowCellID", params.fc_id.as_deref().unwrap_or(""), overwrite),
            conditional_fill("FlowCellPriorUses", &params.fc_uses, overwrite),
            conditional_fill("PoresAvilableAtFlowCellCheck", params.fc_pores.as_deref().unwrap_or(""), overwrite),
            conditional_fill("RunHoursDuration", params.seq_hours.as_deref().unwrap_or(""), overwrite),
            conditional_fill("DateFastaGenerated", &params.fasta_date, overwrite),
            conditional_fill("LibraryPreparationKit", params.seq_kit.as_deref().unwrap_or(""), overwrite),
            conditional_fill("DateRTPCR", &params.rt_date, overwrite),
        ]);

    if params.mode == "minION" {
        lazy = lazy.with_columns([
            if overwrite && !pos_con.is_empty() {
                when(col("PositiveControlPCRCheck").cast(DataType::String).is_null()
                    .or(col("PositiveControlPCRCheck").cast(DataType::String).eq(lit(""))))
                    .then(lit(pos_con))
                    .otherwise(col("PositiveControlPCRCheck").cast(DataType::String))
                    .alias("PositiveControlPCRCheck")
            } else {
                col("PositiveControlPCRCheck").cast(DataType::String).alias("PositiveControlPCRCheck")
            },
            if overwrite && !neg_con.is_empty() {
                when(col("NegativeControlPCRheck").cast(DataType::String).is_null()
                    .or(col("NegativeControlPCRheck").cast(DataType::String).eq(lit(""))))
                    .then(lit(neg_con))
                    .otherwise(col("NegativeControlPCRheck").cast(DataType::String))
                    .alias("NegativeControlPCRheck")
            } else {
                col("NegativeControlPCRheck").cast(DataType::String).alias("NegativeControlPCRheck")
            },
            conditional_fill("institute", &params.lab, overwrite),
        ]);
    } else {
        lazy = lazy.with_columns([
            if overwrite && !pos_con.is_empty() {
                when(col("PositiveControlPCRCheck").cast(DataType::String).is_null()
                    .or(col("PositiveControlPCRCheck").cast(DataType::String).eq(lit(""))))
                    .then(lit(pos_con))
                    .otherwise(col("PositiveControlPCRCheck").cast(DataType::String))
                    .alias("PositiveControlPCRCheck")
            } else {
                col("PositiveControlPCRCheck").cast(DataType::String).alias("PositiveControlPCRCheck")
            },
            if overwrite && !neg_con.is_empty() {
                when(col("NegativeControlPCRCheck").cast(DataType::String).is_null()
                    .or(col("NegativeControlPCRCheck").cast(DataType::String).eq(lit(""))))
                    .then(lit(neg_con))
                    .otherwise(col("NegativeControlPCRCheck").cast(DataType::String))
                    .alias("NegativeControlPCRCheck")
            } else {
                col("NegativeControlPCRCheck").cast(DataType::String).alias("NegativeControlPCRCheck")
            },
            conditional_fill("DateVP1PCR", &params.vp1_date, overwrite),
            conditional_fill("RTPCRMachine", &params.pcr_machine, overwrite),
            conditional_fill("VP1PCRMachine", &params.vp1_pcr_machine, overwrite),
            conditional_fill("RTPCRprimers", &params.rtpcr_primers, overwrite),
            conditional_fill("VP1primers", &params.vp1_primers, overwrite),
        ]);
    }

    lazy.collect()
        .map_err(|e| format!("Failed to fill run constants: {:?}", e))
}

/// Selects only the expected columns for the mode
pub fn select_expected_columns(df: DataFrame, mode: &str) -> Result<DataFrame, String> {
    let expected_columns = expected_columns_for_mode(mode);
    df.select(expected_columns)
        .map_err(|e| format!("Failed to select expected columns: {:?}", e))
}

/// Validates input formats for merge operation
pub fn validate_merge_inputs(params: &MergeParams) -> Result<(), String> {
    let mut errors = Vec::new();

    // Validate run number
    if let Some(err) = validate_run_number(&params.run_num) {
        errors.push(err);
    }

    // Validate dates
    let mut date_fields: Vec<(&str, &str)> = vec![
        (&params.rt_date, "RT PCR Date"),
        (&params.fasta_date, "Fasta Generation Date"),
    ];

    // Add optional seq_date if present
    if let Some(ref seq_date) = params.seq_date {
        date_fields.push((seq_date.as_str(), "Sequencing Date"));
    }

    // Add DDNS-specific dates
    if params.mode != "minION" {
        date_fields.push((&params.vp1_date, "VP1 PCR Date"));
    }

    for (val, name) in date_fields {
        if let Some(err) = validate_date(val, name) {
            errors.push(err);
        }
    }

    if !errors.is_empty() {
        errors.push(String::from("Expected yyyy-mm-dd."));
        Err(format!(
            "{}\n\nRefer to the Guide for more information.",
            errors.join("\n\n")
        ))
    } else {
        Ok(())
    }
}
