use slint::{ComponentHandle, SharedString};

use crate::AppWindow;

pub fn setup_clear_handler(ui: &AppWindow) {
    let ui_handle = ui.as_weak();

    ui.on_clear(move || {
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
