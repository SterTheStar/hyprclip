use crate::models::ClipboardItem;
use crate::settings::Settings;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedItem {
    pub content: String,
    pub preview: String,
    pub item_type: String,
    pub timestamp: String,
    #[serde(default)]
    pub image_bytes: Option<String>,
}

impl PersistedItem {
    pub fn from_clipboard_item(item: &ClipboardItem) -> Self {
        use base64::Engine;
        Self {
            content: item.content().to_string(),
            preview: item.preview().to_string(),
            item_type: format!("{}", item.item_type()),
            timestamp: item.timestamp().to_rfc3339(),
            image_bytes: item.image_bytes().map(|bytes| {
                base64::engine::general_purpose::STANDARD.encode(bytes)
            }),
        }
    }

    pub fn to_clipboard_item(&self) -> Option<ClipboardItem> {
        use base64::Engine;
        match self.item_type.as_str() {
            "Text" => Some(ClipboardItem::new_text(&self.content, &self.preview)),
            "Image" => {
                if let Some(ref bytes_str) = self.image_bytes {
                    let bytes = base64::engine::general_purpose::STANDARD.decode(bytes_str).ok()?;
                    let glib_bytes = gtk::glib::Bytes::from(&bytes);
                    Some(ClipboardItem::new_image(&self.content, &self.preview, glib_bytes))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn age_hours(&self) -> f64 {
        use chrono::DateTime;
        if let Ok(dt) = DateTime::parse_from_rfc3339(&self.timestamp) {
            let now = chrono::Local::now();
            let duration = now.signed_duration_since(dt);
            duration.num_seconds() as f64 / 3600.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryData {
    pub created_at: String,
    pub items: Vec<PersistedItem>,
}

impl HistoryData {
    pub fn new(items: Vec<PersistedItem>) -> Self {
        Self {
            created_at: Local::now().to_rfc3339(),
            items,
        }
    }

    pub fn created_at_datetime(&self) -> Option<DateTime<Local>> {
        DateTime::parse_from_rfc3339(&self.created_at)
            .ok()
            .map(|dt| dt.with_timezone(&Local))
    }
}

pub struct Persistence {
    history_file: PathBuf,
}

impl Persistence {
    pub fn new() -> Self {
        let history_file = Settings::config_dir().join("history.json");
        Self { history_file }
    }

    fn load_raw(&self) -> Option<HistoryData> {
        if !self.history_file.exists() {
            return None;
        }

        match std::fs::read_to_string(&self.history_file) {
            Ok(content) => {
                match serde_json::from_str::<HistoryData>(&content) {
                    Ok(data) => Some(data),
                    Err(_) => {
                        match serde_json::from_str::<Vec<PersistedItem>>(&content) {
                            Ok(items) => Some(HistoryData::new(items)),
                            Err(e) => {
                                tracing::warn!("Failed to parse history file: {}", e);
                                None
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read history file: {}", e);
                None
            }
        }
    }

    pub fn load(&self, max_items: usize) -> Vec<ClipboardItem> {
        match self.load_raw() {
            Some(data) => {
                data.items
                    .iter()
                    .filter_map(|item| item.to_clipboard_item())
                    .take(max_items)
                    .collect()
            }
            None => Vec::new(),
        }
    }

    pub fn save(&self, items: &[ClipboardItem]) -> anyhow::Result<()> {
        let persisted: Vec<PersistedItem> = items
            .iter()
            .map(PersistedItem::from_clipboard_item)
            .collect();

        let data = match self.load_raw() {
            Some(mut existing) => {
                existing.items = persisted;
                existing
            }
            None => HistoryData::new(persisted),
        };

        let content = serde_json::to_string_pretty(&data)?;
        std::fs::write(&self.history_file, content)?;
        Ok(())
    }

    pub fn cleanup_by_age(&self, max_age_days: u64) -> anyhow::Result<()> {
        if max_age_days == 0 || !self.history_file.exists() {
            return Ok(());
        }

        let mut data = match self.load_raw() {
            Some(data) => data,
            None => return Ok(()),
        };

        let max_age_hours = max_age_days as f64 * 24.0;
        data.items.retain(|item| item.age_hours() < max_age_hours);

        let content = serde_json::to_string_pretty(&data)?;
        std::fs::write(&self.history_file, content)?;
        Ok(())
    }

    pub fn should_auto_clear(&self, auto_clear_hours: u64) -> bool {
        if auto_clear_hours == 0 || !self.history_file.exists() {
            return false;
        }

        match self.load_raw() {
            Some(data) => {
                match data.created_at_datetime() {
                    Some(created) => {
                        let now = Local::now();
                        let duration = now.signed_duration_since(created);
                        duration.num_seconds() >= auto_clear_hours as i64 * 3600
                    }
                    None => false,
                }
            }
            None => false,
        }
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        if self.history_file.exists() {
            std::fs::remove_file(&self.history_file)?;
        }
        Ok(())
    }
}
