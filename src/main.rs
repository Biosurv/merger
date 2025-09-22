#![windows_subsystem = "windows"]

slint::slint!(export {AppWindow} from "ui/app.slint";);

use rfd::FileDialog;
use polars::prelude::*;
use slint::SharedString;
use dirs;
use regex::Regex;
//use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use scraper::{Html, Selector};
use serde_json::Value;
<<<<<<< HEAD
=======
use update_checker::UpdateChecker;
>>>>>>> 3e3286f (Updater checker and error handling)
//use std::error::Error;

/*
    Version: 1.0.0
    Date: 2025-02-13
    Description:
    This is the rust version of the merger app. This app will take three files:
    - a barcodes CSV file with two columns sample and barcode.
    - CSV of the the epi database from EpiInfo.
    - MinKNOW report from the sequencing run. 

    In the app, the user can fill in the lab info details that are common to the
    whole run such as run number or pcr dates. The MinKNOW report will extract the
    flowcell id, type, pores, seq kit, device type, runtime, and minknow version.
    Then the app will glue all the info together with the EpiInfo and output a file
    ready for Piranha.

*/

<<<<<<< HEAD

fn main() {
    let ui = AppWindow::new().unwrap();
    
    // File Selection Function
    {
    let ui_handle = ui.as_weak();
    
    // for testing
    ui.set_minknow_file(SharedString::from("C:\\Users\\SheanMobed\\Documents\\Coding\\DDNS_apps\\reports\\20250206_005_report_FBA38845_20250206_1539_74ce1900.html"));
    ui.set_sample_file(SharedString::from("C:\\Users\\SheanMobed\\OneDrive - Biosurv International\\Desktop\\samples.csv"));
    ui.set_epiinfo_file(SharedString::from("C:\\Users\\SheanMobed\\Documents\\Coding\\Polio\\py_scripts\\epiinfo_master.csv"));
    ui.set_destination(SharedString::from("C:\\Users\\SheanMobed\\OneDrive - Biosurv International\\Desktop"));
=======
fn main() {
    let ui = match AppWindow::new() {
        Ok(window) => window,
        Err(e) => {
            eprintln!("Failed to create AppWindow: {:?}", e);
            return;
        }
    };

    
    let mut checker = UpdateChecker::new(
        "Biosurv",       
        "RunReporter",
        env!("CARGO_PKG_VERSION"), // current app version
    )
    .with_settings_namespace("Biosurv", "RunReporter");

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
                        ui.set_info_title(slint::SharedString::from(title_s));
                        ui.set_info_message(slint::SharedString::from(msg_s));
                        ui.set_show_info(1.0);
                    }
                });
            }
            Ok(None) => {
                
            }
            Err(err) => {
                eprintln!("Update check failed: {err}");
                
                let err_s = format!("{err}");
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_info_title(slint::SharedString::from("Update check failed"));
                        ui.set_info_message(slint::SharedString::from(err_s));
                        ui.set_show_info(1.0);
                    }
                });
            }
        }
    });

    // File Selection Function
    {
    let ui_handle = ui.as_weak();

    // for testing
    if let Some(ui) = ui_handle.upgrade() {
        ui.set_minknow_file(SharedString::from("C:\\Users\\MatthewAnderson\\Desktop\\Polio\\examples\\20250206_005_report_FBA38845_20250206_1539_74ce1900.html"));
        ui.set_sample_file(SharedString::from("C:\\Users\\MatthewAnderson\\Desktop\\Polio\\examples\\samples.csv"));
        ui.set_epiinfo_file(SharedString::from("C:\\Users\\MatthewAnderson\\Desktop\\Polio\\examples\\epiinfo_master.csv"));
        ui.set_destination(SharedString::from("C:\\Users\\MatthewAnderson\\Desktop\\Polio\\examples"));
    }

>>>>>>> 3e3286f (Updater checker and error handling)

    ui.on_select_file(move |file_type: SharedString| {
        match file_type.as_str() {
            "sample_file" | "epiinfo_file" | "minknow_file"=> {
<<<<<<< HEAD
                // File selection dialog for piranha and epi files
                if let Some(file_path) = FileDialog::new().pick_file() {
                    let path_str = file_path.to_string_lossy().to_string();
                    
                    if let Some(ui) = ui_handle.upgrade() {
                        if file_type.as_str() == "sample_file" {
                            ui.set_sample_file(SharedString::from(path_str));
                        }
                        else if file_type.as_str() == "minknow_file" {
                            ui.set_minknow_file(SharedString::from(path_str));
                        }  
                        else {
                            ui.set_epiinfo_file(SharedString::from(path_str));
=======
                if let Some(file_path) = FileDialog::new().pick_file() {
                    let path_str = file_path.to_string_lossy().to_string();
                    if let Some(ui) = ui_handle.upgrade() {
                        match file_type.as_str() {
                            "sample_file" => ui.set_sample_file(SharedString::from(path_str)),
                            "minknow_file" => ui.set_minknow_file(SharedString::from(path_str)),
                            _ => ui.set_epiinfo_file(SharedString::from(path_str)),
>>>>>>> 3e3286f (Updater checker and error handling)
                        }
                    }
                }
            },
            "destination" => {
<<<<<<< HEAD
                // Directory selection dialog for output destination
                if let Some(dir_path) = FileDialog::new().pick_folder() {
                    let path_str = dir_path.to_string_lossy().to_string();
                    
=======
                if let Some(dir_path) = FileDialog::new().pick_folder() {
                    let path_str = dir_path.to_string_lossy().to_string();
>>>>>>> 3e3286f (Updater checker and error handling)
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_destination(SharedString::from(path_str));
                    }
                }
            },
            _ => println!("Unknown file type: {}", file_type)
        }
    });
    }
    // end of file selection

    // CLEAR button Section
    {
        let ui_handle = ui.as_weak();
<<<<<<< HEAD
        
        ui.on_clear(move ||{
            // initialise ui 
            let ui = ui_handle.unwrap();
=======
        ui.on_clear(move ||{
            let ui = match ui_handle.upgrade() {
                Some(u) => u,
                None => {
                    eprintln!("Failed to upgrade UI handle in CLEAR button");
                    return;
                }
            };
>>>>>>> 3e3286f (Updater checker and error handling)

            // clear all line edits on slint side
            let empty = format!("");
            ui.set_lab(slint::SharedString::from(empty.clone()));
            ui.set_run_num(slint::SharedString::from(empty.clone()));
            ui.set_pir_ver(slint::SharedString::from(empty.clone()));
            ui.set_minknow_ver(slint::SharedString::from(empty.clone()));
            ui.set_rt_date(slint::SharedString::from(empty.clone()));
            ui.set_vp1_date(slint::SharedString::from(empty.clone()));
            ui.set_seq_date(slint::SharedString::from(empty.clone()));
            ui.set_seq_kit(slint::SharedString::from(empty.clone()));
            ui.set_seq_hours(slint::SharedString::from(empty.clone()));
            ui.set_fc_id(slint::SharedString::from(empty.clone()));
            ui.set_fc_pores(slint::SharedString::from(empty.clone()));
            ui.set_fc_uses(slint::SharedString::from(empty.clone()));

            ui.set_minknow_file(slint::SharedString::from(empty.clone()));
            ui.set_sample_file(slint::SharedString::from(empty.clone()));
            ui.set_epiinfo_file(slint::SharedString::from(empty.clone()));
            ui.set_destination(slint::SharedString::from(empty.clone()));
        });
    }; 
    // end of clear button


<<<<<<< HEAD

=======
>>>>>>> 3e3286f (Updater checker and error handling)
    // Merging Function
    {
    let ui_handle = ui.as_weak();

    ui.on_merge( move |mode:slint::SharedString| {
        if let Some(ui) = ui_handle.upgrade(){

        // check if no files provided
        let mut files_present = true;
        let mut epiinfo_missing = false;

        if ui.get_sample_file().is_empty()  {println!("No Sample File Selected"); files_present = false;}
        if ui.get_minknow_file().is_empty()  {println!("No MinKNOW File Selected"); files_present = false;}
        if ui.get_epiinfo_file().is_empty() {epiinfo_missing = true;}
        if ui.get_destination().is_empty()  {println!("No Destination Selected"); files_present = false;}
        
        // proceed to merge if true
        if !files_present {return;}
        println!("Passed File Check");

        // get file path strings
        let piranha_path = ui.get_sample_file().to_string();
        let epiinfo_path = ui.get_epiinfo_file().to_string();
        let minknow_path = ui.get_minknow_file().to_string();
        let destination_path = ui.get_destination().to_string();
        
        // CSV check
        if !piranha_path.ends_with(".csv") {
            ui.set_error_title(slint::SharedString::from("Invalid Input"));
            ui.set_error_message(slint::SharedString::from("Sample file selected is not a CSV file. Please change to CSV."));
            ui.set_show_error(1.0);
            return;
        }

<<<<<<< HEAD
        if !epiinfo_path.ends_with(".csv") {
=======
        if !epiinfo_path.ends_with(".csv") && !epiinfo_missing {
>>>>>>> 3e3286f (Updater checker and error handling)
            ui.set_error_title(slint::SharedString::from("Invalid Input"));
            ui.set_error_message(slint::SharedString::from("Epi Info file selected is not a CSV file. Please change to CSV."));
            ui.set_show_error(1.0);
            return;
        }

        // HTML Check
        if !minknow_path.ends_with(".html") {
            ui.set_error_title(slint::SharedString::from("Invalid Input"));
            ui.set_error_message(slint::SharedString::from("File selected is not a HTML file. Please change to HTML."));
            ui.set_show_error(1.0);
            return;
        }

        // Read the HTML file
        let mut html_content = String::new();
<<<<<<< HEAD
        let _ = File::open(minknow_path).expect("WRONG FORMAT").read_to_string(&mut html_content);

        let document = Html::parse_document(&html_content);

        let script_selector: Selector = Selector::parse("script").expect("SELECTOR ERR");
=======
        match File::open(&minknow_path) {
            Ok(mut f) => {
                if let Err(e) = f.read_to_string(&mut html_content) {
                    ui.set_error_title(slint::SharedString::from("File Read Error"));
                    ui.set_error_message(slint::SharedString::from(format!("Failed to read HTML file at '{}': {:?}", minknow_path, e)));
                    ui.set_show_error(1.0);
                    return;
                }
            }
            Err(e) => {
                ui.set_error_title(slint::SharedString::from("File Open Error"));
                ui.set_error_message(slint::SharedString::from(format!("Failed to open HTML file at '{}': {:?}", minknow_path, e)));
                ui.set_show_error(1.0);
                return;
            }
        }

        let document = Html::parse_document(&html_content);

        let script_selector = match Selector::parse("script") {
            Ok(sel) => sel,
            Err(e) => {
                ui.set_error_title(slint::SharedString::from("HTML Parse Error"));
                ui.set_error_message(slint::SharedString::from(format!("Failed to parse script selector: {:?}", e)));
                ui.set_show_error(1.0);
                return;
            }
        };
>>>>>>> 3e3286f (Updater checker and error handling)

        // Find the report data
        let mut script_tag = None;
        for script in document.select(&script_selector) {
            let text = script.text().collect::<String>();
            if text.contains("const reportData=") {
                script_tag = Some(script);
                break;
            }
        }

        // Parse REACT JSON if found
        if let Some(script) = script_tag {
            let script_text = script.text().collect::<String>();
            if let Some(json_str) = script_text
                .split("const reportData=")
                .nth(1)
                .and_then(|s| s.split(';').next())
                .map(|s| s.trim()) 
            {
                if let Ok(report_data) = serde_json::from_str::<Value>(json_str) {
                    // Extract MinKNOW version
                    if let Some(software_versions) = report_data.get("software_versions").and_then(|v| v.as_array()) {
                        for version in software_versions {
                            if version.get("title").and_then(|t| t.as_str()) == Some("MinKNOW") {
                                let version_str = version.get("value").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                ui.set_minknow_ver(SharedString::from(version_str));
                            }
                        }
                    }

                    // Extract fc id and kit
                    if let Some(run_config) = report_data.get("run_setup").and_then(|v| v.as_array()) {
                        for config in run_config {
                            match config.get("title").and_then(|t| t.as_str()) {
                                // Some("Flow cell type") => {
                                //     let flow_cell_type = config.get("value").and_then(|v| v.as_str()).map(String::from);
                                //     ui.set_fc
                                // }
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

                    // Extract run hours
                    if let Some(run_config) = report_data.get("run_settings").and_then(|v| v.as_array()) {
                        for config in run_config {
                            match config.get("title").and_then(|t| t.as_str()) {
                                Some("Run limit") => {
                                    let run_time = config.get("value").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                    ui.set_seq_hours(SharedString::from(run_time));
                                }
                                _ => {}
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
                        // Extract run_end_time
                        let date = report_data.get("run_end_time").and_then(|v| v.as_str()).and_then(|s| s.split('T').next()).unwrap_or("Unknown").to_string();
                        ui.set_seq_date(SharedString::from(date));
                    }
                }
                //     if let Some(header) = report_data.get("header") {
                //         let device = header.get("device_type").and_then(|v| v.as_str()).unwrap_or("Unknown");
                //         ui.set_device
                // }
                // Extract the pore data
                    if let Some(series_data) = report_data.get("pore_scan")
                    .and_then(|v| v.get("series_data"))
                    .and_then(|v| v.as_array()) {

                    if let Some(pore_available) = series_data.iter().find(|&s| s.get("name").and_then(|n| n.as_str()) == Some("Pore available")) {
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
    

        // create piranha df
<<<<<<< HEAD
        let mut sample_df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(piranha_path.into()))
            .unwrap()
            .finish().unwrap();
=======
        let mut sample_df = match CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(piranha_path.into())) {
                Ok(reader) => match reader.finish() {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(slint::SharedString::from("CSV Read Error"));
                        ui.set_error_message(slint::SharedString::from(format!("Failed to read sample CSV: {:?}", e)));
                        ui.set_show_error(1.0);
                        return;
                    }
                },
                Err(e) => {
                    ui.set_error_title(slint::SharedString::from("CSV Reader Error"));
                    ui.set_error_message(slint::SharedString::from(format!("Failed to create CSV reader for sample file: {:?}", e)));
                    ui.set_show_error(1.0);
                    return;
                }
            };
>>>>>>> 3e3286f (Updater checker and error handling)

        println!("{:?}", sample_df);

        // create epiinfo df
<<<<<<< HEAD
        let mut epiinfo_df = None; // set to none in case missing

        if !epiinfo_missing {
            epiinfo_df = Some(
                CsvReadOptions::default()
                    .try_into_reader_with_file_path(Some(epiinfo_path.into()))
                    .unwrap()
                    .finish()
                    .unwrap()
            );
=======
        let mut epiinfo_df = None;
        if !epiinfo_missing {
            match CsvReadOptions::default()
                .try_into_reader_with_file_path(Some(epiinfo_path.into())) {
                    Ok(reader) => match reader.finish() {
                        Ok(df) => epiinfo_df = Some(df),
                        Err(e) => {
                            ui.set_error_title(slint::SharedString::from("CSV Read Error"));
                            ui.set_error_message(slint::SharedString::from("Failed to read EpiInfo CSV - This error is likely due to incorrect CSV file encoding. Save the CSV as UTF-8 encoded and try again"));
                            ui.set_show_error(1.0);
                            return;
                        }
                    },
                    Err(e) => {
                        ui.set_error_title(slint::SharedString::from("CSV Reader Error"));
                        ui.set_error_message(slint::SharedString::from(format!("Failed to create CSV reader for EpiInfo file: {:?}", e)));
                        ui.set_show_error(1.0);
                        return;
                    }
                }
>>>>>>> 3e3286f (Updater checker and error handling)
        }

        //  column requirements
        let expected_columns = [
        "sample", "barcode", "IsQCRetest", "IfRetestOriginalRun", "EPID",
        "institute", "SampleType", "CaseOrContact", "Country", "Province", "District", "StoolCondition",
        "SpecimenNumber", "DateOfOnset", "DateStoolCollected", "DateStoolReceivedinLab",
        "DateRNAextraction", "DateRTPCR", "RTPCRMachine", "RTPCRprimers","DateVP1PCR", "VP1PCRMachine",
        "VP1primers", "PositiveControlPCRCheck", "NegativeControlPCRCheck",
        "LibraryPreparationKit", "Well", "RunNumber", "DateSeqRunLoaded", "SequencerUsed", 
        "FlowCellVersion", "FlowCellID", "FlowCellPriorUses", "PoresAvilableAtFlowCellCheck",
        "MinKNOWSoftwareVersion","RunHoursDuration", "DateFastaGenerated", "AnalysisPipelineVersion","RunQC", "Classification",
<<<<<<< HEAD
        "SampleQC", "SampleQCChecksComplete", "QCComments", "DateReported",
        ];

=======
        "SampleQC", "SampleQCChecksComplete", "QCComments", "DateReported"
        ];

        //

>>>>>>> 3e3286f (Updater checker and error handling)
        // Validate headers for the sample DataFrame
        let actual_columns: Vec<String> = sample_df.get_column_names().iter().map(|s| s.to_string()).collect();

        // Check for missing columns
        let missing: Vec<_> = expected_columns
            .iter()
            .filter(|col| !actual_columns.contains(&col.to_string()))
            .cloned()
            .collect();

        if !missing.is_empty() {
            let missing_text = format!("These columns were missing from Samples CSV file: {:?}", missing)
            .replace("\"", "").replace("[", "")
            .replace("]", "");
            ui.set_error_title(slint::SharedString::from("Missing Columns"));
            ui.set_error_message(slint::SharedString::from(missing_text));
            ui.set_show_error(1.0);
            println!("Missing columns: {:?}", missing);
            return;
        }

        // Separating operations depending on if epi info is present
        let merged_df = match epiinfo_df {
            Some(epiinfo_df) => {
                println!("Merging with EPI");
<<<<<<< HEAD
                // rename epi columns to match epi info columns
                let sample_df = sample_df.rename("EPID", PlSmallStr::from_str("EpidNumber")).unwrap();
=======
                let sample_df = match sample_df.rename("EPID", PlSmallStr::from_str("EpidNumber")) {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(slint::SharedString::from("Rename Error"));
                        ui.set_error_message(slint::SharedString::from(format!("Failed to rename EPID column: {:?}", e)));
                        ui.set_show_error(1.0);
                        return;
                    }
                };
>>>>>>> 3e3286f (Updater checker and error handling)
                // sample_df = sample_df.rename("CaseContactOrCommunity", PlSmallStr::from_str("CaseOrContact")).unwrap();
                // sample_df = sample_df.rename("DateReceivedinLab", PlSmallStr::from_str("DateStoolReceivedinLab")).unwrap();
                // sample_df = sample_df.rename("DateSampleCollected", PlSmallStr::from_str("DateStoolCollected")).unwrap();
                // sample_df = sample_df.rename("CultureResult", PlSmallStr::from_str("FinalCellCultureResult")).unwrap();
                // sample_df = sample_df.rename("DateFinalCultureResult", PlSmallStr::from_str("DateFinalCellCultureResults")).unwrap();
                // sample_df = sample_df.rename("ITD_Result", PlSmallStr::from_str("FinalITDResult")).unwrap();

<<<<<<< HEAD
                // finding common columns, and then removing them will allow us to get those columns with data from epi info
=======
>>>>>>> 3e3286f (Updater checker and error handling)
                let sample_cols: std::collections::HashSet<String> = sample_df.get_column_names().iter().map(|&s| s.to_string()).collect();
                let epi_cols: std::collections::HashSet<String> = epiinfo_df.get_column_names().iter().map(|&s| s.to_string()).collect();
                let common_columns: Vec<String> = sample_cols.intersection(&epi_cols).cloned().collect();

<<<<<<< HEAD
                let sample_df = sample_df.drop_many(common_columns);                                           

                // Merge by name
                let mut merged_df: DataFrame = sample_df.left_join(&epiinfo_df, ["sample"], ["ICLabID"]).expect("Failed to merge dataframes");

                // revert column naming for expected output
                let merged_df = merged_df.rename("EpidNumber", PlSmallStr::from_str("EPID")).unwrap();
                // merged_df = merged_df.rename("CaseOrContact", PlSmallStr::from_str("CaseContactOrCommunity")).unwrap();
                // merged_df = merged_df.rename("DateStoolReceivedinLab", PlSmallStr::from_str("DateReceivedinLab")).unwrap();
                // merged_df = merged_df.rename("DateStoolCollected", PlSmallStr::from_str("DateSampleCollected")).unwrap();
                // merged_df = merged_df.rename("FinalCellCultureResult", PlSmallStr::from_str("CultureResult")).unwrap();
                // merged_df = merged_df.rename("DateFinalCellCultureResults", PlSmallStr::from_str("DateFinalCultureResult")).unwrap();
                // merged_df = merged_df.rename("FinalITDResult", PlSmallStr::from_str("ITD_Result")).unwrap();

                println!("completed epi / sample mode");
                merged_df.select(expected_columns).unwrap()
            }
            None => {
                // Since Epi Info is missing, the columns did not need to be changed nor did a merge occur
                // simply set sample_df to merged_df resulting in empty epi info columns
                println!("completed epi skip mode");
                sample_df.select(expected_columns).unwrap()
=======
                let sample_df = sample_df.drop_many(common_columns);

                let mut merged_df = match sample_df.left_join(&epiinfo_df, ["sample"], ["ICLabID"]) {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title("Merge Error".into());
                        ui.set_error_message(format!("Failed to merge dataframes. Reason -> {e}").into());
                        ui.set_show_error(1.0);
                        return;
                    }
                };

                let merged_df = match merged_df.rename("EpidNumber", PlSmallStr::from_str("EPID")) {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(slint::SharedString::from("Rename Error"));
                        ui.set_error_message(slint::SharedString::from(format!("Failed to rename EpidNumber column: {:?}", e)));
                        ui.set_show_error(1.0);
                        return;
                    }
                };

                println!("completed epi / sample mode");
                match merged_df.select(expected_columns) {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(slint::SharedString::from("Select Columns Error"));
                        ui.set_error_message(slint::SharedString::from(format!("Failed to select expected columns: {:?}", e)));
                        ui.set_show_error(1.0);
                        return;
                    }
                }
            }
            None => {
                println!("completed epi skip mode");
                match sample_df.select(expected_columns) {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(slint::SharedString::from("Select Columns Error"));
                        ui.set_error_message(slint::SharedString::from(format!("Failed to select expected columns: {:?}", e)));
                        ui.set_show_error(1.0);
                        return;
                    }
                }
>>>>>>> 3e3286f (Updater checker and error handling)
            }
        };

        println!("After EPI merge");
        println!("{:?}", merged_df);

        // will grab run data if mode is merge
        let mode = mode.as_str();

        let mut merged_df = match mode {
            "merge" => {
            println!("adding run constants");
            // PCR Controls
            let pos_con = ui.get_pos_con().to_string();
            let pos_con = match pos_con.as_str() {"Positive Passed" => "Pass", "Positive Failed" => "Fail", "Unselected" => "", _ => "unknown",};
            
            let neg_con = ui.get_neg_con().to_string();
            let neg_con = match neg_con.as_str() {"Negative Passed" => "Pass", "Negative Failed" => "Fail", "Unselected" => "", _ => "unknown",};

            // Regex for yyyymmdd_XXX format
            let run_num_regex = Regex::new(r"^\d{8}_\d{3}$").unwrap();

            let run_num = ui.get_run_num();

            let mut run_num_err = String::from("");
            
            if !run_num.is_empty() && !run_num_regex.is_match(run_num.as_str()) {
                    run_num_err = format!("\n\nInvalid run number format: {run_num} \nExpected yyyymmdd_xxx.");
                
                    // ui.set_error_title(slint::SharedString::from("Run Number Format Error"));
                    // ui.set_error_message(slint::SharedString::from(run_num_err));
                    // ui.set_show_error(1.0);
                    // println!("Exit on run number format error");
                    // return;
            }

            // Date format check
            let rt_date = ui.get_rt_date();
            let vp1_date = ui.get_vp1_date();
            let seq_date = ui.get_seq_date();
            let fasta_date = ui.get_fasta_date();

            let date_fields = vec![
                rt_date.as_str(),
                vp1_date.as_str(),
                seq_date.as_str(),
                fasta_date.as_str(),
            ];

            // Regex for yyyy-mm-dd format
            let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

            let mut date_errors = Vec::new();

            for (index, date) in date_fields.iter().enumerate() {
                if !date.is_empty() && !date_regex.is_match(date) {
                    let date_err = format!("Invalid date format for field {}: {}", 
                        match index {
                            0 => "RT PCR Date",
                            1 => "VP1 PCR Date",
                            2 => "Sequencing Date",
                            3 => "Fasta Generation Date",
                            _ => "Unknown Date Field"
                        },
                        date
                    );

                    date_errors.push(date_err);

                }
            }

            if !date_errors.is_empty() || !run_num_err.is_empty(){
                if !date_errors.is_empty(){
                        date_errors.push(String::from("Expected yyyy-mm-dd."));
                }

                let joint_date_errors = date_errors.join("\n\n");
                //let combined_error = format!("{joint_date_errors}");
                
                let format_err = format!("\n{joint_date_errors} \n {run_num_err} \n\n Refer to the Guide for more information.");

                ui.set_error_title(slint::SharedString::from("Input Format Error"));
                ui.set_error_message(slint::SharedString::from(format_err));
                ui.set_show_error(1.0);
                println!("Exit on date format error");
                return;
            }

            // Fill in run constant values
<<<<<<< HEAD
            let merged_df: DataFrame = merged_df.clone().lazy()
                                        .with_columns([
                                            col("institute").fill_null(lit(ui.get_lab().as_str())),
                                            col("RunNumber").fill_null(lit(ui.get_run_num().as_str())),
                                            col("MinKNOWSoftwareVersion").fill_null(lit(ui.get_minknow_ver().as_str())),
                                            col("AnalysisPipelineVersion").fill_null(lit(ui.get_pir_ver().as_str())),
                                            col("DateRTPCR").fill_null(lit(ui.get_rt_date().as_str())),
                                            col("DateVP1PCR").fill_null(lit(ui.get_vp1_date().as_str())),
                                            col("RTPCRMachine").fill_null(lit(ui.get_pcr_machine().as_str())),
                                            col("VP1PCRMachine").fill_null(lit(ui.get_pcr_machine().as_str())),
                                            col("PositiveControlPCRCheck").cast(DataType::String).fill_null(lit(pos_con)),
                                            col("NegativeControlPCRCheck").cast(DataType::String).fill_null(lit(neg_con)),
                                            col("LibraryPreparationKit").fill_null(lit(ui.get_seq_kit().as_str())),
                                            col("DateSeqRunLoaded").fill_null(lit(ui.get_seq_date().as_str())),
                                            col("FlowCellID").fill_null(lit(ui.get_fc_id().as_str())),
                                            col("FlowCellPriorUses").fill_null(lit(ui.get_fc_uses().as_str())),
                                            col("PoresAvilableAtFlowCellCheck").fill_null(lit(ui.get_fc_pores().as_str())),
                                            col("RunHoursDuration").fill_null(lit(ui.get_seq_hours().as_str())),
                                            col("DateFastaGenerated").fill_null(lit(ui.get_fasta_date().as_str())),
                                        ]).collect().unwrap();
            println!("{:?}", merged_df);
            merged_df.select(expected_columns).unwrap()
            
            }

            // update mode won't read run constant and change, will leave what was read in
            _ =>{
                println!("UPDATE MODE");
                merged_df.select(expected_columns).unwrap()
            }
        };

        // saving to destionation
        let file_name = format!("{}_merger_output.csv", ui.get_run_num().as_str());
        let file_path = format!("{destination_path}/{file_name}");
        let mut file = std::fs::File::create(file_path).unwrap();
        CsvWriter::new(&mut file).finish(&mut merged_df).unwrap();
=======
            let merged_df: DataFrame = match merged_df.clone().lazy()
                .with_columns([
                    col("institute").fill_null(lit(ui.get_lab().as_str())),
                    col("RunNumber").fill_null(lit(ui.get_run_num().as_str())),
                    col("MinKNOWSoftwareVersion").fill_null(lit(ui.get_minknow_ver().as_str())),
                    col("AnalysisPipelineVersion").fill_null(lit(ui.get_pir_ver().as_str())),
                    col("DateRTPCR").fill_null(lit(ui.get_rt_date().as_str())),
                    col("DateVP1PCR").fill_null(lit(ui.get_vp1_date().as_str())),
                    col("RTPCRMachine").fill_null(lit(ui.get_pcr_machine().as_str())),
                    col("VP1PCRMachine").fill_null(lit(ui.get_pcr_machine().as_str())),
                    col("PositiveControlPCRCheck").cast(DataType::String).fill_null(lit(pos_con)),
                    col("NegativeControlPCRCheck").cast(DataType::String).fill_null(lit(neg_con)),
                    col("LibraryPreparationKit").fill_null(lit(ui.get_seq_kit().as_str())),
                    col("DateSeqRunLoaded").fill_null(lit(ui.get_seq_date().as_str())),
                    col("FlowCellID").fill_null(lit(ui.get_fc_id().as_str())),
                    col("FlowCellPriorUses").fill_null(lit(ui.get_fc_uses().as_str())),
                    col("PoresAvilableAtFlowCellCheck").fill_null(lit(ui.get_fc_pores().as_str())),
                    col("RunHoursDuration").fill_null(lit(ui.get_seq_hours().as_str())),
                    col("DateFastaGenerated").fill_null(lit(ui.get_fasta_date().as_str())),
                ]).collect() {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(slint::SharedString::from("Run Constants Error"));
                        ui.set_error_message(slint::SharedString::from(format!("Failed to fill run constants: {:?}", e)));
                        ui.set_show_error(1.0);
                        return;
                    }
                };
                println!("{:?}", merged_df);
                match merged_df.select(expected_columns) {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(slint::SharedString::from("Select Columns Error"));
                        ui.set_error_message(slint::SharedString::from(format!("Failed to select expected columns after merge: {:?}", e)));
                        ui.set_show_error(1.0);
                        return;
                    }
                }
            }
            _ => {
                println!("UPDATE MODE");
                match merged_df.select(expected_columns) {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(slint::SharedString::from("Select Columns Error"));
                        ui.set_error_message(slint::SharedString::from(format!("Failed to select expected columns in update mode: {:?}", e)));
                        ui.set_show_error(1.0);
                        return;
                    }
                }
            }
        };

        // saving to destination
        let file_name = format!("{}_merger_output.csv", ui.get_run_num().as_str());
        let file_path = format!("{destination_path}/{file_name}");
        let mut file = match std::fs::File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                ui.set_error_title(slint::SharedString::from("File Create Error"));
                ui.set_error_message(slint::SharedString::from(format!("Failed to create output file at '{}': {:?}", file_path, e)));
                ui.set_show_error(1.0);
                return;
            }
        };
        if let Err(e) = CsvWriter::new(&mut file).finish(&mut merged_df) {
            ui.set_error_title(slint::SharedString::from("CSV Write Error"));
            ui.set_error_message(slint::SharedString::from(format!("Failed to write output CSV: {:?}", e)));
            ui.set_show_error(1.0);
            return;
        }
>>>>>>> 3e3286f (Updater checker and error handling)

        println!("{:?}", merged_df);
        
        // success message
        match mode {
            "merge" => {
                println!("MERGE MSG");
<<<<<<< HEAD
                ui.set_info_title(slint::SharedString::from("Merge Succesful"));
=======
                ui.set_info_title(slint::SharedString::from("Merge Successful"));
>>>>>>> 3e3286f (Updater checker and error handling)
                ui.set_info_message(slint::SharedString::from(format!("Merged Detailed Run Report saved to destination as {file_name}.")));
                ui.set_show_info(1.0);
            }

            "update" => {
                println!("UPDATE MSG");
                ui.set_info_title(slint::SharedString::from("Update Succesful"));
                ui.set_info_message(slint::SharedString::from("Updated Detailed Run Report saved to destination."));
                ui.set_show_info(1.0);
            }

            _ => {}
        }

        println!("END OF MERGE FUNCTION");
    }});

    }
    // end of merging function

    // start of template function
    let ui_handle = ui.as_weak(); 
    {
        ui.on_template( move || {
            if let Some(ui) = ui_handle.upgrade(){
<<<<<<< HEAD
            // file path
            let file_path = dirs::download_dir().expect("No Downloads folder found").join("sample_template.csv");

            let mut df = df![
            "sample" => Vec::<String>::new(),
            "barcode" => Vec::<String>::new(),
            "IsQCRetest" => Vec::<String>::new(),
            "IfRetestOriginalRun" => Vec::<String>::new(),
            "EPID" => Vec::<String>::new(),
            "institute" => Vec::<String>::new(),
            "SampleType" => Vec::<String>::new(),
            "CaseOrContact" => Vec::<String>::new(),
            "Country" => Vec::<String>::new(),
            "Province" => Vec::<String>::new(),
            "District" => Vec::<String>::new(),
            "StoolCondition" => Vec::<String>::new(),
            "SpecimenNumber" => Vec::<String>::new(),
            "DateOfOnset" => Vec::<String>::new(),
            "DateStoolCollected" => Vec::<String>::new(),
            "DateStoolReceivedinLab" => Vec::<String>::new(),
            "DateRNAextraction" => Vec::<String>::new(),
            "DateRTPCR" => Vec::<String>::new(),
            "RTPCRMachine" => Vec::<String>::new(),
            "RTPCRprimers" => Vec::<String>::new(),
            "DateVP1PCR" => Vec::<String>::new(),
            "VP1PCRMachine" => Vec::<String>::new(),
            "VP1primers" => Vec::<String>::new(),
            "PositiveControlPCRCheck" => Vec::<String>::new(),
            "NegativeControlPCRCheck" => Vec::<String>::new(),
            "LibraryPreparationKit" => Vec::<String>::new(),
            "Well" => Vec::<String>::new(),
            "RunNumber" => Vec::<String>::new(),
            "DateSeqRunLoaded" => Vec::<String>::new(),
            "SequencerUsed" => Vec::<String>::new(),
            "FlowCellVersion" => Vec::<String>::new(),
            "FlowCellID" => Vec::<String>::new(),
            "FlowCellPriorUses" => Vec::<String>::new(),
            "PoresAvilableAtFlowCellCheck" => Vec::<String>::new(),
            "MinKNOWSoftwareVersion" => Vec::<String>::new(),
            "RunHoursDuration" => Vec::<String>::new(),
            "DateFastaGenerated" => Vec::<String>::new(),
            "AnalysisPipelineVersion" => Vec::<String>::new(),
            "RunQC" => Vec::<String>::new(),
            "Classification" => Vec::<String>::new(),
            "SampleQC" => Vec::<String>::new(),
            "SampleQCChecksComplete" => Vec::<String>::new(),
            "QCComments" => Vec::<String>::new(),
            "DateReported" => Vec::<String>::new()
            ].unwrap();

            // output to downloads
            let file = std::fs::File::create(file_path).unwrap();
            CsvWriter::new(file).finish(&mut df).unwrap();
=======
            let file_path = match dirs::download_dir() {
                Some(dir) => dir.join("sample_template.csv"),
                None => {
                    ui.set_error_title(slint::SharedString::from("Directory Error"));
                    ui.set_error_message(slint::SharedString::from("No Downloads folder found."));
                    ui.set_show_error(1.0);
                    return;
                }
            };

            let mut df = match df![
                "sample" => Vec::<String>::new(),
                "barcode" => Vec::<String>::new(),
                "IsQCRetest" => Vec::<String>::new(),
                "IfRetestOriginalRun" => Vec::<String>::new(),
                "EPID" => Vec::<String>::new(),
                "institute" => Vec::<String>::new(),
                "SampleType" => Vec::<String>::new(),
                "CaseOrContact" => Vec::<String>::new(),
                "Country" => Vec::<String>::new(),
                "Province" => Vec::<String>::new(),
                "District" => Vec::<String>::new(),
                "StoolCondition" => Vec::<String>::new(),
                "SpecimenNumber" => Vec::<String>::new(),
                "DateOfOnset" => Vec::<String>::new(),
                "DateStoolCollected" => Vec::<String>::new(),
                "DateStoolReceivedinLab" => Vec::<String>::new(),
                "DateRNAextraction" => Vec::<String>::new(),
                "DateRTPCR" => Vec::<String>::new(),
                "RTPCRMachine" => Vec::<String>::new(),
                "RTPCRprimers" => Vec::<String>::new(),
                "DateVP1PCR" => Vec::<String>::new(),
                "VP1PCRMachine" => Vec::<String>::new(),
                "VP1primers" => Vec::<String>::new(),
                "PositiveControlPCRCheck" => Vec::<String>::new(),
                "NegativeControlPCRCheck" => Vec::<String>::new(),
                "LibraryPreparationKit" => Vec::<String>::new(),
                "Well" => Vec::<String>::new(),
                "RunNumber" => Vec::<String>::new(),
                "DateSeqRunLoaded" => Vec::<String>::new(),
                "SequencerUsed" => Vec::<String>::new(),
                "FlowCellVersion" => Vec::<String>::new(),
                "FlowCellID" => Vec::<String>::new(),
                "FlowCellPriorUses" => Vec::<String>::new(),
                "PoresAvilableAtFlowCellCheck" => Vec::<String>::new(),
                "MinKNOWSoftwareVersion" => Vec::<String>::new(),
                "RunHoursDuration" => Vec::<String>::new(),
                "DateFastaGenerated" => Vec::<String>::new(),
                "AnalysisPipelineVersion" => Vec::<String>::new(),
                "RunQC" => Vec::<String>::new(),
                "Classification" => Vec::<String>::new(),
                "SampleQC" => Vec::<String>::new(),
                "SampleQCChecksComplete" => Vec::<String>::new(),
                "QCComments" => Vec::<String>::new(),
                "DateReported" => Vec::<String>::new()
            ] {
                Ok(df) => df,
                Err(e) => {
                    ui.set_error_title(slint::SharedString::from("Template Error"));
                    ui.set_error_message(slint::SharedString::from(format!("Failed to create template DataFrame: {:?}", e)));
                    ui.set_show_error(1.0);
                    return;
                }
            };

            let file = match std::fs::File::create(&file_path) {
                Ok(f) => f,
                Err(e) => {
                    ui.set_error_title(slint::SharedString::from("File Create Error"));
                    ui.set_error_message(slint::SharedString::from(format!("Failed to create template file at '{}': {:?}", file_path.display(), e)));
                    ui.set_show_error(1.0);
                    return;
                }
            };
            if let Err(e) = CsvWriter::new(file).finish(&mut df) {
                ui.set_error_title(slint::SharedString::from("CSV Write Error"));
                ui.set_error_message(slint::SharedString::from(format!("Failed to write template CSV: {:?}", e)));
                ui.set_show_error(1.0);
                return;
            }
>>>>>>> 3e3286f (Updater checker and error handling)

            // success message
            ui.set_info_title(slint::SharedString::from("Template saved"));
            ui.set_info_message(slint::SharedString::from("Template Samples.csv saved to downloads folder"));
            ui.set_show_info(1.0);

        }});
    }
    let _ = ui.run();
}