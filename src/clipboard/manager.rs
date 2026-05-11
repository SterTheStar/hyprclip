use crate::models::ClipboardItem;
use crate::models::ClipboardItemType;
use crate::persistence::Persistence;
use crate::settings::Settings;
use gtk::gdk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

pub struct ClipboardManager {
    items: Rc<RefCell<Vec<ClipboardItem>>>,
    display: gdk::Display,
    on_change: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_settings_change: Rc<RefCell<Option<Box<dyn Fn(&Settings)>>>>,
    last_text: Rc<RefCell<String>>,
    settings: Rc<RefCell<Settings>>,
    persistence: Rc<Persistence>,
    config_modified: Rc<RefCell<Option<SystemTime>>>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        let display = gdk::Display::default().expect("Could not get default display");
        let settings = Settings::load();
        let persistence = Persistence::new();
        let config_modified = Settings::config_modified();

        if settings.persistence_enabled {
            if persistence.should_auto_clear(settings.auto_clear_hours) {
                let _ = persistence.clear();
            }
            if settings.max_history_age_days > 0 {
                let _ = persistence.cleanup_by_age(settings.max_history_age_days);
            }
        }

        let items = if settings.persistence_enabled {
            Rc::new(RefCell::new(persistence.load(settings.max_items)))
        } else {
            Rc::new(RefCell::new(Vec::new()))
        };

        let manager = Self {
            items,
            display,
            on_change: Rc::new(RefCell::new(None)),
            on_settings_change: Rc::new(RefCell::new(None)),
            last_text: Rc::new(RefCell::new(String::new())),
            settings: Rc::new(RefCell::new(settings)),
            persistence: Rc::new(persistence),
            config_modified: Rc::new(RefCell::new(config_modified)),
        };

        manager.start_monitoring();
        manager.start_settings_watcher();
        manager
    }

    pub fn set_on_change<F: Fn() + 'static>(&self, callback: F) {
        *self.on_change.borrow_mut() = Some(Box::new(callback));
    }

    pub fn set_on_settings_change<F: Fn(&Settings) + 'static>(&self, callback: F) {
        *self.on_settings_change.borrow_mut() = Some(Box::new(callback));
    }

    fn start_settings_watcher(&self) {
        let settings = self.settings.clone();
        let config_modified = self.config_modified.clone();
        let on_settings_change = self.on_settings_change.clone();
        let on_change = self.on_change.clone();

        glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
            let current_modified = Settings::config_modified();
            let last_modified = *config_modified.borrow();

            if current_modified != last_modified {
                *config_modified.borrow_mut() = current_modified;

                let new_settings = Settings::load();
                *settings.borrow_mut() = new_settings.clone();

                tracing::info!("Config file changed, reloaded settings");

                if let Some(ref cb) = *on_settings_change.borrow() {
                    cb(&new_settings);
                }

                if let Some(ref cb) = *on_change.borrow() {
                    cb();
                }
            }

            glib::ControlFlow::Continue
        });
    }

    fn start_monitoring(&self) {
        let clipboard = self.display.clipboard();
        let poll_interval = self.settings.borrow().poll_interval_ms;

        {
            let items = self.items.clone();
            let on_change = self.on_change.clone();
            let last_text = self.last_text.clone();
            let settings = self.settings.clone();
            let persistence = self.persistence.clone();
            clipboard.connect_changed(move |clipboard| {
                Self::poll_clipboard(clipboard, &items, &on_change, &last_text, &settings, &persistence);
            });
        }

        {
            let items = self.items.clone();
            let on_change = self.on_change.clone();
            let last_text = self.last_text.clone();
            let settings = self.settings.clone();
            let persistence = self.persistence.clone();
            clipboard.connect_content_notify(move |clipboard| {
                Self::poll_clipboard(clipboard, &items, &on_change, &last_text, &settings, &persistence);
            });
        }

        {
            let clipboard = self.display.clipboard();
            let items = self.items.clone();
            let on_change = self.on_change.clone();
            let last_text = self.last_text.clone();
            let settings = self.settings.clone();
            let persistence = self.persistence.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(poll_interval), move || {
                Self::poll_clipboard(&clipboard, &items, &on_change, &last_text, &settings, &persistence);
                glib::ControlFlow::Continue
            });
        }
    }

    fn poll_clipboard(
        clipboard: &gdk::Clipboard,
        items: &Rc<RefCell<Vec<ClipboardItem>>>,
        on_change: &Rc<RefCell<Option<Box<dyn Fn()>>>>,
        last_text: &Rc<RefCell<String>>,
        settings: &Rc<RefCell<Settings>>,
        persistence: &Rc<Persistence>,
    ) {
        let formats = clipboard.formats();

        let has_text = formats.contain_mime_type("text/plain")
            || formats.contain_mime_type("text/plain;charset=utf-8");
        let has_image = formats.contains_type(gdk::Texture::static_type());

        if has_text {
            let items = items.clone();
            let on_change = on_change.clone();
            let last_text = last_text.clone();
            let settings = settings.clone();
            let persistence = persistence.clone();
            clipboard.read_text_async(None::<&gtk::gio::Cancellable>, move |result| {
                if let Ok(Some(text)) = result {
                    let content = text.to_string();
                    if content.is_empty() {
                        return;
                    }

                    let mut last = last_text.borrow_mut();
                    if *last == content {
                        return;
                    }
                    *last = content.clone();
                    drop(last);

                    let s = settings.borrow();
                    let max_preview = s.max_preview_length;
                    let max_items = s.max_items;
                    let deduplicate = s.deduplicate;
                    let persist = s.persistence_enabled;
                    drop(s);

                    let preview = if content.len() > max_preview {
                        format!("{}...", &content[..max_preview])
                    } else {
                        content.clone()
                    };

                    let mut items = items.borrow_mut();

                    if deduplicate {
                        items.retain(|i| i.content() != content);
                    }

                    items.insert(0, ClipboardItem::new_text(&content, &preview));
                    if items.len() > max_items {
                        items.truncate(max_items);
                    }

                    if persist {
                        if let Err(e) = persistence.save(&items) {
                            tracing::warn!("Failed to save clipboard history: {}", e);
                        }
                    }

                    drop(items);

                    if let Some(ref cb) = *on_change.borrow() {
                        cb();
                    }
                }
            });
        } else if has_image {
            let items = items.clone();
            let on_change = on_change.clone();
            let settings = settings.clone();
            let persistence = persistence.clone();
            clipboard.read_texture_async(None::<&gtk::gio::Cancellable>, move |result| {
                if let Ok(Some(texture)) = result {
                    let s = settings.borrow();
                    let max_items = s.max_items;
                    let save_images = s.save_images;
                    let max_image_size = s.max_image_size_mb;
                    let deduplicate = s.deduplicate;
                    let persist = s.persistence_enabled;
                    drop(s);

                    if !save_images {
                        return;
                    }

                    let width = texture.width();
                    let height = texture.height();
                    let dimensions = format!("{}x{}", width, height);
                    let preview = format!("Image {}x{}", width, height);
                    let bytes = texture.save_to_png_bytes();

                    let size_mb = bytes.len() as f64 / (1024.0 * 1024.0);
                    if size_mb > max_image_size {
                        tracing::warn!("Image too large ({:.2} MB), skipping", size_mb);
                        return;
                    }

                    let mut items = items.borrow_mut();

                    if deduplicate {
                        let is_dup = items.iter().any(|i| {
                            i.item_type() == &ClipboardItemType::Image
                                && i.content() == dimensions
                        });
                        if is_dup {
                            return;
                        }
                    }

                    items.insert(0, ClipboardItem::new_image(&dimensions, &preview, bytes));
                    if items.len() > max_items {
                        items.truncate(max_items);
                    }

                    if persist {
                        if let Err(e) = persistence.save(&items) {
                            tracing::warn!("Failed to save clipboard history: {}", e);
                        }
                    }

                    drop(items);

                    if let Some(ref cb) = *on_change.borrow() {
                        cb();
                    }
                }
            });
        }
    }

    pub fn get_items(&self) -> Vec<ClipboardItem> {
        self.items.borrow().clone()
    }

    pub fn search(&self, query: &str) -> Vec<ClipboardItem> {
        let items = self.items.borrow();
        if query.is_empty() {
            return items.clone();
        }

        let q = query.to_lowercase();
        items
            .iter()
            .filter(|item| {
                item.content().to_lowercase().contains(&q)
                    || item.preview().to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }

    pub fn set_clipboard(&self, item: &ClipboardItem) {
        let clipboard = self.display.clipboard();
        match item.item_type() {
            ClipboardItemType::Text => {
                clipboard.set_text(item.content());
            }
            ClipboardItemType::Image => {
                if let Some(bytes) = item.image_bytes() {
                    if let Ok(texture) = gdk::Texture::from_bytes(bytes) {
                        clipboard.set_texture(&texture);
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        self.items.borrow_mut().clear();
        if self.settings.borrow().persistence_enabled {
            if let Err(e) = self.persistence.clear() {
                tracing::warn!("Failed to clear history file: {}", e);
            }
        }
        if let Some(ref cb) = *self.on_change.borrow() {
            cb();
        }
    }

    #[allow(dead_code)]
    pub fn settings(&self) -> Rc<RefCell<Settings>> {
        self.settings.clone()
    }
}
