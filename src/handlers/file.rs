use rfd::FileDialog;
use slint::{ComponentHandle, SharedString};

use crate::AppWindow;

pub fn setup_file_handlers(ui: &AppWindow) {
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
