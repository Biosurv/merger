use polars::prelude::*;
use rfd::FileDialog;
use slint::{ComponentHandle, Model, SharedString};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::csv::read_csv_normalized;
use crate::plate_map::apply_plate_map_to_dataframe;
use crate::template::create_template_for_mode;
use crate::types::PendingMerge;
use crate::{AppWindow, PlateMapWindow};

pub fn setup_plate_map_handlers(
    ui: &AppWindow,
    pending_merge: Rc<RefCell<Option<PendingMerge>>>,
    plate_map_window: Rc<RefCell<Option<PlateMapWindow>>>,
    plate_entries: Rc<RefCell<HashMap<String, (String, String)>>>,
) {
    // Yes handler
    {
        let ui_handle = ui.as_weak();
        let pending_merge = pending_merge.clone();
        let plate_map_window = plate_map_window.clone();
        let plate_entries = plate_entries.clone();

        ui.on_missing_plate_yes(move || {
            let ui = match ui_handle.upgrade() {
                Some(u) => u,
                None => return,
            };

            let fr = ui.get_is_french();
            ui.set_show_missing_plate_prompt(0.0);

            // Clear any previous entries when opening plate map
            plate_entries.borrow_mut().clear();

            let mut slot = plate_map_window.borrow_mut();
            if slot.is_none() {
                let win = match PlateMapWindow::new() {
                    Ok(w) => w,
                    Err(e) => {
                        ui.set_error_title(if fr { "Erreur de carte de plaque" } else { "Plate Map Error" }.into());
                        ui.set_error_message((if fr {
                            format!("Impossible d'ouvrir la carte de plaque : {e:?}")
                        } else {
                            format!("Failed to open Plate Map: {e:?}")
                        }).into());
                        ui.set_show_error(1.0);
                        return;
                    }
                };

                // language setting
                win.set_is_french(fr);

                win.on_well_clicked(|_well: SharedString| {});

                // Save well data from popup
                {
                    let plate_entries = plate_entries.clone();
                    win.on_save_well(
                        move |well: SharedString, sample: SharedString, barcode: SharedString| {
                            let sample_str = sample.to_string().trim().to_string();
                            let barcode_str = barcode.to_string().trim().to_string();

                            // Skip empty entries
                            if sample_str.is_empty() || barcode_str.is_empty() {
                                println!("[plate_map handler] on_save_well: skipping empty entry for well '{}'", well);
                                return;
                            }

                            println!("[plate_map handler] on_save_well: well='{}', sample='{}', barcode='{}'",
                                well, sample_str, barcode_str);
                            plate_entries.borrow_mut().insert(
                                well.to_string().to_uppercase(),
                                (sample_str, barcode_str),
                            );
                            println!("[plate_map handler] plate_entries now has {} entries",
                                plate_entries.borrow().len());
                        },
                    );
                }

                // continue: apply plate map and save to CSV
                win.on_continue_clicked({
                    let ui_handle = ui.as_weak();
                    let plate_map_window = plate_map_window.clone();
                    let pending_merge = pending_merge.clone();
                    let plate_entries = plate_entries.clone();

                    move || {
                        println!("[plate_map handler] Continue clicked!");
                        println!("[plate_map handler] plate_entries has {} entries", plate_entries.borrow().len());
                        for (well, (sample, barcode)) in plate_entries.borrow().iter() {
                            println!("[plate_map handler]   {} -> sample='{}', barcode='{}'", well, sample, barcode);
                        }

                        // Hide the plate map window
                        if let Some(w) = plate_map_window.borrow().as_ref() {
                            let _ = w.hide();
                        }

                        let ui = match ui_handle.upgrade() {
                            Some(u) => u,
                            None => return,
                        };

                        let fr = ui.get_is_french();

                        // get pendingmerge data
                        let pm = match pending_merge.borrow_mut().take() {
                            Some(pm) => pm,
                            None => {
                                ui.set_error_title(if fr { "Erreur" } else { "Error" }.into());
                                ui.set_error_message(if fr {
                                    "Aucune donnée de fusion en attente trouvée.".into()
                                } else {
                                    "No pending merge data found.".into()
                                });
                                ui.set_show_error(1.0);
                                return;
                            }
                        };

                        let entries = plate_entries.borrow();

                        if entries.is_empty() {
                            ui.set_error_title(if fr { "Aucune donnée" } else { "No data" }.into());
                            ui.set_error_message(if fr {
                                "Aucun puits n'a été rempli. Veuillez remplir au moins un puits.".into()
                            } else {
                                "No wells were filled. Please fill at least one well.".into()
                            });
                            ui.set_show_error(1.0);
                            // restore 
                            *pending_merge.borrow_mut() = Some(pm);
                            return;
                        }

                        // Read the sample CSV
                        let (df, delim) = match read_csv_normalized(&pm.piranha_path) {
                            Ok((df, delim)) => (df, delim),
                            Err(e) => {
                                ui.set_error_title(if fr { "Erreur de lecture CSV" } else { "CSV Read Error" }.into());
                                ui.set_error_message(e.into());
                                ui.set_show_error(1.0);
                                return;
                            }
                        };

                        // Apply plate map entries
                        let mut updated_df = match apply_plate_map_to_dataframe(df, &entries) {
                            Ok(df) => df,
                            Err(e) => {
                                ui.set_error_title(if fr { "Erreur de carte de plaque" } else { "Plate Map Error" }.into());
                                ui.set_error_message(e.into());
                                ui.set_show_error(1.0);
                                return;
                            }
                        };

                        // Write back to the same file
                        let mut file = match std::fs::File::create(&pm.piranha_path) {
                            Ok(f) => f,
                            Err(e) => {
                                ui.set_error_title(if fr { "Erreur d'écriture de fichier" } else { "File Write Error" }.into());
                                ui.set_error_message(
                                    (if fr {
                                        format!("Impossible d'écrire dans '{}' : {:?}", pm.piranha_path, e)
                                    } else {
                                        format!("Failed to write to '{}': {:?}", pm.piranha_path, e)
                                    }).into(),
                                );
                                ui.set_show_error(1.0);
                                return;
                            }
                        };

                        if let Err(e) =
                            CsvWriter::new(&mut file).with_separator(delim).finish(&mut updated_df)
                        {
                            ui.set_error_title(if fr { "Erreur d'écriture CSV" } else { "CSV Write Error" }.into());
                            ui.set_error_message(
                                (if fr {
                                    format!("Échec de l'écriture du CSV : {:?}", e)
                                } else {
                                    format!("Failed to write CSV: {:?}", e)
                                }).into(),
                            );
                            ui.set_show_error(1.0);
                            return;
                        }

                        // Clear plate entries
                        drop(entries);
                        plate_entries.borrow_mut().clear();

                        // Show success and prompt to merge again
                        let file_label = pm.piranha_path.split(['/', '\\']).last().unwrap_or(&pm.piranha_path);
                        ui.set_info_title(if fr { "Carte de plaque appliquée" } else { "Plate Map Applied" }.into());
                        ui.set_info_message(if fr {
                            format!(
                                "Les données d'échantillons et de codes-barres ont été ajoutées à {}.\n\nVeuillez cliquer sur Fusionner pour continuer.",
                                file_label
                            )
                        } else {
                            format!(
                                "Sample and barcode data has been added to {}.\n\nPlease click Merge to continue.",
                                file_label
                            )
                        }.into());
                        ui.set_show_info(1.0);
                    }
                });

                // Cancel button
                win.on_cancel_clicked({
                    let ui_handle = ui.as_weak();
                    let plate_map_window = plate_map_window.clone();
                    let pending_merge = pending_merge.clone();
                    let plate_entries = plate_entries.clone();

                    move || {
                        if let Some(w) = plate_map_window.borrow().as_ref() {
                            let _ = w.hide();
                        }

                        *pending_merge.borrow_mut() = None;
                        plate_entries.borrow_mut().clear();

                        if let Some(ui) = ui_handle.upgrade() {
                            let fr = ui.get_is_french();
                            ui.set_info_title(if fr { "Annulé" } else { "Cancelled" }.into());
                            ui.set_info_message(if fr {
                                "Carte de plaque annulée.".into()
                            } else {
                                "Plate map cancelled.".into()
                            });
                            ui.set_show_info(1.0);
                        }
                    }
                });

                // barcodes vertically
                win.on_fill_barcodes_vertical({
                    let plate_map_window = plate_map_window.clone();

                    move || {
                        if let Some(w) = plate_map_window.borrow().as_ref() {
                            let filled: Vec<bool> = w.get_filled().iter().collect();
                            let mut barcodes: Vec<SharedString> = w.get_barcodes().iter().collect();

                            let mut barcode_num = 1;
                            // Iterate column by column (c), then row by row (r)
                            for c in 0..12 {
                                for r in 0..8 {
                                    let idx = r * 12 + c;
                                    if !filled[idx] && barcode_num <= 96 {
                                        barcodes[idx] = SharedString::from(format!("barcode{:02}", barcode_num));
                                        barcode_num += 1;
                                    }
                                }
                            }

                            w.set_barcodes(std::rc::Rc::new(slint::VecModel::from(barcodes)).into());
                        }
                    }
                });

                //  barcodes horizontally
                win.on_fill_barcodes_horizontal({
                    let plate_map_window = plate_map_window.clone();

                    move || {
                        if let Some(w) = plate_map_window.borrow().as_ref() {
                            let filled: Vec<bool> = w.get_filled().iter().collect();
                            let mut barcodes: Vec<SharedString> = w.get_barcodes().iter().collect();

                            let mut barcode_num = 1;
                            // Iterate row by row (r), then column by column (c)
                            for r in 0..8 {
                                for c in 0..12 {
                                    let idx = r * 12 + c;
                                    if !filled[idx] && barcode_num <= 96 {
                                        barcodes[idx] = SharedString::from(format!("barcode{:02}", barcode_num));
                                        barcode_num += 1;
                                    }
                                }
                            }

                            w.set_barcodes(std::rc::Rc::new(slint::VecModel::from(barcodes)).into());
                        }
                    }
                });

                // sample preview
                win.on_calculate_sample_preview({
                    let plate_map_window = plate_map_window.clone();

                    move || {
                        if let Some(w) = plate_map_window.borrow().as_ref() {
                            let from_idx = w.get_sample_from_idx() as usize;
                            let to_idx = w.get_sample_to_idx() as usize;
                            let direction = w.get_sample_fill_direction().to_string();
                            let filled: Vec<bool> = w.get_filled().iter().collect();

                            let mut preview: Vec<bool> = vec![false; 96];

                            // calculate path based on direction
                            let indices = if direction == "Vertical" {
                                // Vertical
                                let from_row = from_idx / 12;
                                let from_col = from_idx % 12;
                                let to_row = to_idx / 12;
                                let to_col = to_idx % 12;

                                let mut indices = Vec::new();
                                if from_col <= to_col {
                                    for c in from_col..=to_col {
                                        let row_start = if c == from_col { from_row } else { 0 };
                                        let row_end = if c == to_col { to_row } else { 7 };
                                        for r in row_start..=row_end {
                                            indices.push(r * 12 + c);
                                        }
                                    }
                                } else {
                                    // Wrap around case
                                    for c in from_col..12 {
                                        let row_start = if c == from_col { from_row } else { 0 };
                                        for r in row_start..8 {
                                            indices.push(r * 12 + c);
                                        }
                                    }
                                    for c in 0..=to_col {
                                        let row_end = if c == to_col { to_row } else { 7 };
                                        for r in 0..=row_end {
                                            indices.push(r * 12 + c);
                                        }
                                    }
                                }
                                indices
                            } else {
                                // Horizontal
                                let mut indices = Vec::new();
                                if from_idx <= to_idx {
                                    for idx in from_idx..=to_idx {
                                        indices.push(idx);
                                    }
                                } else {
                                    // Wrap around case
                                    for idx in from_idx..96 {
                                        indices.push(idx);
                                    }
                                    for idx in 0..=to_idx {
                                        indices.push(idx);
                                    }
                                }
                                indices
                            };

                            // Mark preview wells
                            for idx in indices {
                                if !filled[idx] {
                                    preview[idx] = true;
                                }
                            }

                            w.set_sample_preview(std::rc::Rc::new(slint::VecModel::from(preview)).into());
                        }
                    }
                });

                // Fill sample preview
                win.on_fill_sample_preview({
                    let plate_map_window = plate_map_window.clone();

                    move || {
                        if let Some(w) = plate_map_window.borrow().as_ref() {
                            let preview: Vec<bool> = w.get_sample_preview().iter().collect();
                            let mut samples: Vec<SharedString> = w.get_samples().iter().collect();
                            let prefix = w.get_sample_prefix().to_string();
                            let start_str = w.get_sample_start().to_string();
                            let start_str = start_str.trim();
                            let pad_width = start_str.len();
                            let start_num = start_str.parse::<u32>().unwrap_or(1);
                            let direction = w.get_sample_fill_direction().to_string();
                            let from_idx = w.get_sample_from_idx() as usize;
                            let to_idx = w.get_sample_to_idx() as usize;

                            // Get the indices in order based on direction
                            let indices: Vec<usize> = if direction == "Vertical" {
                                let from_row = from_idx / 12;
                                let from_col = from_idx % 12;
                                let to_row = to_idx / 12;
                                let to_col = to_idx % 12;

                                let mut indices = Vec::new();
                                if from_col <= to_col {
                                    for c in from_col..=to_col {
                                        let row_start = if c == from_col { from_row } else { 0 };
                                        let row_end = if c == to_col { to_row } else { 7 };
                                        for r in row_start..=row_end {
                                            indices.push(r * 12 + c);
                                        }
                                    }
                                } else {
                                    for c in from_col..12 {
                                        let row_start = if c == from_col { from_row } else { 0 };
                                        for r in row_start..8 {
                                            indices.push(r * 12 + c);
                                        }
                                    }
                                    for c in 0..=to_col {
                                        let row_end = if c == to_col { to_row } else { 7 };
                                        for r in 0..=row_end {
                                            indices.push(r * 12 + c);
                                        }
                                    }
                                }
                                indices
                            } else {
                                if from_idx <= to_idx {
                                    (from_idx..=to_idx).collect()
                                } else {
                                    (from_idx..96).chain(0..=to_idx).collect()
                                }
                            };

                            // Fill samples in order for wells that are in preview
                            let mut current_num = start_num;
                            for idx in indices {
                                if preview[idx] && current_num <= 9999 {
                                    samples[idx] = SharedString::from(format!("{}{:0>width$}", prefix, current_num, width = pad_width));
                                    current_num += 1;
                                }
                            }

                            w.set_samples(std::rc::Rc::new(slint::VecModel::from(samples)).into());

                            // Clear the preview
                            let empty_preview: Vec<bool> = vec![false; 96];
                            w.set_sample_preview(std::rc::Rc::new(slint::VecModel::from(empty_preview)).into());
                        }
                    }
                });

                // Clear sample preview
                win.on_clear_sample_preview({
                    let plate_map_window = plate_map_window.clone();

                    move || {
                        if let Some(w) = plate_map_window.borrow().as_ref() {
                            let empty_preview: Vec<bool> = vec![false; 96];
                            w.set_sample_preview(std::rc::Rc::new(slint::VecModel::from(empty_preview)).into());
                        }
                    }
                });

                // Clear all wells
                win.on_clear_all_wells({
                    let plate_map_window = plate_map_window.clone();
                    let plate_entries = plate_entries.clone();

                    move || {
                        if let Some(w) = plate_map_window.borrow().as_ref() {
                            // Reset all arrays to empty
                            let empty_filled: Vec<bool> = vec![false; 96];
                            let empty_strings: Vec<SharedString> = vec![SharedString::from(""); 96];

                            w.set_filled(std::rc::Rc::new(slint::VecModel::from(empty_filled)).into());
                            w.set_samples(std::rc::Rc::new(slint::VecModel::from(empty_strings.clone())).into());
                            w.set_barcodes(std::rc::Rc::new(slint::VecModel::from(empty_strings)).into());

                            // cclear the plate entries HashMap
                            plate_entries.borrow_mut().clear();
                        }
                    }
                });

                *slot = Some(win);
            } else {
                // Window already exist so just clear the filled state and update language
                if let Some(w) = slot.as_ref() {
                    w.set_is_french(fr);
                    // Reset filled array to all false
                    let empty_filled: Vec<bool> = vec![false; 96];
                    w.set_filled(std::rc::Rc::new(slint::VecModel::from(empty_filled)).into());
                }
            }

            if let Some(w) = slot.as_ref() {
                let _ = w.show();
            }
        });
    }

    // No button handler
    {
        let ui_handle = ui.as_weak();
        let pending_merge = pending_merge.clone();

        ui.on_missing_plate_no(move || {
            let ui = match ui_handle.upgrade() {
                Some(u) => u,
                None => return,
            };

            let fr = ui.get_is_french();
            ui.set_show_missing_plate_prompt(0.0);
            *pending_merge.borrow_mut() = None;

            ui.set_error_title(if fr { "Échantillon/code-barres manquant" } else { "Missing sample/barcode" }.into());
            ui.set_error_message(if fr {
                "Veuillez ajouter un échantillon + code-barres pour continuer.".into()
            } else {
                "Please add sample + barcode to continue.".into()
            });
            ui.set_show_error(1.0);
        });
    }
}

// Handler for the standalone plate map
pub fn setup_standalone_plate_map_handler(
    ui: &AppWindow,
    standalone_plate_map_window: Rc<RefCell<Option<PlateMapWindow>>>,
    standalone_plate_entries: Rc<RefCell<HashMap<String, (String, String)>>>,
) {
    let ui_handle = ui.as_weak();
    let plate_map_window = standalone_plate_map_window.clone();
    let plate_entries = standalone_plate_entries.clone();

    ui.on_plate_map(move || {
        let ui = match ui_handle.upgrade() {
            Some(u) => u,
            None => return,
        };

        let fr = ui.get_is_french();

        // Clear any previous entries
        plate_entries.borrow_mut().clear();

        let mut slot = plate_map_window.borrow_mut();
        if slot.is_none() {
            let win = match PlateMapWindow::new() {
                Ok(w) => w,
                Err(e) => {
                    ui.set_error_title(if fr { "Erreur de carte de plaque" } else { "Plate Map Error" }.into());
                    ui.set_error_message((if fr {
                        format!("Impossible d'ouvrir la carte de plaque : {e:?}")
                    } else {
                        format!("Failed to open Plate Map: {e:?}")
                    }).into());
                    ui.set_show_error(1.0);
                    return;
                }
            };

            // Set create mode and propagate language
            win.set_create_mode(true);
            win.set_destination_path(SharedString::from(""));
            win.set_is_french(fr);

            win.on_well_clicked(|_well: SharedString| {});

            // Save well data from popup
            {
                let plate_entries = plate_entries.clone();
                win.on_save_well(
                    move |well: SharedString, sample: SharedString, barcode: SharedString| {
                        let sample_str = sample.to_string().trim().to_string();
                        let barcode_str = barcode.to_string().trim().to_string();

                        if sample_str.is_empty() || barcode_str.is_empty() {
                            println!("[standalone plate_map] on_save_well: skipping empty entry for well '{}'", well);
                            return;
                        }

                        println!("[standalone plate_map] on_save_well: well='{}', sample='{}', barcode='{}'",
                            well, sample_str, barcode_str);
                        plate_entries.borrow_mut().insert(
                            well.to_string().to_uppercase(),
                            (sample_str, barcode_str),
                        );
                    },
                );
            }

            // Select destination folder
            win.on_select_destination({
                let plate_map_window = plate_map_window.clone();

                move || {
                    if let Some(folder) = FileDialog::new().pick_folder() {
                        if let Some(w) = plate_map_window.borrow().as_ref() {
                            w.set_destination_path(SharedString::from(folder.to_string_lossy().to_string()));
                        }
                    }
                }
            });

            // Continue button: create template, apply plate map, save to destination
            win.on_continue_clicked({
                let ui_handle = ui.as_weak();
                let plate_map_window = plate_map_window.clone();
                let plate_entries = plate_entries.clone();

                move || {
                    let ui = match ui_handle.upgrade() {
                        Some(u) => u,
                        None => return,
                    };

                    let fr = ui.get_is_french();

                    let win_ref = plate_map_window.borrow();
                    let win = match win_ref.as_ref() {
                        Some(w) => w,
                        None => return,
                    };

                    // Check if in create mode
                    if !win.get_create_mode() {
                        return;
                    }

                    let destination_path = win.get_destination_path().to_string();
                    if destination_path.is_empty() {
                        ui.set_error_title(if fr { "Aucune destination" } else { "No Destination" }.into());
                        ui.set_error_message(if fr {
                            "Veuillez sélectionner un dossier de destination.".into()
                        } else {
                            "Please select a destination folder.".into()
                        });
                        ui.set_show_error(1.0);
                        return;
                    }

                    // Collect plate entries from the UI
                    let filled: Vec<bool> = win.get_filled().iter().collect();
                    let samples: Vec<SharedString> = win.get_samples().iter().collect();
                    let barcodes: Vec<SharedString> = win.get_barcodes().iter().collect();

                    // Build entries from the UI state
                    let mut entries: HashMap<String, (String, String)> = HashMap::new();
                    for idx in 0..96 {
                        if filled[idx] {
                            let row = idx / 12;
                            let col = (idx % 12) + 1;
                            let row_char = (b'A' + row as u8) as char;
                            let well_id = format!("{}{}", row_char, col);
                            let sample = samples[idx].to_string();
                            let barcode = barcodes[idx].to_string();
                            if !sample.is_empty() && !barcode.is_empty() {
                                entries.insert(well_id, (sample, barcode));
                            }
                        }
                    }

                    // merge with saved entries from popup
                    for (well, (sample, barcode)) in plate_entries.borrow().iter() {
                        entries.insert(well.clone(), (sample.clone(), barcode.clone()));
                    }

                    if entries.is_empty() {
                        ui.set_error_title(if fr { "Aucune donnée" } else { "No data" }.into());
                        ui.set_error_message(if fr {
                            "Aucun puits n'a été rempli. Veuillez remplir au moins un puits.".into()
                        } else {
                            "No wells were filled. Please fill at least one well.".into()
                        });
                        ui.set_show_error(1.0);
                        return;
                    }

                    let current_mode = ui.get_mode().to_string();

                    // Create template DataFrame
                    let df = match create_template_for_mode(&current_mode) {
                        Ok(df) => df,
                        Err(e) => {
                            ui.set_error_title(if fr { "Erreur de modèle" } else { "Template Error" }.into());
                            ui.set_error_message((if fr {
                                format!("Échec de la création du modèle : {:?}", e)
                            } else {
                                format!("Failed to create template: {:?}", e)
                            }).into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    };

                    // Apply plate map entries
                    let mut updated_df = match apply_plate_map_to_dataframe(df, &entries) {
                        Ok(df) => df,
                        Err(e) => {
                            ui.set_error_title(if fr { "Erreur de carte de plaque" } else { "Plate Map Error" }.into());
                            ui.set_error_message(e.into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    };

                    // Generate filename
                    let file_name = if current_mode == "minION" {
                        "sample_platemap_minion.csv"
                    } else {
                        "sample_platemap_ddns.csv"
                    };
                    let file_path = format!("{}/{}", destination_path, file_name);

                    // Write to file
                    let mut file = match std::fs::File::create(&file_path) {
                        Ok(f) => f,
                        Err(e) => {
                            ui.set_error_title(if fr { "Erreur d'écriture de fichier" } else { "File Write Error" }.into());
                            ui.set_error_message((if fr {
                                format!("Impossible de créer le fichier '{}' : {:?}", file_path, e)
                            } else {
                                format!("Failed to create file '{}': {:?}", file_path, e)
                            }).into());
                            ui.set_show_error(1.0);
                            return;
                        }
                    };

                    if let Err(e) = CsvWriter::new(&mut file).finish(&mut updated_df) {
                        ui.set_error_title(if fr { "Erreur d'écriture CSV" } else { "CSV Write Error" }.into());
                        ui.set_error_message((if fr {
                            format!("Échec de l'écriture du CSV : {:?}", e)
                        } else {
                            format!("Failed to write CSV: {:?}", e)
                        }).into());
                        ui.set_show_error(1.0);
                        return;
                    }

                    // Hide window and clear entries
                    drop(win_ref);
                    if let Some(w) = plate_map_window.borrow().as_ref() {
                        let _ = w.hide();
                    }
                    plate_entries.borrow_mut().clear();

                    // Show success
                    let mode_label = if current_mode == "minION" { "minION" } else { "DDNS" };
                    ui.set_info_title(if fr { "Carte de plaque enregistrée" } else { "Plate Map Saved" }.into());
                    ui.set_info_message(if fr {
                        format!(
                            "Fichier d'échantillons {} avec les données de la carte de plaque enregistré sous {}.",
                            mode_label, file_name
                        )
                    } else {
                        format!(
                            "{} sample file with plate map data saved as {}.",
                            mode_label, file_name
                        )
                    }.into());
                    ui.set_show_info(1.0);
                }
            });

            // Cancel button
            win.on_cancel_clicked({
                let ui_handle = ui.as_weak();
                let plate_map_window = plate_map_window.clone();
                let plate_entries = plate_entries.clone();

                move || {
                    if let Some(w) = plate_map_window.borrow().as_ref() {
                        let _ = w.hide();
                    }

                    plate_entries.borrow_mut().clear();

                    if let Some(ui) = ui_handle.upgrade() {
                        let fr = ui.get_is_french();
                        ui.set_info_title(if fr { "Annulé" } else { "Cancelled" }.into());
                        ui.set_info_message(if fr {
                            "Carte de plaque annulée.".into()
                        } else {
                            "Plate map cancelled.".into()
                        });
                        ui.set_show_info(1.0);
                    }
                }
            });

            // Fill barcodes vertically
            win.on_fill_barcodes_vertical({
                let plate_map_window = plate_map_window.clone();

                move || {
                    if let Some(w) = plate_map_window.borrow().as_ref() {
                        let filled: Vec<bool> = w.get_filled().iter().collect();
                        let mut barcodes: Vec<SharedString> = w.get_barcodes().iter().collect();

                        let mut barcode_num = 1;
                        for c in 0..12 {
                            for r in 0..8 {
                                let idx = r * 12 + c;
                                if !filled[idx] && barcode_num <= 96 {
                                    barcodes[idx] = SharedString::from(format!("barcode{:02}", barcode_num));
                                    barcode_num += 1;
                                }
                            }
                        }

                        w.set_barcodes(std::rc::Rc::new(slint::VecModel::from(barcodes)).into());
                    }
                }
            });

            // Fill barcodes horizontally
            win.on_fill_barcodes_horizontal({
                let plate_map_window = plate_map_window.clone();

                move || {
                    if let Some(w) = plate_map_window.borrow().as_ref() {
                        let filled: Vec<bool> = w.get_filled().iter().collect();
                        let mut barcodes: Vec<SharedString> = w.get_barcodes().iter().collect();

                        let mut barcode_num = 1;
                        for r in 0..8 {
                            for c in 0..12 {
                                let idx = r * 12 + c;
                                if !filled[idx] && barcode_num <= 96 {
                                    barcodes[idx] = SharedString::from(format!("barcode{:02}", barcode_num));
                                    barcode_num += 1;
                                }
                            }
                        }

                        w.set_barcodes(std::rc::Rc::new(slint::VecModel::from(barcodes)).into());
                    }
                }
            });

            // Calculate sample preview
            win.on_calculate_sample_preview({
                let plate_map_window = plate_map_window.clone();

                move || {
                    if let Some(w) = plate_map_window.borrow().as_ref() {
                        let from_idx = w.get_sample_from_idx() as usize;
                        let to_idx = w.get_sample_to_idx() as usize;
                        let direction = w.get_sample_fill_direction().to_string();
                        let filled: Vec<bool> = w.get_filled().iter().collect();

                        let mut preview: Vec<bool> = vec![false; 96];

                        let indices = if direction == "Vertical" {
                            let from_row = from_idx / 12;
                            let from_col = from_idx % 12;
                            let to_row = to_idx / 12;
                            let to_col = to_idx % 12;

                            let mut indices = Vec::new();
                            if from_col <= to_col {
                                for c in from_col..=to_col {
                                    let row_start = if c == from_col { from_row } else { 0 };
                                    let row_end = if c == to_col { to_row } else { 7 };
                                    for r in row_start..=row_end {
                                        indices.push(r * 12 + c);
                                    }
                                }
                            } else {
                                for c in from_col..12 {
                                    let row_start = if c == from_col { from_row } else { 0 };
                                    for r in row_start..8 {
                                        indices.push(r * 12 + c);
                                    }
                                }
                                for c in 0..=to_col {
                                    let row_end = if c == to_col { to_row } else { 7 };
                                    for r in 0..=row_end {
                                        indices.push(r * 12 + c);
                                    }
                                }
                            }
                            indices
                        } else {
                            let mut indices = Vec::new();
                            if from_idx <= to_idx {
                                for idx in from_idx..=to_idx {
                                    indices.push(idx);
                                }
                            } else {
                                for idx in from_idx..96 {
                                    indices.push(idx);
                                }
                                for idx in 0..=to_idx {
                                    indices.push(idx);
                                }
                            }
                            indices
                        };

                        for idx in indices {
                            if !filled[idx] {
                                preview[idx] = true;
                            }
                        }

                        w.set_sample_preview(std::rc::Rc::new(slint::VecModel::from(preview)).into());
                    }
                }
            });

            // Fill sample preview
            win.on_fill_sample_preview({
                let plate_map_window = plate_map_window.clone();

                move || {
                    if let Some(w) = plate_map_window.borrow().as_ref() {
                        let preview: Vec<bool> = w.get_sample_preview().iter().collect();
                        let mut samples: Vec<SharedString> = w.get_samples().iter().collect();
                        let prefix = w.get_sample_prefix().to_string();
                        let start_str = w.get_sample_start().to_string();
                        let start_str = start_str.trim();
                        let pad_width = start_str.len();
                        let start_num = start_str.parse::<u32>().unwrap_or(1);
                        let direction = w.get_sample_fill_direction().to_string();
                        let from_idx = w.get_sample_from_idx() as usize;
                        let to_idx = w.get_sample_to_idx() as usize;

                        let indices: Vec<usize> = if direction == "Vertical" {
                            let from_row = from_idx / 12;
                            let from_col = from_idx % 12;
                            let to_row = to_idx / 12;
                            let to_col = to_idx % 12;

                            let mut indices = Vec::new();
                            if from_col <= to_col {
                                for c in from_col..=to_col {
                                    let row_start = if c == from_col { from_row } else { 0 };
                                    let row_end = if c == to_col { to_row } else { 7 };
                                    for r in row_start..=row_end {
                                        indices.push(r * 12 + c);
                                    }
                                }
                            } else {
                                for c in from_col..12 {
                                    let row_start = if c == from_col { from_row } else { 0 };
                                    for r in row_start..8 {
                                        indices.push(r * 12 + c);
                                    }
                                }
                                for c in 0..=to_col {
                                    let row_end = if c == to_col { to_row } else { 7 };
                                    for r in 0..=row_end {
                                        indices.push(r * 12 + c);
                                    }
                                }
                            }
                            indices
                        } else {
                            if from_idx <= to_idx {
                                (from_idx..=to_idx).collect()
                            } else {
                                (from_idx..96).chain(0..=to_idx).collect()
                            }
                        };

                        let mut current_num = start_num;
                        for idx in indices {
                            if preview[idx] && current_num <= 9999 {
                                samples[idx] = SharedString::from(format!("{}{:0>width$}", prefix, current_num, width = pad_width));
                                current_num += 1;
                            }
                        }

                        w.set_samples(std::rc::Rc::new(slint::VecModel::from(samples)).into());

                        let empty_preview: Vec<bool> = vec![false; 96];
                        w.set_sample_preview(std::rc::Rc::new(slint::VecModel::from(empty_preview)).into());
                    }
                }
            });

            // clear sample preview
            win.on_clear_sample_preview({
                let plate_map_window = plate_map_window.clone();

                move || {
                    if let Some(w) = plate_map_window.borrow().as_ref() {
                        let empty_preview: Vec<bool> = vec![false; 96];
                        w.set_sample_preview(std::rc::Rc::new(slint::VecModel::from(empty_preview)).into());
                    }
                }
            });

            // Clear all wells
            win.on_clear_all_wells({
                let plate_map_window = plate_map_window.clone();
                let plate_entries = plate_entries.clone();

                move || {
                    if let Some(w) = plate_map_window.borrow().as_ref() {
                        let empty_filled: Vec<bool> = vec![false; 96];
                        let empty_strings: Vec<SharedString> = vec![SharedString::from(""); 96];

                        w.set_filled(std::rc::Rc::new(slint::VecModel::from(empty_filled)).into());
                        w.set_samples(std::rc::Rc::new(slint::VecModel::from(empty_strings.clone())).into());
                        w.set_barcodes(std::rc::Rc::new(slint::VecModel::from(empty_strings)).into());

                        plate_entries.borrow_mut().clear();
                    }
                }
            });

            *slot = Some(win);
        } else {
            //  reset state for create mode and update language
            if let Some(w) = slot.as_ref() {
                w.set_create_mode(true);
                w.set_destination_path(SharedString::from(""));
                w.set_is_french(fr);

                let empty_filled: Vec<bool> = vec![false; 96];
                let empty_strings: Vec<SharedString> = vec![SharedString::from(""); 96];
                w.set_filled(std::rc::Rc::new(slint::VecModel::from(empty_filled)).into());
                w.set_samples(std::rc::Rc::new(slint::VecModel::from(empty_strings.clone())).into());
                w.set_barcodes(std::rc::Rc::new(slint::VecModel::from(empty_strings)).into());
            }
        }

        if let Some(w) = slot.as_ref() {
            let _ = w.show();
        }
    });
}
