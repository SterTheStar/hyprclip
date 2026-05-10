use crate::models::ClipboardItem;
use crate::models::ClipboardItemType;
use gtk::gdk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ClipboardManager {
    items: Rc<RefCell<Vec<ClipboardItem>>>,
    display: gdk::Display,
    on_change: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    last_text: Rc<RefCell<String>>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        let display = gdk::Display::default().expect("Could not get default display");
        let items = Rc::new(RefCell::new(Vec::new()));

        let manager = Self {
            items,
            display,
            on_change: Rc::new(RefCell::new(None)),
            last_text: Rc::new(RefCell::new(String::new())),
        };

        manager.start_monitoring();
        manager
    }

    pub fn set_on_change<F: Fn() + 'static>(&self, callback: F) {
        *self.on_change.borrow_mut() = Some(Box::new(callback));
    }

    fn start_monitoring(&self) {
        let clipboard = self.display.clipboard();

        // signal 1: clipboard owner changed
        {
            let items = self.items.clone();
            let on_change = self.on_change.clone();
            let last_text = self.last_text.clone();
            clipboard.connect_changed(move |clipboard| {
                Self::poll_clipboard(clipboard, &items, &on_change, &last_text);
            });
        }

        // signal 2: content changed (more granular)
        {
            let items = self.items.clone();
            let on_change = self.on_change.clone();
            let last_text = self.last_text.clone();
            clipboard.connect_content_notify(move |clipboard| {
                Self::poll_clipboard(clipboard, &items, &on_change, &last_text);
            });
        }

        // fallback: periodic polling every 500ms
        {
            let clipboard = self.display.clipboard();
            let items = self.items.clone();
            let on_change = self.on_change.clone();
            let last_text = self.last_text.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                Self::poll_clipboard(&clipboard, &items, &on_change, &last_text);
                glib::ControlFlow::Continue
            });
        }
    }

    fn poll_clipboard(
        clipboard: &gdk::Clipboard,
        items: &Rc<RefCell<Vec<ClipboardItem>>>,
        on_change: &Rc<RefCell<Option<Box<dyn Fn()>>>>,
        last_text: &Rc<RefCell<String>>,
    ) {
        let formats = clipboard.formats();

        let has_text = formats.contain_mime_type("text/plain")
            || formats.contain_mime_type("text/plain;charset=utf-8");
        let has_image = formats.contains_type(gdk::Texture::static_type());

        if has_text {
            let items = items.clone();
            let on_change = on_change.clone();
            let last_text = last_text.clone();
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

                    let preview = if content.len() > 200 {
                        format!("{}...", &content[..200])
                    } else {
                        content.clone()
                    };

                    let mut items = items.borrow_mut();
                    items.insert(0, ClipboardItem::new_text(&content, &preview));
                    if items.len() > 200 {
                        items.pop();
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
            clipboard.read_texture_async(None::<&gtk::gio::Cancellable>, move |result| {
                if let Ok(Some(texture)) = result {
                    let width = texture.width();
                    let height = texture.height();
                    let dimensions = format!("{}x{}", width, height);
                    let preview = format!("Image {}x{}", width, height);
                    let bytes = texture.save_to_png_bytes();

                    let mut items = items.borrow_mut();
                    let is_dup = items.first().map(|i| {
                        i.item_type() == &ClipboardItemType::Image
                            && i.content() == dimensions
                    }).unwrap_or(false);

                    if !is_dup {
                        items.insert(0, ClipboardItem::new_image(&dimensions, &preview, bytes));
                        if items.len() > 200 {
                            items.pop();
                        }
                        drop(items);

                        if let Some(ref cb) = *on_change.borrow() {
                            cb();
                        }
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
        if let Some(ref cb) = *self.on_change.borrow() {
            cb();
        }
    }
}
