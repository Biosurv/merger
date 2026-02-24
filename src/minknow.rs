use scraper::{Html, Selector};
use serde_json::Value;
use std::fs::File;
use std::io::Read;

#[derive(Default)]
pub struct MinKnowData {
    pub minknow_ver: String,
    pub fc_id: String,
    pub seq_kit: String,
    pub seq_hours: String,
    pub seq_date: String,
    pub fc_pores: String,
}

pub fn parse_minknow_html(path: &str) -> Result<MinKnowData, String> {
    let mut html_content = String::new();
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open HTML file at '{}': {:?}", path, e))?;
    file.read_to_string(&mut html_content)
        .map_err(|e| format!("Failed to read HTML file at '{}': {:?}", path, e))?;

    let document = Html::parse_document(&html_content);
    let script_selector = Selector::parse("script")
        .map_err(|e| format!("Failed to parse script selector: {:?}", e))?;

    let mut script_tag = None;
    for script in document.select(&script_selector) {
        let text = script.text().collect::<String>();
        if text.contains("const reportData=") {
            script_tag = Some(script);
            break;
        }
    }

    let mut data = MinKnowData::default();

    if let Some(script) = script_tag {
        let script_text = script.text().collect::<String>();
        if let Some(json_str) = script_text
            .split("const reportData=")
            .nth(1)
            .and_then(|s| s.split(';').next())
            .map(|s| s.trim())
        {
            if let Ok(report_data) = serde_json::from_str::<Value>(json_str) {
                // Software versions
                if let Some(software_versions) = report_data.get("software_versions").and_then(|v| v.as_array()) {
                    for version in software_versions {
                        if version.get("title").and_then(|t| t.as_str()) == Some("MinKNOW") {
                            data.minknow_ver = version
                                .get("value")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string();
                        }
                    }
                }

                // Run setup
                if let Some(run_config) = report_data.get("run_setup").and_then(|v| v.as_array()) {
                    for config in run_config {
                        match config.get("title").and_then(|t| t.as_str()) {
                            Some("Flow cell ID") => {
                                data.fc_id = config
                                    .get("value")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown")
                                    .to_string();
                            }
                            Some("Kit type") => {
                                data.seq_kit = config
                                    .get("value")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown")
                                    .to_string();
                            }
                            _ => {}
                        }
                    }
                }

                // Run settings
                if let Some(run_settings) = report_data.get("run_settings").and_then(|v| v.as_array()) {
                    for config in run_settings {
                        if config.get("title").and_then(|t| t.as_str()) == Some("Run limit") {
                            data.seq_hours = config
                                .get("value")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string();
                        }
                    }
                }

                // Run end time
                data.seq_date = report_data
                    .get("run_end_time")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.split('T').next())
                    .unwrap_or("Unknown")
                    .to_string();

                // Pore scan
                if let Some(series_data) = report_data
                    .get("pore_scan")
                    .and_then(|v| v.get("series_data"))
                    .and_then(|v| v.as_array())
                {
                    if let Some(pore_available) = series_data
                        .iter()
                        .find(|&s| s.get("name").and_then(|n| n.as_str()) == Some("Pore available"))
                    {
                        if let Some(data_arr) = pore_available.get("data").and_then(|v| v.as_array()) {
                            if let Some(first_data_pair) = data_arr.get(0) {
                                if let Some(value) = first_data_pair.get(1).and_then(|v| v.as_i64()) {
                                    data.fc_pores = value.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(data)
}
