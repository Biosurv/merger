//#![windows_subsystem = "windows"]

slint::slint!(export {AppWindow} from "ui/app.slint";);

use rfd::FileDialog;
use polars::prelude::*;
use slint::SharedString;
use dirs;
use regex::Regex;

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

fn main() {
    let ui = AppWindow::new().unwrap();
    
    // File Selection Function
    {
    let ui_handle = ui.as_weak();
    
    // for testing
    ui.set_sample_file(SharedString::from("C:\\Users\\SheanMobed\\OneDrive - Biosurv International\\Desktop\\samples.csv"));
    ui.set_epiinfo_file(SharedString::from("C:\\Users\\SheanMobed\\Documents\\Coding\\Polio\\epiinfo_master.csv"));
    ui.set_destination(SharedString::from("C:\\Users\\SheanMobed\\OneDrive - Biosurv International\\Desktop"));

    ui.on_select_file(move |file_type: SharedString| {
        match file_type.as_str() {
            "sample_file" | "epiinfo_file" => {
                // File selection dialog for piranha and epi files
                if let Some(file_path) = FileDialog::new().pick_file() {
                    let path_str = file_path.to_string_lossy().to_string();
                    
                    if let Some(ui) = ui_handle.upgrade() {
                        if file_type.as_str() == "sample_file" {
                            ui.set_sample_file(SharedString::from(path_str));
                        } else {
                            ui.set_epiinfo_file(SharedString::from(path_str));
                        }
                    }
                }
            },
            "destination" => {
                // Directory selection dialog for output destination
                if let Some(dir_path) = FileDialog::new().pick_folder() {
                    let path_str = dir_path.to_string_lossy().to_string();
                    
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
        
        ui.on_clear(move ||{
            // initialise ui 
            let ui = ui_handle.unwrap();

            // clear all line edits on slint side
            let empty = format!("");
            ui.set_lab(slint::SharedString::from(empty.clone()));
            ui.set_run_num(slint::SharedString::from(empty.clone()));
            ui.set_pir_ver(slint::SharedString::from(empty.clone()));
            ui.set_mink_ver(slint::SharedString::from(empty.clone()));
            ui.set_rt_date(slint::SharedString::from(empty.clone()));
            ui.set_vp1_date(slint::SharedString::from(empty.clone()));
            ui.set_seq_date(slint::SharedString::from(empty.clone()));
            ui.set_seq_kit(slint::SharedString::from(empty.clone()));
            ui.set_seq_hours(slint::SharedString::from(empty.clone()));
            ui.set_fc_id(slint::SharedString::from(empty.clone()));
            ui.set_fc_pores(slint::SharedString::from(empty.clone()));
            ui.set_fc_uses(slint::SharedString::from(empty.clone()));

            ui.set_sample_file(slint::SharedString::from(empty.clone()));
            ui.set_epiinfo_file(slint::SharedString::from(empty.clone()));
            ui.set_destination(slint::SharedString::from(empty.clone()));
        });
    }; 
    // end of clear button



    // Merging Function
    {
    let ui_handle = ui.as_weak();

    ui.on_merge( move |mode:slint::SharedString| {
        if let Some(ui) = ui_handle.upgrade(){

        // check if no files provided
        let mut files_present = true;
        let mut epiinfo_missing = false;

        if ui.get_sample_file().is_empty()  {println!("No Sample File Selected"); files_present = false;}
        if ui.get_epiinfo_file().is_empty() {epiinfo_missing = true;}
        if ui.get_destination().is_empty()  {println!("No Destination Selected"); files_present = false;}
        
        // proceed to merge if true
        if !files_present {return;}
        println!("Passed File Check");

        // get file path strings
        let piranha_path = ui.get_sample_file().to_string();
        let epiinfo_path = ui.get_epiinfo_file().to_string();
        let destination_path = ui.get_destination().to_string();
        
        // CSV check
        if !piranha_path.ends_with(".csv") {
            ui.set_error_title(slint::SharedString::from("Invalid Input"));
            ui.set_error_message(slint::SharedString::from("Sample file selected is not a CSV file. Please change to CSV."));
            ui.set_show_error(1.0);
            return;
        }

        if !epiinfo_path.ends_with(".csv") {
            ui.set_error_title(slint::SharedString::from("Invalid Input"));
            ui.set_error_message(slint::SharedString::from("Epi Info file selected is not a CSV file. Please change to CSV."));
            ui.set_show_error(1.0);
            return;
        }

        // create piranha df
        let mut sample_df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(piranha_path.into()))
            .unwrap()
            .finish().unwrap();

        println!("{:?}", sample_df);

        // create epiinfo df
        let mut epiinfo_df = None; // set to none in case missing

        if !epiinfo_missing {
            epiinfo_df = Some(
                CsvReadOptions::default()
                    .try_into_reader_with_file_path(Some(epiinfo_path.into()))
                    .unwrap()
                    .finish()
                    .unwrap()
            );
        }

        //  column requirements
        let expected_columns = [
        "barcode", "sample", "EPID", "SequencingLab", "IsQCRetest",
        "IfRetestOriginalRun", "SampleType", "CaseContactOrCommunity",
        "CountryOfSampleOrigin", "StoolCondition", "SpecimenNumber",
        "DateOfOnset", "DateSampleCollected", "DateReceivedinLab", "CultureResult",
        "DateFinalCultureResult", "ITD_Result", "DateFinalITDresult", "SangerSequenceID",
        "DateSangerResultGenerated", "DelaysInProccessingForDDNS", "DetailsOfDelays",
        "DateRNA_Extraction", "DateRTPCR", "RT_PCR_comments", "DateVP1PCR", "PCR_comments",
        "BrandOfPCRMachine", "PositiveControlPCRCheck", "NegativeControlPCRheck",
        "LibraryPreparationKit", "DateSeqRunLoaded", "RunNumber", "FlowCellID",
        "FlowCellPriorUses", "PoresAvilableAtFlowCellCheck", "MinKNOWSoftwareVersion",
        "RunHoursDuration", "Date_Fasta_generated", "RunQC", "SampleQC",
        "SampleQCChecksComplete", "QCComments", "ToReport", "DateReported",
        ];

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
                // rename epi columns to match epi info columns
                let mut sample_df = sample_df.rename("EPID", PlSmallStr::from_str("EpidNumber")).unwrap();
                sample_df = sample_df.rename("CaseContactOrCommunity", PlSmallStr::from_str("CaseOrContact")).unwrap();
                sample_df = sample_df.rename("DateReceivedinLab", PlSmallStr::from_str("DateStoolReceivedinLab")).unwrap();
                sample_df = sample_df.rename("DateSampleCollected", PlSmallStr::from_str("DateStoolCollected")).unwrap();
                sample_df = sample_df.rename("CultureResult", PlSmallStr::from_str("FinalCellCultureResult")).unwrap();
                sample_df = sample_df.rename("DateFinalCultureResult", PlSmallStr::from_str("DateFinalCellCultureResults")).unwrap();
                sample_df = sample_df.rename("ITD_Result", PlSmallStr::from_str("FinalITDResult")).unwrap();

                // finding common columns, and then removing them will allow us to get those columns with data from epi info
                let sample_cols: std::collections::HashSet<String> = sample_df.get_column_names().iter().map(|&s| s.to_string()).collect();
                let epi_cols: std::collections::HashSet<String> = epiinfo_df.get_column_names().iter().map(|&s| s.to_string()).collect();
                let common_columns: Vec<String> = sample_cols.intersection(&epi_cols).cloned().collect();

                let sample_df = sample_df.drop_many(common_columns);                                           

                // Merge by name
                let mut merged_df: DataFrame = sample_df.left_join(&epiinfo_df, ["sample"], ["ICLabID"]).expect("Failed to merge dataframes");

                // revert column naming for expected output
                let mut merged_df = merged_df.rename("EpidNumber", PlSmallStr::from_str("EPID")).unwrap();
                merged_df = merged_df.rename("CaseOrContact", PlSmallStr::from_str("CaseContactOrCommunity")).unwrap();
                merged_df = merged_df.rename("DateStoolReceivedinLab", PlSmallStr::from_str("DateReceivedinLab")).unwrap();
                merged_df = merged_df.rename("DateStoolCollected", PlSmallStr::from_str("DateSampleCollected")).unwrap();
                merged_df = merged_df.rename("FinalCellCultureResult", PlSmallStr::from_str("CultureResult")).unwrap();
                merged_df = merged_df.rename("DateFinalCellCultureResults", PlSmallStr::from_str("DateFinalCultureResult")).unwrap();
                merged_df = merged_df.rename("FinalITDResult", PlSmallStr::from_str("ITD_Result")).unwrap();

                println!("completed epi / sample mode");
                merged_df.select(expected_columns).unwrap()
            }
            None => {
                // Since Epi Info is missing, the columns did not need to be changed nor did a merge occur
                // simply set sample_df to merged_df resulting in empty epi info columns
                println!("completed epi skip mode");
                sample_df.select(expected_columns).unwrap()
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
            let pos_con = match pos_con.as_str() {"true" => "Pass", "false" => "Fail", _ => "unknown",};
            
            let neg_con = ui.get_neg_con().to_string();
            let neg_con = match neg_con.as_str() {"true" => "Pass", "false" => "Fail", _ => "unknown",};

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
                    let date_err = format!("Invalid date format for field {}: {}. Expected yyyy-mm-dd", 
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

            if !date_errors.is_empty() {
                let combined_error = date_errors.join("\n");
                
                ui.set_error_title(slint::SharedString::from("Date Format Error"));
                ui.set_error_message(slint::SharedString::from(combined_error));
                ui.set_show_error(1.0);
                println!("Exit on date format error");
                return;
            }

            // Fill in run constant values
            let merged_df: DataFrame = merged_df.clone().lazy()
                                        .with_columns([
                                            col("SequencingLab").fill_null(lit(ui.get_lab().as_str())),
                                            col("RunNumber").fill_null(lit(ui.get_run_num().as_str())),
                                            col("MinKNOWSoftwareVersion").fill_null(lit(ui.get_mink_ver().as_str())),
                                            col("DateRTPCR").fill_null(lit(ui.get_rt_date().as_str())),
                                            col("DateVP1PCR").fill_null(lit(ui.get_vp1_date().as_str())),
                                            col("BrandOfPCRMachine").fill_null(lit(ui.get_pcr_machine().as_str())),
                                            col("PositiveControlPCRCheck").cast(DataType::String).fill_null(lit(pos_con)),
                                            col("NegativeControlPCRheck").cast(DataType::String).fill_null(lit(neg_con)),
                                            col("LibraryPreparationKit").fill_null(lit(ui.get_seq_kit().as_str())),
                                            col("DateSeqRunLoaded").fill_null(lit(ui.get_seq_date().as_str())),
                                            col("FlowCellID").fill_null(lit(ui.get_fc_id().as_str())),
                                            col("FlowCellPriorUses").fill_null(lit(ui.get_fc_uses().as_str())),
                                            col("PoresAvilableAtFlowCellCheck").fill_null(lit(ui.get_fc_pores().as_str())),
                                            col("RunHoursDuration").fill_null(lit(ui.get_seq_hours().as_str())),
                                            col("Date_Fasta_generated").fill_null(lit(ui.get_fasta_date().as_str())),
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
        let file_path = format!("{}/output.csv", destination_path);
        let mut file = std::fs::File::create(file_path).unwrap();
        CsvWriter::new(&mut file).finish(&mut merged_df).unwrap();

        println!("{:?}", merged_df);
        
        // success message
        match mode {
            "merge" => {
                println!("MERGE MSG");
                ui.set_info_title(slint::SharedString::from("Merge Succesful"));
                ui.set_info_message(slint::SharedString::from("Merged Detailed Run Report saved to destination."));
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
            // file path
            let file_path = dirs::download_dir().expect("No Downloads folder found").join("sample_template.csv");

            let mut df = df![
            "barcode" => Vec::<String>::new(),
            "sample" => Vec::<String>::new(),
            "EPID" => Vec::<String>::new(),
            "SequencingLab" => Vec::<String>::new(),
            "IsQCRetest" => Vec::<String>::new(),
            "IfRetestOriginalRun" => Vec::<String>::new(),
            "SampleType" => Vec::<String>::new(),
            "CaseContactOrCommunity" => Vec::<String>::new(),
            "CountryOfSampleOrigin" => Vec::<String>::new(),
            "StoolCondition" => Vec::<String>::new(),
            "SpecimenNumber" => Vec::<String>::new(),
            "DateOfOnset" => Vec::<String>::new(),
            "DateSampleCollected" => Vec::<String>::new(),
            "DateReceivedinLab" => Vec::<String>::new(),
            "CultureResult" => Vec::<String>::new(),
            "DateFinalCultureResult" => Vec::<String>::new(),
            "ITD_Result" => Vec::<String>::new(),
            "DateFinalITDresult" => Vec::<String>::new(),
            "SangerSequenceID" => Vec::<String>::new(),
            "DateSangerResultGenerated" => Vec::<String>::new(),
            "DelaysInProccessingForDDNS" => Vec::<String>::new(),
            "DetailsOfDelays" => Vec::<String>::new(),
            "DateRNA_Extraction" => Vec::<String>::new(),
            "DateRTPCR" => Vec::<String>::new(),
            "RT_PCR_comments" => Vec::<String>::new(),
            "DateVP1PCR" => Vec::<String>::new(),
            "PCR_comments" => Vec::<String>::new(),
            "BrandOfPCRMachine" => Vec::<String>::new(),
            "PositiveControlPCRCheck" => Vec::<String>::new(),
            "NegativeControlPCRheck" => Vec::<String>::new(),
            "LibraryPreparationKit" => Vec::<String>::new(),
            "DateSeqRunLoaded" => Vec::<String>::new(),
            "RunNumber" => Vec::<String>::new(),
            "FlowCellID" => Vec::<String>::new(),
            "FlowCellPriorUses" => Vec::<String>::new(),
            "PoresAvilableAtFlowCellCheck" => Vec::<String>::new(),
            "MinKNOWSoftwareVersion" => Vec::<String>::new(),
            "RunHoursDuration" => Vec::<String>::new(),
            "Date_Fasta_generated" => Vec::<String>::new(),
            "RunQC" => Vec::<String>::new(),
            "SampleQC" => Vec::<String>::new(),
            "SampleQCChecksComplete" => Vec::<String>::new(),
            "QCComments" => Vec::<String>::new(),
            "ToReport" => Vec::<String>::new(),
            "DateReported" => Vec::<String>::new()
            ].unwrap();

            // output to downloads
            let file = std::fs::File::create(file_path).unwrap();
            CsvWriter::new(file).finish(&mut df).unwrap();

            // success message
            ui.set_info_title(slint::SharedString::from("Template saved"));
            ui.set_info_message(slint::SharedString::from("Template Samples.csv saved to downloads folder"));
            ui.set_show_info(1.0);

        }});
    }
    let _ = ui.run();
}