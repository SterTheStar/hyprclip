use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    System,
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::System => write!(f, "system"),
            Theme::Light => write!(f, "light"),
            Theme::Dark => write!(f, "dark"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // Clipboard behavior
    #[serde(default = "default_max_items")]
    pub max_items: usize,

    #[serde(default)]
    pub persistence_enabled: bool,

    #[serde(default = "default_max_preview_length")]
    pub max_preview_length: usize,

    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,

    #[serde(default = "default_true")]
    pub hide_on_select: bool,

    #[serde(default = "default_true")]
    pub deduplicate: bool,

    // Appearance
    #[serde(default)]
    pub theme: Theme,

    #[serde(default = "default_font_size")]
    pub font_size: u32,

    #[serde(default = "default_window_width")]
    pub window_width: i32,

    #[serde(default = "default_window_height")]
    pub window_height: i32,

    // Images
    #[serde(default = "default_max_image_size_mb")]
    pub max_image_size_mb: f64,

    #[serde(default = "default_true")]
    pub save_images: bool,

    // Cleanup
    #[serde(default = "default_auto_clear_hours")]
    pub auto_clear_hours: u64,

    #[serde(default = "default_max_history_age_days")]
    pub max_history_age_days: u64,
}

fn default_max_items() -> usize {
    100
}

fn default_max_preview_length() -> usize {
    200
}

fn default_poll_interval_ms() -> u64 {
    500
}

fn default_true() -> bool {
    true
}

fn default_font_size() -> u32 {
    13
}

fn default_window_width() -> i32 {
    900
}

fn default_window_height() -> i32 {
    500
}

fn default_max_image_size_mb() -> f64 {
    10.0
}

fn default_auto_clear_hours() -> u64 {
    0
}

fn default_max_history_age_days() -> u64 {
    0
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_items: default_max_items(),
            persistence_enabled: false,
            max_preview_length: default_max_preview_length(),
            poll_interval_ms: default_poll_interval_ms(),
            hide_on_select: default_true(),
            deduplicate: default_true(),
            theme: Theme::default(),
            font_size: default_font_size(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            max_image_size_mb: default_max_image_size_mb(),
            save_images: default_true(),
            auto_clear_hours: default_auto_clear_hours(),
            max_history_age_days: default_max_history_age_days(),
        }
    }
}

impl Settings {
    pub fn config_dir() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("hyprclip")
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_file();

        // Migrate from old "config" file if it exists
        if !path.exists() {
            let old_path = Self::config_dir().join("config");
            if old_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&old_path) {
                    if let Ok(settings) = toml::from_str::<Settings>(&content) {
                        let _ = settings.save();
                        let _ = std::fs::remove_file(&old_path);
                        tracing::info!("Migrated config from 'config' to 'config.toml'");
                        return settings;
                    }
                }
            }
        }

        if !path.exists() {
            let settings = Self::default();
            let _ = settings.save();
            return settings;
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<Settings>(&content) {
                Ok(settings) => settings,
                Err(e) => {
                    tracing::warn!("Failed to parse config file: {}. Using defaults.", e);
                    Self::default()
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read config file: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;

        let content = self.generate_config_file();
        std::fs::write(Self::config_file(), content)?;
        Ok(())
    }

    fn generate_config_file(&self) -> String {
        let persistence = if self.persistence_enabled { "true" } else { "false" };
        let hide_on_select = if self.hide_on_select { "true" } else { "false" };
        let deduplicate = if self.deduplicate { "true" } else { "false" };
        let save_images = if self.save_images { "true" } else { "false" };

        format!(r#"#  _    _                      _ _       
# | |  | |                    | (_)      
# | |__| |_   _ _ __  _ __ ___| |_ _ __  
# |  __  | | | | '_ \| '__/ __| | | '_ \ 
# | |  | | |_| | |_) | | | (__| | | |_) |
# |_|  |_|\__, | .__/|_|  \___|_|_| .__/ 
#          __/ | |                | |    
#         |___/|_|                |_|    
#
#  Configuration file for hyprclip
#  https://github.com/SterTheStar/hyprclip
#


# =============================================================================
#  CLIPBOARD BEHAVIOR
# =============================================================================

# Maximum number of items stored in clipboard history
# Default: 100
max_items = {max_items}

# Save clipboard history between sessions
# When enabled, history is saved to ~/.config/hyprclip/history.json
# Default: false
persistence_enabled = {persistence}

# Maximum character length for text preview in the list
# Default: 200
max_preview_length = {max_preview_length}

# How often to check the clipboard for changes (in milliseconds)
# Lower values = more responsive, higher values = less CPU usage
# Default: 500
poll_interval_ms = {poll_interval_ms}

# Hide the window after selecting an item
# Default: true
hide_on_select = {hide_on_select}

# Remove duplicate entries automatically
# When true, copying the same text moves it to the top instead of duplicating
# Default: true
deduplicate = {deduplicate}


# =============================================================================
#  APPEARANCE
# =============================================================================

# Color theme
# Options: "system", "light", "dark"
# Default: "system"
theme = "{theme}"

# Font size in pixels for labels
# Default: 13
font_size = {font_size}

# Window dimensions
# Default: 900x500
window_width = {window_width}
window_height = {window_height}


# =============================================================================
#  IMAGES
# =============================================================================

# Save images to clipboard history
# Default: true
save_images = {save_images}

# Maximum image size in megabytes
# Images larger than this are silently ignored
# Default: 10.0
max_image_size_mb = {max_image_size_mb}


# =============================================================================
#  CLEANUP
# =============================================================================

# Auto-clear history after this many hours of inactivity
# Set to 0 to disable
# Works based on real time (includes time while PC is off)
# Default: 0 (disabled)
auto_clear_hours = {auto_clear_hours}

# Delete individual items older than this many days
# Set to 0 to disable
# Default: 0 (disabled)
max_history_age_days = {max_history_age_days}
"#,
            max_items = self.max_items,
            persistence = persistence,
            max_preview_length = self.max_preview_length,
            poll_interval_ms = self.poll_interval_ms,
            hide_on_select = hide_on_select,
            deduplicate = deduplicate,
            theme = self.theme,
            font_size = self.font_size,
            window_width = self.window_width,
            window_height = self.window_height,
            save_images = save_images,
            max_image_size_mb = self.max_image_size_mb,
            auto_clear_hours = self.auto_clear_hours,
            max_history_age_days = self.max_history_age_days,
        )
    }

    pub fn config_modified() -> Option<SystemTime> {
        Self::config_file()
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
    }
}
