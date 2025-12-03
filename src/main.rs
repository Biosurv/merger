#![windows_subsystem = "windows"]

slint::slint!(export {AppWindow} from "ui/app.slint";);

use rfd::FileDialog;
use polars::prelude::*;
use slint::SharedString;
use dirs;
use regex::Regex;
use std::fs::File;
use std::io::{self, Read, BufRead, BufReader, Write};
use std::sync::Arc;
use scraper::{Html, Selector};
use serde_json::Value;
use update_checker::UpdateChecker;

/*
    Version: 1.1.1
    Date: 2025-10-30
    Authors: Shean Mobed, Matthew Anderson

    -TO add:
    - French version
    - warning for empty barcode and samples
    - Verify date format from EpiInfo

*/

fn detect_delimiter(path: &str) -> io::Result<u8> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut buf = String::new();
    let mut comma = 0usize;
    let mut semi = 0usize;
    let mut tab = 0usize;

    for _ in 0..50 {
        buf.clear();
        if reader.read_line(&mut buf)? == 0 {
            break;
        }
        for &b in buf.as_bytes() {
            match b {
                b',' => comma += 1,
                b';' => semi += 1,
                b'\t' => tab += 1,
                _ => {}
            }
        }
    }

    let delim = if semi >= comma && semi >= tab && semi > 0 {
        b';'
    } else if tab >= comma && tab > 0 {
        b'\t'
    } else {
        b','
    };

    Ok(delim)
}

fn read_csv_normalized(path: &str) -> Result<DataFrame, String> {

    let delim = detect_delimiter(path)
        .map_err(|e| format!("Failed to detect delimiter for '{}': {e}", path))?;

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {e}", path))?;

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
                .with_truncate_ragged_lines(true);

            if delim == b';' {
                po.with_decimal_comma(true)
            } else {
                po
            }
        });

    let cursor = std::io::Cursor::new(content.into_bytes());
    let reader = options.into_reader_with_file_handle(cursor);

    reader
        .finish()
        .map_err(|e| format!("Failed to read CSV '{}': {e}", path))
}

fn main() {
    let ui = match AppWindow::new() {
        Ok(window) => window,
        Err(e) => {
            eprintln!("Failed to create AppWindow: {:?}", e);
            return;
        }
    };

    // Update checker
    let mut checker = UpdateChecker::new(
        "Biosurv",
        "merger",
        env!("CARGO_PKG_VERSION"),
    )
    .with_settings_namespace("Biosurv", "merger");
    let _ = checker.clear_cache();
    checker.check_prereleases = false;
    checker.min_interval_minutes = 0;
    checker.github_token = std::env::var("GITHUB_TOKEN").ok();

    let ui_weak = ui.as_weak();
    std::thread::spawn(move || {
        match checker.check(false) {
            Ok(Some(info)) => {
                let title_s = String::from("Update available");
                let msg_s = format!(
                    "A new version is available: v{}\nYou are on v{}.\nOpen the release page:\n{}",
                    info.tag,
                    env!("CARGO_PKG_VERSION"),
                    info.html_url
                );
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_info_title(SharedString::from(title_s));
                        ui.set_info_message(SharedString::from(msg_s));
                        ui.set_show_info(1.0);
                    }
                });
            }
            Ok(None) => {}
            Err(err) => {
                eprintln!("Update check failed: {err}");
                let err_s = format!("{err}");
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_info_title(SharedString::from("Update check failed"));
                        ui.set_info_message(SharedString::from(err_s));
                        ui.set_show_info(1.0);
                    }
                });
            }
        }
    });

    // File selection
    {
        let ui_handle = ui.as_weak();

        ui.on_select_file(move |file_type: SharedString| {
            match file_type.as_str() {
                "sample_file" | "epiinfo_file" | "minknow_file" => {
                    if let Some(file_path) = FileDialog::new().pick_file() {
                        let path_str = file_path.to_string_lossy().to_string();
                        if let Some(ui) = ui_handle.upgrade() {
                            match file_type.as_str() {
                                "sample_file" => ui.set_sample_file(SharedString::from(path_str)),
                                "minknow_file" => ui.set_minknow_file(SharedString::from(path_str)),
                                _ => ui.set_epiinfo_file(SharedString::from(path_str)),
                            }
                        }
                    }
                }
                "destination" => {
                    if let Some(dir_path) = FileDialog::new().pick_folder() {
                        let path_str = dir_path.to_string_lossy().to_string();
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_destination(SharedString::from(path_str));
                        }
                    }
                }
                _ => println!("Unknown file type: {}", file_type),
            }
        });
    }

    // Clear
    {
        let ui_handle = ui.as_weak();
        ui.on_clear(move ||{
            let ui = match ui_handle.upgrade() {
                Some(u) => u,
                None => {
                    eprintln!("Failed to upgrade UI handle in CLEAR button");
                    return;
                }
            };

            let empty = SharedString::from("");

            // general
            ui.set_lab(empty.clone());
            ui.set_run_num(empty.clone());
            ui.set_pir_ver(empty.clone());
            ui.set_minknow_ver(empty.clone());

            // dates/seq
            ui.set_rt_date(empty.clone());
            ui.set_vp1_date(empty.clone());
            ui.set_seq_date(empty.clone());
            ui.set_seq_kit(empty.clone());
            ui.set_seq_hours(empty.clone());
            ui.set_fc_id(empty.clone());
            ui.set_fc_pores(empty.clone());
            ui.set_fc_uses(empty.clone());
            ui.set_fasta_date(empty.clone());

            // PCR extras
            ui.set_rtpcr_primers(empty.clone());
            ui.set_pcr_machine(empty.clone());
            ui.set_vp1_pcr_machine(empty.clone());
            ui.set_vp1_primers(empty.clone());

            // files
            ui.set_minknow_file(empty.clone());
            ui.set_sample_file(empty.clone());
            ui.set_epiinfo_file(empty.clone());
            ui.set_destination(empty.clone());
        });
    }

    // Merge / Update
    {
        let ui_handle = ui.as_weak();

        ui.on_merge( move |mode_action:SharedString| {
            if let Some(ui) = ui_handle.upgrade(){

                // presence
                let mut files_present = true;
                let mut epiinfo_missing = false;

                if ui.get_sample_file().is_empty()  {println!("No Sample File Selected"); files_present = false;}
                if ui.get_minknow_file().is_empty() {println!("No MinKNOW File Selected"); files_present = false;}
                if ui.get_epiinfo_file().is_empty() {epiinfo_missing = true;}
                if ui.get_destination().is_empty()  {println!("No Destination Selected"); files_present = false;}

                if !files_present { return; }
                println!("Passed File Check");

                let piranha_path = ui.get_sample_file().to_string();
                let epiinfo_path = ui.get_epiinfo_file().to_string();
                let minknow_path = ui.get_minknow_file().to_string();
                let destination_path = ui.get_destination().to_string();

                // simple extensions
                if !piranha_path.ends_with(".csv") {
                    ui.set_error_title("Invalid Input".into());
                    ui.set_error_message("Sample file selected is not a CSV file. Please change to CSV.".into());
                    ui.set_show_error(1.0);
                    return;
                }
                if !epiinfo_path.ends_with(".csv") && !epiinfo_missing {
                    ui.set_error_title("Invalid Input".into());
                    ui.set_error_message("Epi Info file selected is not a CSV file. Please change to CSV.".into());
                    ui.set_show_error(1.0);
                    return;
                }
                if !minknow_path.ends_with(".html") {
                    ui.set_error_title("Invalid Input".into());
                    ui.set_error_message("File selected is not a HTML file. Please change to HTML.".into());
                    ui.set_show_error(1.0);
                    return;
                }

                // Parse MinKNOW
                let mut html_content = String::new();
                match File::open(&minknow_path) {
                    Ok(mut f) => {
                        if let Err(e) = f.read_to_string(&mut html_content) {
                            ui.set_error_title("File Read Error".into());
                            ui.set_error_message(format!("Failed to read HTML file at '{}': {:?}", minknow_path, e).into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    }
                    Err(e) => {
                        ui.set_error_title("File Open Error".into());
                        ui.set_error_message(format!("Failed to open HTML file at '{}': {:?}", minknow_path, e).into());
                        ui.set_show_error(1.0);
                        return;
                    }
                }

                let document = Html::parse_document(&html_content);
                let script_selector = match Selector::parse("script") {
                    Ok(sel) => sel,
                    Err(e) => {
                        ui.set_error_title("HTML Parse Error".into());
                        ui.set_error_message(format!("Failed to parse script selector: {:?}", e).into());
                        ui.set_show_error(1.0);
                        return;
                    }
                };

                let mut script_tag = None;
                for script in document.select(&script_selector) {
                    let text = script.text().collect::<String>();
                    if text.contains("const reportData=") {
                        script_tag = Some(script);
                        break;
                    }
                }

                if let Some(script) = script_tag {
                    let script_text = script.text().collect::<String>();
                    if let Some(json_str) = script_text
                        .split("const reportData=")
                        .nth(1)
                        .and_then(|s| s.split(';').next())
                        .map(|s| s.trim())
                    {
                        if let Ok(report_data) = serde_json::from_str::<Value>(json_str) {
                            if let Some(software_versions) = report_data.get("software_versions").and_then(|v| v.as_array()) {
                                for version in software_versions {
                                    if version.get("title").and_then(|t| t.as_str()) == Some("MinKNOW") {
                                        let version_str = version.get("value").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                        ui.set_minknow_ver(SharedString::from(version_str));
                                    }
                                }
                            }
                            if let Some(run_config) = report_data.get("run_setup").and_then(|v| v.as_array()) {
                                for config in run_config {
                                    match config.get("title").and_then(|t| t.as_str()) {
                                        Some("Flow cell ID") => {
                                            let flow_cell_id = config.get("value").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                            ui.set_fc_id(SharedString::from(flow_cell_id));
                                        }
                                        Some("Kit type") => {
                                            let kit_type = config.get("value").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                            ui.set_seq_kit(SharedString::from(kit_type));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            if let Some(run_settings) = report_data.get("run_settings").and_then(|v| v.as_array()) {
                                for config in run_settings {
                                    if config.get("title").and_then(|t| t.as_str()) == Some("Run limit") {
                                        let run_time = config.get("value").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                        ui.set_seq_hours(SharedString::from(run_time));
                                    }
                                }
                            }
                            if let Some(json_str) = script_text
                                .split("const reportData=")
                                .nth(1)
                                .and_then(|s| s.split(';').next())
                                .map(|s| s.trim())
                            {
                                if let Ok(report_data) = serde_json::from_str::<Value>(json_str) {
                                    let date = report_data
                                        .get("run_end_time")
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| s.split('T').next())
                                        .unwrap_or("Unknown")
                                        .to_string();
                                    ui.set_seq_date(SharedString::from(date));
                                }
                            }
                            if let Some(series_data) = report_data.get("pore_scan")
                                .and_then(|v| v.get("series_data"))
                                .and_then(|v| v.as_array()) {
                                if let Some(pore_available) = series_data.iter()
                                    .find(|&s| s.get("name").and_then(|n| n.as_str()) == Some("Pore available")) {
                                    if let Some(data) = pore_available.get("data").and_then(|v| v.as_array()) {
                                        if let Some(first_data_pair) = data.get(0) {
                                            if let Some(value) = first_data_pair.get(1).and_then(|v| v.as_i64()) {
                                                let run_pores = Some(value).unwrap_or(0).to_string();
                                                ui.set_fc_pores(SharedString::from(run_pores));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Read CSVs
                let mut sample_df = match read_csv_normalized(&piranha_path) {
                    Ok(df) => df,
                    Err(msg) => {
                        ui.set_error_title("CSV Read Error".into());
                        ui.set_error_message(msg.into());
                        ui.set_show_error(1.0);
                        return;
                    }
                };

                let merged_df = if !epiinfo_missing {
                    let mut epi_df = match read_csv_normalized(&epiinfo_path) {
                        Ok(df) => df,
                        Err(msg) => {
                            ui.set_error_title("CSV Read Error".into());
                            ui.set_error_message(msg.into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    };

                    let has_col = |df: &DataFrame, name: &str| {
                        df.get_column_names().iter().any(|n| n.as_str() == name)
                    };

                    // minION-specific renames from EpiInfo
                    if ui.get_mode().as_str() == "minION" {
                        let rename_pairs = [
                            ("DateFinalCellCultureResults", "DateFinalCultureResult"),
                            ("DateFinalrRTPCRResults", "DateFinalITDresult"),
                            ("FinalITDResult", "ITDResult"),
                            ("SequenceName", "SangerSequenceID"),
                            ("DateSeqResult", "DateSangerResultGenerated")
                        ];

                        for (old, new_) in rename_pairs {
                            if has_col(&epi_df, old) && !has_col(&epi_df, new_) {
                                if let Err(e) = epi_df.rename(old, PlSmallStr::from_str(new_)) {
                                    ui.set_error_title("Epi Info rename error".into());
                                    ui.set_error_message(
                                        format!("Failed to rename '{}' → '{}': {e}", old, new_).into(),
                                    );
                                    ui.set_show_error(1.0);
                                    return;
                                }
                            }
                        }
                    }

                    let sample_cols: std::collections::HashSet<String> =
                        sample_df.get_column_names().iter().map(|&s| s.to_string()).collect();
                    let epi_cols: std::collections::HashSet<String> =
                        epi_df.get_column_names().iter().map(|&s| s.to_string()).collect();
                    let common_columns: Vec<String> =
                        sample_cols.intersection(&epi_cols).cloned().collect();

                    let sample_df = sample_df.drop_many(common_columns);

                    // Join on sample
                    let merged = match sample_df.left_join(&epi_df, ["sample"], ["ICLabID"]) {
                        Ok(df) => df,
                        Err(e) => {
                            ui.set_error_title("Merge Error".into());
                            ui.set_error_message(format!("Failed to merge dataframes: {e}").into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    };

                    match merged
                        .lazy()
                        .with_columns([when(col("EPID").is_null())
                            .then(col("EpidNumber"))
                            .otherwise(col("EPID"))
                            .alias("EPID")])
                        .collect()
                    {
                        Ok(df) => {
                            match df.drop("EpidNumber") {
                                Ok(df2) => df2,
                                Err(_) => df,
                            }
                        }
                        Err(e) => {
                            ui.set_error_title("Post-merge Transform Error".into());
                            ui.set_error_message(format!("Failed to normalize EPID column: {e}").into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    }
                } else {
                    sample_df
                };

                let current_mode = ui.get_mode().to_string();

                // DDNS expected
                let expected_ddns = [
                    "sample","barcode","IsQCRetest","IfRetestOriginalRun","EPID",
                    "SampleType","CaseOrContact","Country","Province","District","StoolCondition",
                    "SpecimenNumber", "SameAliquot", "DateOfOnset","DateStoolCollected","DateStoolReceivedinLab","DateStoolsuspension",
                    "DateRNAextraction","DateRTPCR","RTPCRMachine","RTPCRprimers","DateVP1PCR","VP1PCRMachine",
                    "VP1primers","PositiveControlPCRCheck","NegativeControlPCRCheck",
                    "LibraryPreparationKit","Well","RunNumber","DateSeqRunLoaded","SequencerUsed",
                    "FlowCellVersion","FlowCellID","FlowCellPriorUses","PoresAvilableAtFlowCellCheck",
                    "MinKNOWSoftwareVersion","RunHoursDuration","DateFastaGenerated","AnalysisPipelineVersion","RunQC","DDNSclassification",
                    "SampleQC","SampleQCChecksComplete","QCComments","DateReported"
                ];

                // minION expected
                let expected_minion = [
                    "sample","barcode","IsQCRetest","IfRetestOriginalRun","institute","EPID","CaseOrContact","CountryOfSampleOrigin",
                    "SpecimenNumber","DateOfOnset","DateStoolCollected","DateStoolReceivedinLab","DateStoolsuspension",
                    "DateFinalCultureResult","FlaskNumber",
                    "FinalCellCultureResult","DateFinalITDresult","ITDResult","ITDMixture","DateSangerResultGenerated",
                    "SangerSequenceID","SequencingLab","DelaysInProcessingForSequencing","DetailsOfDelays","IsclassificationQCRetest",
                    "RTPCRcomments","DateRNAExtraction","DateRTPCR","PositiveControlPCRCheck","NegativeControlPCRheck",
                    "LibraryPreparationKit","RunNumber","DateSeqRunLoaded","FlowCellID","FlowCellPriorUses",
                    "PoresAvilableAtFlowCellCheck","MinKNOWSoftwareVersion","RunHoursDuration","DateFastaGenerated",
                    "AnalysisPipelineVersion","RunQC","IsolateClassification","SampleQC","SampleQCChecksComplete","QCComments","DateReported"
                ];

                let expected_columns: Vec<&str> = if current_mode == "minION" {
                    expected_minion.to_vec()
                } else {
                    expected_ddns.to_vec()
                };

                // Validate
                let actual_columns: Vec<String> = merged_df.get_column_names().iter().map(|s| s.to_string()).collect();
                let missing: Vec<_> = expected_columns
                    .iter()
                    .filter(|col| !actual_columns.contains(&col.to_string()))
                    .cloned()
                    .collect();

                if !missing.is_empty() {
                    let missing_text = format!(
                        "These columns were missing from the samples file for {}: {}\n\nPlease ensure you are using the correct samples.csv template",
                        current_mode,
                        missing.join(", ")
                    );
                    ui.set_error_title(SharedString::from("Missing Columns"));
                    ui.set_error_message(SharedString::from(missing_text));
                    ui.set_show_error(1.0);
                    println!("Missing columns: {:?}", missing);
                    return;
                }

                // Merge/Update action
                let action = mode_action.as_str();
                let mut merged_df = if action == "merge" {
                    // PCR Controls
                    let pos_con = match ui.get_pos_con().as_str() {
                        "Positive Passed" => "Pass",
                        "Positive Failed" => "Fail",
                        "Unselected" => "",
                        _ => "unknown",
                    };
                    let neg_con = match ui.get_neg_con().as_str() {
                        "Negative Passed" => "Pass",
                        "Negative Failed" => "Fail",
                        "Unselected" => "",
                        _ => "unknown",
                    };

                    // Run number format
                    let run_num_regex = Regex::new(r"^\d{8}_\d{3}$").unwrap();
                    let run_num = ui.get_run_num();
                    let mut run_num_err = String::from("");
                    if !run_num.is_empty() && !run_num_regex.is_match(run_num.as_str()) {
                        run_num_err = format!("\n\nInvalid run number format: {run_num} \nExpected yyyymmdd_xxx.");
                    }

                    // Dates to validate
                    let rt_date = ui.get_rt_date();
                    let vp1_date = ui.get_vp1_date();
                    let seq_date = ui.get_seq_date();
                    let fasta_date = ui.get_fasta_date();

                    let date_fields: Vec<(&str,&str)> = if current_mode == "minION" {
                        vec![
                            (rt_date.as_str(),"RT PCR Date"),
                            (seq_date.as_str(),"Sequencing Date"),
                            (fasta_date.as_str(),"Fasta Generation Date"),
                        ]
                    } else {
                        vec![
                            (rt_date.as_str(),"RT PCR Date"),
                            (vp1_date.as_str(),"VP1 PCR Date"),
                            (seq_date.as_str(),"Sequencing Date"),
                            (fasta_date.as_str(),"Fasta Generation Date"),
                        ]
                    };

                    let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
                    let mut date_errors = Vec::new();
                    for (val, name) in date_fields {
                        if !val.is_empty() && !date_regex.is_match(val) {
                            date_errors.push(format!("Invalid date format for field {}: {}", name, val));
                        }
                    }

                    if !date_errors.is_empty() || !run_num_err.is_empty() {
                        if !date_errors.is_empty() { date_errors.push(String::from("Expected yyyy-mm-dd.")); }
                        let joint_date_errors = date_errors.join("\n\n");
                        let format_err = format!("\n{joint_date_errors} \n {run_num_err} \n\n Refer to the Guide for more information.");
                        ui.set_error_title("Input Format Error".into());
                        ui.set_error_message(format_err.into());
                        ui.set_show_error(1.0);
                        return;
                    }

                    // Common fills
                    let mut lazy = merged_df.clone().lazy()
                        .with_columns([
                            col("RunNumber").fill_null(lit(ui.get_run_num().as_str())),
                            col("MinKNOWSoftwareVersion").fill_null(lit(ui.get_minknow_ver().as_str())),
                            col("AnalysisPipelineVersion").fill_null(lit(ui.get_pir_ver().as_str())),
                            col("DateSeqRunLoaded").fill_null(lit(ui.get_seq_date().as_str())),
                            col("FlowCellID").fill_null(lit(ui.get_fc_id().as_str())),
                            col("FlowCellPriorUses").fill_null(lit(ui.get_fc_uses().as_str())),
                            col("PoresAvilableAtFlowCellCheck").fill_null(lit(ui.get_fc_pores().as_str())),
                            col("RunHoursDuration").fill_null(lit(ui.get_seq_hours().as_str())),
                            col("DateFastaGenerated").fill_null(lit(ui.get_fasta_date().as_str())),
                            col("LibraryPreparationKit").fill_null(lit(ui.get_seq_kit().as_str())),
                            col("DateRTPCR").fill_null(lit(ui.get_rt_date().as_str())),
                        ]);

                    if current_mode == "minION" {
                        // minION specifics
                        lazy = lazy.with_columns([
                            col("PositiveControlPCRCheck").cast(DataType::String).fill_null(lit(pos_con)),
                            col("NegativeControlPCRheck").cast(DataType::String).fill_null(lit(neg_con)),
                            col("institute").fill_null(lit(ui.get_lab().as_str())),
                        ]);
                    } else {
                        // DDNS specifics
                        lazy = lazy.with_columns([
                            col("PositiveControlPCRCheck").cast(DataType::String).fill_null(lit(pos_con)),
                            col("NegativeControlPCRCheck").cast(DataType::String).fill_null(lit(neg_con)),
                            col("DateVP1PCR").fill_null(lit(ui.get_vp1_date().as_str())),
                            col("RTPCRMachine").fill_null(lit(ui.get_pcr_machine().as_str())),
                            col("VP1PCRMachine").fill_null(lit(ui.get_vp1_pcr_machine().as_str())),
                            col("RTPCRprimers").fill_null(lit(ui.get_rtpcr_primers().as_str())),
                            col("VP1primers").fill_null(lit(ui.get_vp1_primers().as_str())),
                        ]);
                    }

                    let mut df = match lazy.collect() {
                        Ok(df) => df,
                        Err(e) => {
                            ui.set_error_title("Run Constants Error".into());
                            ui.set_error_message(format!("Failed to fill run constants: {:?}", e).into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    };

                    match df.select(expected_columns.clone()) {
                        Ok(df2) => df2,
                        Err(e) => {
                            ui.set_error_title("Select Columns Error".into());
                            ui.set_error_message(format!("Failed to select expected columns after merge: {:?}", e).into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    }
                } else {
                    match merged_df.select(expected_columns.clone()) {
                        Ok(df) => df,
                        Err(e) => {
                            ui.set_error_title("Select Columns Error".into());
                            ui.set_error_message(format!("Failed to select expected columns in update mode: {:?}", e).into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    }
                };

                // Save
                let file_name = format!("{}_merger_output.csv", ui.get_run_num().as_str());
                let file_path = format!("{}/{}", destination_path, file_name);
                let mut file = match std::fs::File::create(&file_path) {
                    Ok(f) => f,
                    Err(e) => {
                        ui.set_error_title("File Create Error".into());
                        ui.set_error_message(format!("Failed to create output file at '{}': {:?}", file_path, e).into());
                        ui.set_show_error(1.0);
                        return;
                    }
                };
                if let Err(e) = CsvWriter::new(&mut file).finish(&mut merged_df) {
                    ui.set_error_title("CSV Write Error".into());
                    ui.set_error_message(format!("Failed to write output CSV: {:?}", e).into());
                    ui.set_show_error(1.0);
                    return;
                }

                // Success
                match mode_action.as_str() {
                    "merge" => {
                        ui.set_info_title("Merge Successful".into());
                        ui.set_info_message(format!("Merged Detailed Run Report saved to destination as {}.", file_name).into());
                        ui.set_show_info(1.0);
                    }
                    "update" => {
                        ui.set_info_title("Update Succesful".into());
                        ui.set_info_message("Updated Detailed Run Report saved to destination.".into());
                        ui.set_show_info(1.0);
                    }
                    _ => {}
                }
            }
        });
    }

    // Template (DDNS + minION)
    let ui_handle = ui.as_weak();
    {
        ui.on_template(move || {
            if let Some(ui) = ui_handle.upgrade() {
                let current_mode = ui.get_mode().to_string();

                let file_name = if current_mode == "minION" {
                    "sample_template_minion.csv"
                } else {
                    "sample_template_ddns.csv"
                };

                let file_path = match dirs::download_dir() {
                    Some(dir) => dir.join(file_name),
                    None => {
                        ui.set_error_title("Directory Error".into());
                        ui.set_error_message("No Downloads folder found.".into());
                        ui.set_show_error(1.0);
                        return;
                    }
                };

                let mut df = if current_mode == "minION" {
                    match df![
                        "sample"                          => Vec::<String>::new(),
                        "barcode"                         => Vec::<String>::new(),
                        "IsQCRetest"                      => Vec::<String>::new(),
                        "IfRetestOriginalRun"             => Vec::<String>::new(),
                        "institute"                       => Vec::<String>::new(),
                        "EPID"                            => Vec::<String>::new(),
                        "CaseOrContact"                   => Vec::<String>::new(),
                        "CountryOfSampleOrigin"           => Vec::<String>::new(),
                        "SpecimenNumber"                  => Vec::<String>::new(),
                        "DateOfOnset"                     => Vec::<String>::new(),
                        "DateStoolCollected"              => Vec::<String>::new(),
                        "DateStoolReceivedinLab"          => Vec::<String>::new(),
                        "DateStoolsuspension"             => Vec::<String>::new(),
                        "TypeofPositiveControl"           => Vec::<String>::new(),
                        "DatePositiveControlreconstituted"=> Vec::<String>::new(),
                        "DateFinalCultureResult"          => Vec::<String>::new(),
                        "FlaskNumber"                     => Vec::<String>::new(),
                        "FinalCellCultureResult"          => Vec::<String>::new(),
                        "DateFinalITDresult"              => Vec::<String>::new(),
                        "ITDResult"                       => Vec::<String>::new(),
                        "ITDMixture"                      => Vec::<String>::new(),
                        "DateSangerResultGenerated"       => Vec::<String>::new(),
                        "SangerSequenceID"                => Vec::<String>::new(),
                        "SequencingLab"                   => Vec::<String>::new(),
                        "DelaysInProcessingForSequencing" => Vec::<String>::new(),
                        "DetailsOfDelays"                 => Vec::<String>::new(),
                        "IsclassificationQCRetest"        => Vec::<String>::new(),
                        "RTPCRcomments"                   => Vec::<String>::new(),
                        "DateRNAExtraction"               => Vec::<String>::new(),
                        "DateRTPCR"                       => Vec::<String>::new(),
                        "PositiveControlPCRCheck"         => Vec::<String>::new(),
                        "NegativeControlPCRheck"          => Vec::<String>::new(),
                        "LibraryPreparationKit"           => Vec::<String>::new(),
                        "RunNumber"                       => Vec::<String>::new(),
                        "DateSeqRunLoaded"                => Vec::<String>::new(),
                        "FlowCellID"                      => Vec::<String>::new(),
                        "FlowCellPriorUses"               => Vec::<String>::new(),
                        "PoresAvilableAtFlowCellCheck"    => Vec::<String>::new(),
                        "MinKNOWSoftwareVersion"          => Vec::<String>::new(),
                        "RunHoursDuration"                => Vec::<String>::new(),
                        "DateFastaGenerated"              => Vec::<String>::new(),
                        "AnalysisPipelineVersion"         => Vec::<String>::new(),
                        "RunQC"                           => Vec::<String>::new(),
                        "IsolateClassification"           => Vec::<String>::new(),
                        "SampleQC"                        => Vec::<String>::new(),
                        "SampleQCChecksComplete"          => Vec::<String>::new(),
                        "QCComments"                      => Vec::<String>::new(),
                        "DateReported"                    => Vec::<String>::new()
                    ] {
                        Ok(df) => df,
                        Err(e) => {
                            ui.set_error_title("Template Error".into());
                            ui.set_error_message(format!("Failed to create minION template DataFrame: {:?}", e).into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    }
                } else {
                    match df![
                        "sample"                        => Vec::<String>::new(),
                        "barcode"                       => Vec::<String>::new(),
                        "IsQCRetest"                    => Vec::<String>::new(),
                        "IfRetestOriginalRun"           => Vec::<String>::new(),
                        "EPID"                          => Vec::<String>::new(),
                        "SampleType"                    => Vec::<String>::new(),
                        "CaseOrContact"                 => Vec::<String>::new(),
                        "Country"                       => Vec::<String>::new(),
                        "Province"                      => Vec::<String>::new(),
                        "District"                      => Vec::<String>::new(),
                        "StoolCondition"                => Vec::<String>::new(),
                        "SpecimenNumber"                => Vec::<String>::new(),
                        "SameAliquot"                   => Vec::<String>::new(),
                        "DateOfOnset"                   => Vec::<String>::new(),
                        "DateStoolCollected"            => Vec::<String>::new(),
                        "DateStoolReceivedinLab"        => Vec::<String>::new(),
                        "DateStoolsuspension"           => Vec::<String>::new(),
                        "DateRNAextraction"             => Vec::<String>::new(),
                        "DateRTPCR"                     => Vec::<String>::new(),
                        "RTPCRMachine"                  => Vec::<String>::new(),
                        "RTPCRprimers"                  => Vec::<String>::new(),
                        "DateVP1PCR"                    => Vec::<String>::new(),
                        "VP1PCRMachine"                 => Vec::<String>::new(),
                        "VP1primers"                    => Vec::<String>::new(),
                        "PositiveControlPCRCheck"       => Vec::<String>::new(),
                        "NegativeControlPCRCheck"       => Vec::<String>::new(),
                        "LibraryPreparationKit"         => Vec::<String>::new(),
                        "Well"                          => Vec::<String>::new(),
                        "RunNumber"                     => Vec::<String>::new(),
                        "DateSeqRunLoaded"              => Vec::<String>::new(),
                        "SequencerUsed"                 => Vec::<String>::new(),
                        "FlowCellVersion"               => Vec::<String>::new(),
                        "FlowCellID"                    => Vec::<String>::new(),
                        "FlowCellPriorUses"             => Vec::<String>::new(),
                        "PoresAvilableAtFlowCellCheck"  => Vec::<String>::new(),
                        "MinKNOWSoftwareVersion"        => Vec::<String>::new(),
                        "RunHoursDuration"              => Vec::<String>::new(),
                        "DateFastaGenerated"            => Vec::<String>::new(),
                        "AnalysisPipelineVersion"       => Vec::<String>::new(),
                        "RunQC"                         => Vec::<String>::new(),
                        "DDNSclassification"            => Vec::<String>::new(),
                        "SampleQC"                      => Vec::<String>::new(),
                        "SampleQCChecksComplete"        => Vec::<String>::new(),
                        "QCComments"                    => Vec::<String>::new(),
                        "DateReported"                  => Vec::<String>::new()
                    ] {
                        Ok(df) => df,
                        Err(e) => {
                            ui.set_error_title("Template Error".into());
                            ui.set_error_message(format!("Failed to create DDNS template DataFrame: {:?}", e).into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    }
                };

                let file = match std::fs::File::create(&file_path) {
                    Ok(f) => f,
                    Err(e) => {
                        ui.set_error_title("File Create Error".into());
                        ui.set_error_message(format!("Failed to create template file at '{}': {:?}", file_path.display(), e).into());
                        ui.set_show_error(1.0);
                        return;
                    }
                };
                if let Err(e) = CsvWriter::new(file).finish(&mut df) {
                    ui.set_error_title("CSV Write Error".into());
                    ui.set_error_message(format!("Failed to write template CSV: {:?}", e).into());
                    ui.set_show_error(1.0);
                    return;
                }

                let mode_label = if current_mode == "minION" { "minION" } else { "DDNS" };

                ui.set_info_title("Template saved".into());
                ui.set_info_message(
                    format!("{} template saved to downloads folder as {}.", mode_label, file_name).into()
                );
                ui.set_show_info(1.0);
            }
        });
    }

    let _ = ui.run();
}