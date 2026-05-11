use chrono::{DateTime, Local};
use gtk::glib;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ClipboardItemType {
    Text,
    Image,
}

impl fmt::Display for ClipboardItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardItemType::Text => write!(f, "Text"),
            ClipboardItemType::Image => write!(f, "Image"),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClipboardItem {
    id: u64,
    content: String,
    preview: String,
    item_type: ClipboardItemType,
    timestamp: DateTime<Local>,
    image_bytes: Option<glib::Bytes>,
}

impl ClipboardItem {
    pub fn new_text(content: &str, preview: &str) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            content: content.to_string(),
            preview: preview.to_string(),
            item_type: ClipboardItemType::Text,
            timestamp: Local::now(),
            image_bytes: None,
        }
    }

    pub fn new_image(dimensions: &str, preview: &str, image_bytes: glib::Bytes) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            content: dimensions.to_string(),
            preview: preview.to_string(),
            item_type: ClipboardItemType::Image,
            timestamp: Local::now(),
            image_bytes: Some(image_bytes),
        }
    }

    pub fn image_bytes(&self) -> Option<&glib::Bytes> {
        self.image_bytes.as_ref()
    }

    #[allow(dead_code)]
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn preview(&self) -> &str {
        &self.preview
    }

    pub fn item_type(&self) -> &ClipboardItemType {
        &self.item_type
    }

    pub fn timestamp(&self) -> &DateTime<Local> {
        &self.timestamp
    }

    pub fn formatted_time(&self) -> String {
        self.timestamp.format("%H:%M:%S").to_string()
    }

    #[allow(dead_code)]
    pub fn age_hours(&self) -> f64 {
        let now = Local::now();
        let duration = now.signed_duration_since(self.timestamp);
        duration.num_seconds() as f64 / 3600.0
    }

    #[allow(dead_code)]
    pub fn image_size_mb(&self) -> f64 {
        self.image_bytes
            .as_ref()
            .map(|b| b.len() as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0)
    }
}
