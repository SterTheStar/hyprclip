use crate::clipboard::ClipboardManager;
use crate::ui::clipboard_list::ClipboardList;
use adw::prelude::*;
use gtk::{self, glib, Overflow};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

pub struct MainWindow {
    window: adw::ApplicationWindow,
    list: ClipboardList,
    search_entry: gtk::SearchEntry,
}

impl MainWindow {
    pub fn new(app: &adw::Application, clipboard_manager: Rc<ClipboardManager>) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .default_width(900)
            .default_height(500)
            .hide_on_close(true)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_overflow(Overflow::Hidden);

        let win = window.clone();
        window.connect_realize(move |_| {
            if let Some(native) = win.native() {
                if let Some(surface) = native.surface() {
                    surface.set_opaque_region(None);
                }
            }
        });

        let list = ClipboardList::new(clipboard_manager);

        let search_entry = gtk::SearchEntry::builder()
            .hexpand(true)
            .placeholder_text("Search clipboard history...")
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(12)
            .margin_end(12)
            .build();

        let instance = Self {
            window,
            list,
            search_entry,
        };

        instance.setup_ui();
        instance.setup_signals();
        instance
    }

    fn setup_ui(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(
            "window {
                background-color: @window_bg_color;
                border-radius: 12px;
            }",
        );
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .build();

        main_box.append(&self.search_entry);

        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .build();

        scrolled.set_child(Some(self.list.widget()));
        main_box.append(&scrolled);

        self.window.set_content(Some(&main_box));

        let list = self.list.clone();
        self.search_entry
            .connect_search_changed(move |entry| {
                let query = entry.text().to_string();
                list.filter(&query);
            });
    }

    fn setup_signals(&self) {
        let window = self.window.clone();
        let clipboard_manager = self.list.clipboard_manager();
        self.list.widget().connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            let items = clipboard_manager.get_items();
            if let Some(item) = items.get(index) {
                clipboard_manager.set_clipboard(item);
            }
            window.set_visible(false);
        });

        let controller = gtk::EventControllerKey::new();
        let window = self.window.clone();
        controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                window.set_visible(false);
            }
            glib::Propagation::Proceed
        });
        self.window.add_controller(controller);

        let close_timeout: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));

        self.window.connect_is_active_notify(move |win| {
            if !win.is_active() {
                let timeout_ref = close_timeout.clone();
                let w = win.clone();
                let id = glib::timeout_add_local(Duration::from_secs(2), move || {
                    timeout_ref.set(None);
                    w.set_visible(false);
                    glib::ControlFlow::Break
                });
                close_timeout.set(Some(id));
            } else if let Some(id) = close_timeout.take() {
                id.remove();
            }
        });
    }

    pub fn show(&self) {
        self.window.present();
        self.search_entry.grab_focus();
    }
}
