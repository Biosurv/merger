use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, ETAG, IF_NONE_MATCH, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

static DEFAULT_UA: &str = "UpdateChecker/1.0 (rust)";
static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_millis(4500))
        .build()
        .expect("Failed to build HTTP client")

});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag: String,
    pub html_url: String,
    pub etag: Option<String>
}

#[derive(Debug, Clone)]
pub struct UpdateChecker {
    pub owner : String,
    pub repo : String,
    pub current_version : String,
    pub check_prereleases : bool,
    pub min_interval_minutes: i64,
    pub github_token: Option<String>,
    org: String,
    app: String
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("http status: {0}")]
    Http(u16),
    #[error("Parsing error: {0}")]
    Json(String),
    #[error("IO error: {0}")]
    Io(String)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SavedState {
    last_checked_iso: Option<String>,
    etag: Option<String>,
    seen_version: Option<String>,
}


impl UpdateChecker {
    pub fn new<S: Into<String>>(owner: S, repo: S, current_version: S) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
            current_version: current_version.into(),
            check_prereleases: false,
            min_interval_minutes: 60 * 24,
            github_token: None,
            org: "YOUR_ORG".into(),
            app: "YOUR_APP".into(),
        }
    }

    pub fn with_settings_namespace(mut self, org: impl Into<String>, app: impl Into<String>) -> Self {
        self.org = org.into();
        self.app = app.into();
        self
    }

    pub fn check(&self, force: bool) -> Result<Option<ReleaseInfo>, UpdateError> {
        if !force && !self.should_check_now()? {
            return Ok(None);
        }

        let mut state = self.load_state().unwrap_or_default();

        let url = if self.check_prereleases {
            format!("https://api.github.com/repos/{}/{}/releases", self.owner, self.repo)
        } else {
            format!("https://api.github.com/repos/{}/{}/releases/latest", self.owner, self.repo)
        };

        let mut req = CLIENT
            .get(&url)
            .header(ACCEPT, "application/vnd.github+json")
            .header(USER_AGENT, format!("{}-updater", self.repo));

        if let Some(et) = &state.etag {
            req = req.header(IF_NONE_MATCH, et);
        }
        if let Some(tok) = &self.github_token {
            req = req.header(AUTHORIZATION, format!("Bearer {}", tok));
        }

        let resp = req.send().map_err(|e| UpdateError::Network(e.to_string()))?;

        // record last-checked regardless of outcome
        self.touch_last_checked(&mut state)?;

        if resp.status().as_u16() == 304 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            return Err(UpdateError::Http(resp.status().as_u16()));
        }

        let etag_hdr = resp
            .headers()
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        if let Some(et) = etag_hdr.clone() {
            state.etag = Some(et);
            self.save_state(&state)?;
        }

        let body = resp.text().map_err(|e| UpdateError::Network(e.to_string()))?;

        let latest = if self.check_prereleases {
            let releases: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| UpdateError::Json(e.to_string()))?;
            let arr = releases
                .as_array()
                .ok_or_else(|| UpdateError::Json("expected array".into()))?;
            let mut filtered: Vec<&serde_json::Value> =
                arr.iter().filter(|r| r.get("draft").and_then(|d| d.as_bool()) != Some(true)).collect();
            // newest by created_at
            filtered.sort_by(|a, b| {
                let ka = a.get("created_at").and_then(|x| x.as_str()).unwrap_or("");
                let kb = b.get("created_at").and_then(|x| x.as_str()).unwrap_or("");
                kb.cmp(ka)
            });
            let chosen = filtered.first().ok_or_else(|| UpdateError::Json("no releases".into()))?;
            ReleaseInfo {
                tag: chosen.get("tag_name").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                html_url: chosen.get("html_url").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                etag: etag_hdr,
            }
        } else {
            let obj: serde_json::Value =
                serde_json::from_str(&body).map_err(|e| UpdateError::Json(e.to_string()))?;
            ReleaseInfo {
                tag: obj.get("tag_name").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                html_url: obj.get("html_url").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                etag: etag_hdr,
            }
        };

        if latest.tag.is_empty() {
            return Ok(None);
        }

        if cmp_semver(&latest.tag, &self.current_version) == Ordering::Greater {
            state.seen_version = Some(latest.tag.clone());
            self.save_state(&state)?;
            return Ok(Some(latest));
        }

        Ok(None)
    }

    fn should_check_now(&self) -> Result<bool, UpdateError> {
        if self.min_interval_minutes <= 0 {
            return Ok(true);
        }
        let state = self.load_state().unwrap_or_default();
        let Some(iso) = state.last_checked_iso else { return Ok(true) };
        let Ok(last) = iso.parse::<DateTime<Utc>>() else { return Ok(true) };
        let delta = Utc::now() - last;
        Ok(delta.num_minutes() >= self.min_interval_minutes)
    }

    fn touch_last_checked(&self, state: &mut SavedState) -> Result<(), UpdateError> {
        state.last_checked_iso = Some(Utc::now().to_rfc3339());
        self.save_state(state)
    }

    fn state_path(&self) -> Result<PathBuf, UpdateError> {
        let proj = directories::ProjectDirs::from("com", &self.org, &self.app)
            .ok_or_else(|| UpdateError::Io("cannot determine config dir".into()))?;
        let dir = proj.config_dir().to_path_buf();
        fs::create_dir_all(&dir).map_err(|e| UpdateError::Io(e.to_string()))?;
        Ok(dir.join("updater_state.json"))
    }

    fn load_state(&self) -> Result<SavedState, UpdateError> {
        let path = self.state_path()?;
        if !path.exists() {
            return Ok(SavedState::default());
        }
        let mut f = fs::File::open(&path).map_err(|e| UpdateError::Io(e.to_string()))?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(|e| UpdateError::Io(e.to_string()))?;
        serde_json::from_str(&s).map_err(|e| UpdateError::Json(e.to_string()))
    }

    fn save_state(&self, state: &SavedState) -> Result<(), UpdateError> {
        let path = self.state_path()?;
        let mut f = fs::File::create(&path).map_err(|e| UpdateError::Io(e.to_string()))?;
        let s = serde_json::to_string_pretty(state).map_err(|e| UpdateError::Json(e.to_string()))?;
        use std::io::Write;
        f.write_all(s.as_bytes()).map_err(|e| UpdateError::Io(e.to_string()))
    }

    pub fn clear_cache(&self) -> Result<(), UpdateError> {
    let path = self.state_path()?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| UpdateError::Io(e.to_string()))?;
    }
    Ok(())
}
}


fn normalise(v: &str) -> String {
    let v = v.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    let mut nums = Vec::new();
    for part in v.split(|c| c == '.' || c == '-' || c == '+') {
        if nums.len() >= 3 { break; }
        if let Ok(n) = part.parse::<u64>() {
            nums.push(n.to_string());
        } else {
            break;
        }
    }
    if nums.is_empty() { v.to_string() } else { nums.join(".") }
}
fn cmp_semver(a: &str, b: &str) -> Ordering {
    let pa: Vec<u64> = normalise(a).split('.').filter_map(|x| x.parse().ok()).collect();
    let pb: Vec<u64> = normalise(b).split('.').filter_map(|x| x.parse().ok()).collect();
    let max_len = pa.len().max(pb.len()).max(3);
    for i in 0..max_len {
        let na = *pa.get(i).unwrap_or(&0);
        let nb = *pb.get(i).unwrap_or(&0);
        if na > nb { return Ordering::Greater; }
        if na < nb { return Ordering::Less; }
    }
    Ordering::Equal
}



#[cfg(feature = "slint")]
pub mod slint_helpers {
    use super::*;
    use open;

    
    pub trait InfoBoxLike {
        fn set_info_title(&self, s: slint::SharedString);
        fn set_info_message(&self, s: slint::SharedString);
        fn set_show_info(&self, v: f32);
    }

    pub fn check_and_inform<App: InfoBoxLike>(
        ui: &App,
        checker: &UpdateChecker,
        force: bool,
    ) -> Result<Option<ReleaseInfo>, UpdateError> {
        if let Some(info) = checker.check(force)? {
            ui.set_info_title("Update available".into());
            ui.set_info_message(
                format!("A new version is available: v{}\nYou are on v{}.\nSee the release page in your browser.",
                        info.tag, checker.current_version).into()
            );
            ui.set_show_info(1.0);
            return Ok(Some(info));
        }
        Ok(None)
    }

    pub trait UpdateBoxLike {
        fn set_update_title(&self, s: slint::SharedString);
        fn set_update_message(&self, s: slint::SharedString);
        fn set_update_url(&self, s: slint::SharedString);
        fn set_show_update(&self, v: f32);
    }

    pub fn check_with_confirm<App: UpdateBoxLike>(
        ui: &App,
        checker: &UpdateChecker,
        force: bool,
    ) -> Result<Option<ReleaseInfo>, UpdateError> {
        if let Some(info) = checker.check(force)? {
            ui.set_update_title("Update available".into());
            ui.set_update_message(
                format!("A new version is available: v{}\nYou are on v{}.\nOpen the download page?",
                        info.tag, checker.current_version).into()
            );
            ui.set_update_url(info.html_url.clone().into());
            ui.set_show_update(1.0);
            return Ok(Some(info));
        }
        Ok(None)
    }

    pub fn open_url(url: &str) {
        let _ = open::that(url);
    }


}