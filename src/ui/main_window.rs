use crate::clipboard::ClipboardManager;
use crate::settings::{Settings, Theme};
use crate::ui::clipboard_list::ClipboardList;
use adw::prelude::*;
use gtk::{self, glib, Overflow};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

pub struct MainWindow {
    window: adw::ApplicationWindow,
    list: ClipboardList,
    search_entry: gtk::SearchEntry,
    settings: Rc<RefCell<Settings>>,
    css_provider: gtk::CssProvider,
}

impl MainWindow {
    pub fn new(app: &adw::Application, clipboard_manager: Rc<ClipboardManager>) -> Self {
        let settings = clipboard_manager.settings();
        let s = settings.borrow();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .default_width(s.window_width)
            .default_height(s.window_height)
            .hide_on_close(true)
            .build();

        Self::apply_theme(&s.theme);

        drop(s);

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

        let list = ClipboardList::new(clipboard_manager.clone());

        let search_entry = gtk::SearchEntry::builder()
            .hexpand(true)
            .placeholder_text("Search clipboard history...")
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(12)
            .margin_end(12)
            .build();

        let css_provider = gtk::CssProvider::new();

        let instance = Self {
            window,
            list,
            search_entry,
            settings: settings.clone(),
            css_provider,
        };

        instance.setup_ui();
        instance.setup_signals(clipboard_manager.clone());
        instance.setup_hot_reload(clipboard_manager);
        instance
    }

    fn apply_theme(theme: &Theme) {
        let style_manager = adw::StyleManager::default();
        match theme {
            Theme::System => {
                style_manager.set_color_scheme(adw::ColorScheme::Default);
            }
            Theme::Light => {
                style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
            }
            Theme::Dark => {
                style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
            }
        }
    }

    fn setup_ui(&self) {
        let font_size = self.settings.borrow().font_size;

        let css = format!(
            "window {{
                background-color: @window_bg_color;
                border-radius: 12px;
            }}
            label {{
                font-size: {}px;
            }}",
            font_size
        );
        self.css_provider.load_from_string(&css);
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().unwrap(),
            &self.css_provider,
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

    fn setup_hot_reload(&self, clipboard_manager: Rc<ClipboardManager>) {
        let css_provider = self.css_provider.clone();
        let window = self.window.clone();
        let settings = self.settings.clone();

        clipboard_manager.set_on_settings_change(move |new_settings| {
            Self::apply_theme(&new_settings.theme);

            let css = format!(
                "window {{
                    background-color: @window_bg_color;
                    border-radius: 12px;
                }}
                label {{
                    font-size: {}px;
                }}",
                new_settings.font_size
            );
            css_provider.load_from_string(&css);

            window.set_default_size(new_settings.window_width, new_settings.window_height);

            *settings.borrow_mut() = new_settings.clone();

            tracing::info!("Applied new settings: theme={}, font_size={}, window={}x{}",
                new_settings.theme, new_settings.font_size,
                new_settings.window_width, new_settings.window_height);
        });
    }

    fn setup_signals(&self, clipboard_manager: Rc<ClipboardManager>) {
        let window = self.window.clone();
        let settings = self.settings.clone();
        let cm = clipboard_manager.clone();
        self.list.widget().connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            let items = cm.get_items();
            if let Some(item) = items.get(index) {
                cm.set_clipboard(item);
            }
            if settings.borrow().hide_on_select {
                window.set_visible(false);
            }
        });

        let controller = gtk::EventControllerKey::new();
        let window = self.window.clone();
        let list_widget = self.list.widget().clone();
        let search_entry = self.search_entry.clone();
        controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                gtk::gdk::Key::Escape => {
                    window.set_visible(false);
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Down => {
                    let list = &list_widget;
                    let selected = list.selected_row();
                    let first_child = list.first_child();
                    match selected {
                        Some(row) => {
                            let next = row.next_sibling();
                            if let Some(next_row) = next {
                                if let Ok(list_row) = next_row.downcast::<gtk::ListBoxRow>() {
                                    list.select_row(Some(&list_row));
                                    list_row.grab_focus();
                                }
                            }
                        }
                        None => {
                            if let Some(first) = first_child {
                                if let Ok(list_row) = first.downcast::<gtk::ListBoxRow>() {
                                    list.select_row(Some(&list_row));
                                    list_row.grab_focus();
                                }
                            }
                        }
                    }
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Up => {
                    let list = &list_widget;
                    let selected = list.selected_row();
                    if let Some(row) = selected {
                        let prev = row.prev_sibling();
                        if let Some(prev_row) = prev {
                            if let Ok(list_row) = prev_row.downcast::<gtk::ListBoxRow>() {
                                list.select_row(Some(&list_row));
                                list_row.grab_focus();
                            }
                        }
                    }
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter => {
                    let list = &list_widget;
                    if let Some(row) = list.selected_row() {
                        row.activate();
                    }
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Tab => {
                    search_entry.grab_focus();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        self.window.add_controller(controller);

        let list_widget = self.list.widget().clone();
        let search_controller = gtk::EventControllerKey::new();
        search_controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                gtk::gdk::Key::Down => {
                    let list = &list_widget;
                    let selected = list.selected_row();
                    let first_child = list.first_child();
                    match selected {
                        Some(row) => {
                            let next = row.next_sibling();
                            if let Some(next_row) = next {
                                if let Ok(list_row) = next_row.downcast::<gtk::ListBoxRow>() {
                                    list.select_row(Some(&list_row));
                                    list_row.grab_focus();
                                }
                            }
                        }
                        None => {
                            if let Some(first) = first_child {
                                if let Ok(list_row) = first.downcast::<gtk::ListBoxRow>() {
                                    list.select_row(Some(&list_row));
                                    list_row.grab_focus();
                                }
                            }
                        }
                    }
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        self.search_entry.add_controller(search_controller);

        let focus_controller = gtk::EventControllerFocus::new();
        let window = self.window.clone();
        let focus_timeout: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        let focus_timeout_leave = focus_timeout.clone();
        focus_controller.connect_leave(move |_| {
            let window = window.clone();
            let timeout = focus_timeout_leave.clone();
            let source_id = glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
                window.set_visible(false);
                timeout.borrow_mut().take();
                glib::ControlFlow::Break
            });
            *focus_timeout_leave.borrow_mut() = Some(source_id);
        });
        let focus_timeout_enter = focus_timeout.clone();
        focus_controller.connect_enter(move |_| {
            if let Some(source_id) = focus_timeout_enter.borrow_mut().take() {
                source_id.remove();
            }
        });
        self.window.add_controller(focus_controller);
    }

    pub fn show(&self) {
        self.window.present();
        self.search_entry.grab_focus();
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    pub fn is_visible(&self) -> bool {
        self.window.is_visible()
    }
}
