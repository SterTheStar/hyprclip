use crate::clipboard::ClipboardManager;
use crate::models::{ClipboardItem, ClipboardItemType};
use adw::prelude::*;
use gtk::gdk;
use std::rc::Rc;

#[derive(Clone)]
pub struct ClipboardList {
    list_box: gtk::ListBox,
    clipboard_manager: Rc<ClipboardManager>,
}

impl ClipboardList {
    pub fn new(clipboard_manager: Rc<ClipboardManager>) -> Self {
        let list_box = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .build();

        let instance = Self {
            list_box,
            clipboard_manager,
        };

        instance.setup_signals();
        instance.setup_auto_refresh();
        instance.refresh();
        instance
    }

    fn setup_signals(&self) {
        // row-activated is handled by MainWindow
    }

    fn setup_auto_refresh(&self) {
        let list = self.clone();
        self.clipboard_manager.set_on_change(move || {
            list.refresh();
        });
    }

    pub fn refresh(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let items = self.clipboard_manager.get_items();
        for item in items {
            self.add_item_row(&item);
        }
    }

    pub fn filter(&self, query: &str) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let items = self.clipboard_manager.search(query);
        for item in items {
            self.add_item_row(&item);
        }
    }

    fn add_item_row(&self, item: &ClipboardItem) {
        let row = gtk::ListBoxRow::builder().build();

        let hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(12)
            .margin_end(12)
            .build();

        let icon: gtk::Widget = match item.item_type() {
            ClipboardItemType::Text => {
                gtk::Image::builder()
                    .icon_name("text-x-generic-symbolic")
                    .pixel_size(24)
                    .build()
                    .upcast()
            }
            ClipboardItemType::Image => {
                if let Some(bytes) = item.image_bytes() {
                    if let Ok(texture) = gdk::Texture::from_bytes(bytes) {
                        let img = gtk::Image::from_paintable(Some(&texture));
                        img.set_pixel_size(35);
                        img.upcast()
                    } else {
                        gtk::Image::builder()
                            .icon_name("image-x-generic-symbolic")
                            .pixel_size(24)
                            .build()
                            .upcast()
                    }
                } else {
                    gtk::Image::builder()
                        .icon_name("image-x-generic-symbolic")
                        .pixel_size(24)
                        .build()
                        .upcast()
                }
            }
        };
        hbox.append(&icon);

        let content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let content_label = gtk::Label::builder()
            .label(item.preview())
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();
        content_box.append(&content_label);

        let meta_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .build();

        let type_label = gtk::Label::builder()
            .label(&format!("{}", item.item_type()))
            .build();
        meta_box.append(&type_label);

        let time_label = gtk::Label::builder()
            .label(&item.formatted_time())
            .build();
        meta_box.append(&time_label);

        content_box.append(&meta_box);
        hbox.append(&content_box);

        row.set_child(Some(&hbox));
        self.list_box.append(&row);
    }

    pub fn widget(&self) -> &gtk::ListBox {
        &self.list_box
    }

    #[allow(dead_code)]
    pub fn clipboard_manager(&self) -> Rc<ClipboardManager> {
        self.clipboard_manager.clone()
    }
}
