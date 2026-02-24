#![windows_subsystem = "windows"]

slint::slint!(export {AppWindow, PlateMapWindow} from "ui/app.slint";);

mod csv;
mod handlers;
mod merge;
mod minknow;
mod plate_map;
mod template;
mod types;

use polars::prelude::*;
use slint::SharedString;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use update_checker::UpdateChecker;

use crate::csv::{read_csv_normalized, check_sample_barcode_status, SampleBarcodeStatus};
use crate::handlers::{setup_clear_handler, setup_file_handlers, setup_plate_map_handlers, setup_standalone_plate_map_handler};
use crate::merge::{
    fill_run_constants, merge_with_epiinfo, rename_epiinfo_columns_for_minion,
    select_expected_columns, validate_columns, validate_merge_inputs, MergeParams,
};
use crate::minknow::parse_minknow_html;
use crate::template::create_template_for_mode;
use crate::types::PendingMerge;

/*
    Version: 1.2.0
    Date: 2026-02-11
    Authors: Matthew Anderson, Shean Mobed

    -TO add:
    - Verify date format from EpiInfo

*/

fn main() {
    let ui = match AppWindow::new() {
        Ok(window) => window,
        Err(e) => {
            eprintln!("Failed to create AppWindow: {:?}", e);
            return;
        }
    };

    let pending_merge: Rc<RefCell<Option<PendingMerge>>> = Rc::new(RefCell::new(None));
    let plate_map_window: Rc<RefCell<Option<PlateMapWindow>>> = Rc::new(RefCell::new(None));
    let plate_entries: Rc<RefCell<HashMap<String, (String, String)>>> =
        Rc::new(RefCell::new(HashMap::new()));

    // Standalone plate map (for "Plate Map" button)
    let standalone_plate_map_window: Rc<RefCell<Option<PlateMapWindow>>> = Rc::new(RefCell::new(None));
    let standalone_plate_entries: Rc<RefCell<HashMap<String, (String, String)>>> =
        Rc::new(RefCell::new(HashMap::new()));

    // Update checker
    setup_update_checker(&ui);

    // Setup handlers from modules
    setup_file_handlers(&ui);
    setup_clear_handler(&ui);
    setup_plate_map_handlers(
        &ui,
        pending_merge.clone(),
        plate_map_window.clone(),
        plate_entries.clone(),
    );

    // Merge / Update handler
    setup_merge_handler(&ui, pending_merge.clone());

    // Template handler
    setup_template_handler(&ui);

    // Standalone Plate Map handler
    setup_standalone_plate_map_handler(
        &ui,
        standalone_plate_map_window,
        standalone_plate_entries,
    );

    let _ = ui.run();
}

fn setup_update_checker(ui: &AppWindow) {
    let mut checker = UpdateChecker::new("Biosurv", "merger", env!("CARGO_PKG_VERSION"))
        .with_settings_namespace("Biosurv", "merger");
    let _ = checker.clear_cache();
    checker.check_prereleases = false;
    checker.min_interval_minutes = 0;
    checker.github_token = std::env::var("GITHUB_TOKEN").ok();

    let ui_weak = ui.as_weak();
    std::thread::spawn(move || {
        match checker.check(false) {
            Ok(Some(info)) => {
                let tag = info.tag.clone();
                let url = info.html_url.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let fr = ui.get_is_french();
                        ui.set_info_title(SharedString::from(
                            if fr { "Mise à jour disponible" } else { "Update available" }
                        ));
                        ui.set_info_message(SharedString::from(if fr {
                            format!(
                                "Une nouvelle version est disponible : v{}\nVous utilisez v{}.\nOuvrir la page de publication :\n{}",
                                tag, env!("CARGO_PKG_VERSION"), url
                            )
                        } else {
                            format!(
                                "A new version is available: v{}\nYou are on v{}.\nOpen the release page:\n{}",
                                tag, env!("CARGO_PKG_VERSION"), url
                            )
                        }));
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
                        let fr = ui.get_is_french();
                        ui.set_info_title(SharedString::from(
                            if fr { "Échec de la vérification de mise à jour" } else { "Update check failed" }
                        ));
                        ui.set_info_message(SharedString::from(err_s));
                        ui.set_show_info(1.0);
                    }
                });
            }
        }
    });
}

fn setup_merge_handler(ui: &AppWindow, pending_merge: Rc<RefCell<Option<PendingMerge>>>) {
    let ui_handle = ui.as_weak();

    ui.on_merge(move |mode_action: SharedString| {
        if let Some(ui) = ui_handle.upgrade() {
            let fr = ui.get_is_french();

            // File presence check
            let mut files_present = true;
            let mut epiinfo_missing = false;

            if ui.get_sample_file().is_empty() {
                println!("No Sample File Selected");
                files_present = false;
            }
            if ui.get_epiinfo_file().is_empty() {
                epiinfo_missing = true;
            }
            let minknow_missing = ui.get_minknow_file().is_empty();

            // At least one optional file must be present
            if epiinfo_missing && minknow_missing {
                ui.set_error_title(if fr { "Fichiers d'entrée manquants" } else { "Missing Input Files" }.into());
                ui.set_error_message(if fr {
                    "Au moins un fichier optionnel est requis.\n\n\
                     Veuillez fournir soit :\n\
                     - Fichier HTML MinKNOW\n\
                     - Fichier CSV Epi Info\n\
                     - Les deux fichiers".into()
                } else {
                    "At least one optional file is required.\n\n\
                     Please provide either:\n\
                     - MinKNOW HTML file\n\
                     - Epi Info CSV file\n\
                     - Both files".into()
                });
                ui.set_show_error(1.0);
                return;
            }

            if ui.get_destination().is_empty() {
                println!("No Destination Selected");
                files_present = false;
            }

            if !files_present {
                return;
            }
            println!("Passed File Check");

            let piranha_path = ui.get_sample_file().to_string();
            let epiinfo_path = ui.get_epiinfo_file().to_string();
            let minknow_path = ui.get_minknow_file().to_string();
            let destination_path = ui.get_destination().to_string();

            // Extension validation
            if !piranha_path.ends_with(".csv") {
                ui.set_error_title(if fr { "Entrée invalide" } else { "Invalid Input" }.into());
                ui.set_error_message(if fr {
                    "Le fichier d'échantillons sélectionné n'est pas un fichier CSV. Veuillez changer en CSV.".into()
                } else {
                    "Sample file selected is not a CSV file. Please change to CSV.".into()
                });
                ui.set_show_error(1.0);
                return;
            }
            if !epiinfo_path.ends_with(".csv") && !epiinfo_missing {
                ui.set_error_title(if fr { "Entrée invalide" } else { "Invalid Input" }.into());
                ui.set_error_message(if fr {
                    "Le fichier Epi Info sélectionné n'est pas un fichier CSV. Veuillez changer en CSV.".into()
                } else {
                    "Epi Info file selected is not a CSV file. Please change to CSV.".into()
                });
                ui.set_show_error(1.0);
                return;
            }
            if !minknow_missing && !minknow_path.ends_with(".html") {
                ui.set_error_title(if fr { "Entrée invalide" } else { "Invalid Input" }.into());
                ui.set_error_message(if fr {
                    "Le fichier sélectionné n'est pas un fichier HTML. Veuillez changer en HTML.".into()
                } else {
                    "File selected is not a HTML file. Please change to HTML.".into()
                });
                ui.set_show_error(1.0);
                return;
            }

            // Parse MinKNOW HTML
            let minknow_data = if !minknow_missing {
                match parse_minknow_html(&minknow_path) {
                    Ok(data) => {
                        ui.set_minknow_ver(SharedString::from(data.minknow_ver.clone()));
                        ui.set_fc_id(SharedString::from(data.fc_id.clone()));
                        ui.set_seq_kit(SharedString::from(data.seq_kit.clone()));
                        ui.set_seq_hours(SharedString::from(data.seq_hours.clone()));
                        ui.set_seq_date(SharedString::from(data.seq_date.clone()));
                        ui.set_fc_pores(SharedString::from(data.fc_pores.clone()));
                        Some(data)
                    }
                    Err(e) => {
                        ui.set_error_title(if fr { "Erreur d'analyse HTML" } else { "HTML Parse Error" }.into());
                        ui.set_error_message(e.into());
                        ui.set_show_error(1.0);
                        return;
                    }
                }
            } else {
                None
            };

            // Read sample CSV
            let (sample_df, mut delim) = match read_csv_normalized(&piranha_path) {
                Ok((df, delim)) => (df, delim),
                Err(msg) => {
                    ui.set_error_title(if fr { "Erreur de lecture CSV" } else { "CSV Read Error" }.into());
                    ui.set_error_message(msg.into());
                    ui.set_show_error(1.0);
                    return;
                }
            };

            // Check sample/barcode status
            let sample_status = match check_sample_barcode_status(&sample_df) {
                Ok(status) => status,
                Err(e) => {
                    ui.set_error_title(if fr { "Erreur de vérification des échantillons" } else { "Samples Check Error" }.into());
                    ui.set_error_message(
                        (if fr {
                            format!("Échec de la vérification des colonnes échantillon/code-barres : {e}")
                        } else {
                            format!("Failed to check sample/barcode columns: {e}")
                        }).into(),
                    );
                    ui.set_show_error(1.0);
                    return;
                }
            };

            match sample_status {
                SampleBarcodeStatus::Empty => {
                    // Empty template - show plate map
                    *pending_merge.borrow_mut() = Some(PendingMerge {
                        mode_action: mode_action.to_string(),
                        piranha_path: piranha_path.clone(),
                        epiinfo_path: epiinfo_path.clone(),
                        minknow_path: minknow_path.clone(),
                        destination_path: destination_path.clone(),
                    });

                    ui.set_missing_plate_prompt_message(if fr {
                        "Vous avez fourni un fichier modèle vide. Souhaitez-vous ajouter des échantillons et des codes-barres à l'aide de la carte de plaque ?"
                    } else {
                        "You have provided an empty template file. Would you like to add samples and barcodes using the plate map?"
                    }.into());
                    ui.set_show_missing_plate_prompt(1.0);
                    return;
                }
                SampleBarcodeStatus::Incomplete { missing_rows } => {
                    // Some rows incomplete - show error
                    let row_list = if missing_rows.len() <= 10 {
                        missing_rows.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", ")
                    } else {
                        format!("{}, ... and {} more",
                            missing_rows[..10].iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", "),
                            missing_rows.len() - 10
                        )
                    };
                    ui.set_error_title(if fr { "Données d'échantillon incomplètes" } else { "Incomplete Sample Data" }.into());
                    ui.set_error_message(if fr {
                        format!("Les lignes suivantes manquent de données d'échantillon ou de code-barres : {}\n\nVeuillez compléter le fichier CSV avant de fusionner.", row_list)
                    } else {
                        format!("The following rows are missing sample or barcode data: {}\n\nPlease complete the CSV file before merging.", row_list)
                    }.into());
                    ui.set_show_error(1.0);
                    return;
                }
                SampleBarcodeStatus::Complete => {
                    // All good, continue with merge
                }
            }

            // Merge with EpiInfo if present
            let current_mode = ui.get_mode().to_string();
            let merged_df = if !epiinfo_missing {
                let (mut epi_df, epi_delim) = match read_csv_normalized(&epiinfo_path) {
                    Ok((df, d)) => (df, d),
                    Err(msg) => {
                        ui.set_error_title(if fr { "Erreur de lecture CSV" } else { "CSV Read Error" }.into());
                        ui.set_error_message(msg.into());
                        ui.set_show_error(1.0);
                        return;
                    }
                };

                delim = epi_delim;

                if current_mode == "minION" {
                    if let Err(e) = rename_epiinfo_columns_for_minion(&mut epi_df) {
                        ui.set_error_title(if fr { "Erreur de renommage Epi Info" } else { "Epi Info rename error" }.into());
                        ui.set_error_message(e.into());
                        ui.set_show_error(1.0);
                        return;
                    }
                }

                match merge_with_epiinfo(sample_df, epi_df) {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(if fr { "Erreur de fusion" } else { "Merge Error" }.into());
                        ui.set_error_message(e.into());
                        ui.set_show_error(1.0);
                        return;
                    }
                }
            } else {
                sample_df
            };

            // Validate columns
            if let Err(e) = validate_columns(&merged_df, &current_mode) {
                ui.set_error_title(SharedString::from(if fr { "Colonnes manquantes" } else { "Missing Columns" }));
                ui.set_error_message(SharedString::from(e));
                ui.set_show_error(1.0);
                return;
            }

            // Apply merge or update action
            let action = mode_action.as_str();
            let mut final_df = if action == "merge" {
                let params = MergeParams {
                    mode: current_mode.clone(),
                    action: action.to_string(),
                    overwrite_existing: ui.get_overwrite_existing(),
                    run_num: ui.get_run_num().to_string(),
                    minknow_ver: minknow_data.as_ref().map(|d| d.minknow_ver.clone()),
                    pir_ver: ui.get_pir_ver().to_string(),
                    seq_date: minknow_data.as_ref().map(|d| d.seq_date.clone()),
                    fc_id: minknow_data.as_ref().map(|d| d.fc_id.clone()),
                    fc_uses: ui.get_fc_uses().to_string(),
                    fc_pores: minknow_data.as_ref().map(|d| d.fc_pores.clone()),
                    seq_hours: minknow_data.as_ref().map(|d| d.seq_hours.clone()),
                    fasta_date: ui.get_fasta_date().to_string(),
                    seq_kit: minknow_data.as_ref().map(|d| d.seq_kit.clone()),
                    rt_date: ui.get_rt_date().to_string(),
                    lab: ui.get_lab().to_string(),
                    pos_con: ui.get_pos_con().to_string(),
                    neg_con: ui.get_neg_con().to_string(),
                    vp1_date: ui.get_vp1_date().to_string(),
                    pcr_machine: ui.get_pcr_machine().to_string(),
                    vp1_pcr_machine: ui.get_vp1_pcr_machine().to_string(),
                    rtpcr_primers: ui.get_rtpcr_primers().to_string(),
                    vp1_primers: ui.get_vp1_primers().to_string(),
                };

                // Validate inputs
                if let Err(e) = validate_merge_inputs(&params) {
                    ui.set_error_title(if fr { "Erreur de format d'entrée" } else { "Input Format Error" }.into());
                    ui.set_error_message(e.into());
                    ui.set_show_error(1.0);
                    return;
                }

                // Fill run constants
                match fill_run_constants(merged_df, &params) {
                    Ok(df) => match select_expected_columns(df, &current_mode) {
                        Ok(df) => df,
                        Err(e) => {
                            ui.set_error_title(if fr { "Erreur de sélection de colonnes" } else { "Select Columns Error" }.into());
                            ui.set_error_message(e.into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    },
                    Err(e) => {
                        ui.set_error_title(if fr { "Erreur des constantes d'exécution" } else { "Run Constants Error" }.into());
                        ui.set_error_message(e.into());
                        ui.set_show_error(1.0);
                        return;
                    }
                }
            } else {
                match select_expected_columns(merged_df, &current_mode) {
                    Ok(df) => df,
                    Err(e) => {
                        ui.set_error_title(if fr { "Erreur de sélection de colonnes" } else { "Select Columns Error" }.into());
                        ui.set_error_message(e.into());
                        ui.set_show_error(1.0);
                        return;
                    }
                }
            };

            // Save output
            let file_name = format!("{}_merger_output.csv", ui.get_run_num().as_str());
            let file_path = format!("{}/{}", destination_path, file_name);
            let mut file = match std::fs::File::create(&file_path) {
                Ok(f) => f,
                Err(e) => {
                    ui.set_error_title(if fr { "Erreur de création de fichier" } else { "File Create Error" }.into());
                    ui.set_error_message(
                        (if fr {
                            format!("Impossible de créer le fichier de sortie à '{}' : {:?}", file_path, e)
                        } else {
                            format!("Failed to create output file at '{}': {:?}", file_path, e)
                        }).into(),
                    );
                    ui.set_show_error(1.0);
                    return;
                }
            };

            if let Err(e) = CsvWriter::new(&mut file)
                .with_separator(delim)
                .finish(&mut final_df)
            {
                ui.set_error_title(if fr { "Erreur d'écriture CSV" } else { "CSV Write Error" }.into());
                ui.set_error_message((if fr {
                    format!("Échec de l'écriture du CSV de sortie : {:?}", e)
                } else {
                    format!("Failed to write output CSV: {:?}", e)
                }).into());
                ui.set_show_error(1.0);
                return;
            }

            // Success message
            match mode_action.as_str() {
                "merge" => {
                    ui.set_info_title(if fr { "Fusion réussie" } else { "Merge Successful" }.into());
                    ui.set_info_message(if fr {
                        format!(
                            "Le rapport détaillé fusionné a été enregistré dans la destination sous {}.",
                            file_name
                        )
                    } else {
                        format!(
                            "Merged Detailed Run Report saved to destination as {}.",
                            file_name
                        )
                    }.into());
                    ui.set_show_info(1.0);
                }
                "update" => {
                    ui.set_info_title(if fr { "Mise à jour réussie" } else { "Update Successful" }.into());
                    ui.set_info_message(if fr {
                        "Le rapport détaillé mis à jour a été enregistré dans la destination.".into()
                    } else {
                        "Updated Detailed Run Report saved to destination.".into()
                    });
                    ui.set_show_info(1.0);
                }
                _ => {}
            }
        }
    });
}

fn setup_template_handler(ui: &AppWindow) {
    let ui_handle = ui.as_weak();

    ui.on_template(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let fr = ui.get_is_french();
            let current_mode = ui.get_mode().to_string();

            let file_name = if current_mode == "minION" {
                "sample_template_minion.csv"
            } else {
                "sample_template_ddns.csv"
            };

            let file_path = match dirs::download_dir() {
                Some(dir) => dir.join(file_name),
                None => {
                    ui.set_error_title(if fr { "Erreur de répertoire" } else { "Directory Error" }.into());
                    ui.set_error_message(if fr {
                        "Aucun dossier de téléchargements trouvé.".into()
                    } else {
                        "No Downloads folder found.".into()
                    });
                    ui.set_show_error(1.0);
                    return;
                }
            };

            let mut df = match create_template_for_mode(&current_mode) {
                Ok(df) => df,
                Err(e) => {
                    ui.set_error_title(if fr { "Erreur de modèle" } else { "Template Error" }.into());
                    ui.set_error_message(if fr {
                        format!("Échec de la création du modèle {} : {:?}", current_mode, e)
                    } else {
                        format!("Failed to create {} template DataFrame: {:?}", current_mode, e)
                    }.into());
                    ui.set_show_error(1.0);
                    return;
                }
            };

            let file = match std::fs::File::create(&file_path) {
                Ok(f) => f,
                Err(e) => {
                    ui.set_error_title(if fr { "Erreur de création de fichier" } else { "File Create Error" }.into());
                    ui.set_error_message(if fr {
                        format!(
                            "Impossible de créer le fichier modèle à '{}' : {:?}",
                            file_path.display(), e
                        )
                    } else {
                        format!(
                            "Failed to create template file at '{}': {:?}",
                            file_path.display(), e
                        )
                    }.into());
                    ui.set_show_error(1.0);
                    return;
                }
            };

            if let Err(e) = CsvWriter::new(file).finish(&mut df) {
                ui.set_error_title(if fr { "Erreur d'écriture CSV" } else { "CSV Write Error" }.into());
                ui.set_error_message((if fr {
                    format!("Échec de l'écriture du modèle CSV : {:?}", e)
                } else {
                    format!("Failed to write template CSV: {:?}", e)
                }).into());
                ui.set_show_error(1.0);
                return;
            }

            let mode_label = if current_mode == "minION" {
                "minION"
            } else {
                "DDNS"
            };

            ui.set_info_title(if fr { "Modèle enregistré" } else { "Template saved" }.into());
            ui.set_info_message(if fr {
                format!(
                    "Modèle {} enregistré dans le dossier de téléchargements sous {}.",
                    mode_label, file_name
                )
            } else {
                format!(
                    "{} template saved to downloads folder as {}.",
                    mode_label, file_name
                )
            }.into());
            ui.set_show_info(1.0);
        }
    });
}
