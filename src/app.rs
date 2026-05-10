use crate::clipboard::ClipboardManager;
use crate::ui::main_window::MainWindow;
use adw::prelude::*;
use gtk::{gio, glib};
use std::cell::{Cell, RefCell};
use std::io::Read;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::rc::Rc;

fn socket_path() -> PathBuf {
    let mut path = dirs_next::runtime_dir()
        .or_else(|| dirs_next::home_dir().map(|h| h.join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push("hyprclip.sock");
    path
}

pub struct HyprclipApp {
    app: adw::Application,
    _hold: Option<gio::ApplicationHoldGuard>,
}

impl HyprclipApp {
    pub fn new(gui: bool) -> Self {
        let app = adw::Application::builder()
            .application_id("com.github.hyprclip")
            .flags(gio::ApplicationFlags::FLAGS_NONE)
            .build();

        let window: Rc<RefCell<Option<MainWindow>>> = Rc::new(RefCell::new(None));
        let cm: Rc<RefCell<Option<Rc<ClipboardManager>>>> = Rc::new(RefCell::new(None));
        let toggle_request = Rc::new(Cell::new(gui));

        let _hold = if !gui {
            let path = socket_path();
            let _ = std::fs::remove_file(&path);
            if let Ok(listener) = UnixListener::bind(&path) {
                listener.set_nonblocking(true).ok();
                let tr = toggle_request.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    if let Ok((mut stream, _)) = listener.accept() {
                        let mut buf = [0u8; 16];
                        if stream.read(&mut buf).is_ok() {
                            let msg = std::str::from_utf8(&buf).unwrap_or("");
                            if msg.starts_with("toggle") || msg.starts_with("show") {
                                tr.set(true);
                            }
                        }
                    }
                    glib::ControlFlow::Continue
                });
            }
            Some(app.hold())
        } else {
            None
        };

        {
            let toggle_request = toggle_request.clone();
            let window = window.clone();
            let cm = cm.clone();
            app.connect_activate(move |app| {
                let mut cm = cm.borrow_mut();
                if cm.is_none() {
                    *cm = Some(Rc::new(ClipboardManager::new()));
                }
                let clipboard_manager = cm.as_ref().unwrap().clone();
                drop(cm);

                let mut win = window.borrow_mut();
                if let Some(ref w) = *win {
                    if toggle_request.get() {
                        if w.is_visible() {
                            w.hide();
                        } else {
                            w.show();
                        }
                        toggle_request.set(false);
                    }
                    return;
                }

                let w = MainWindow::new(app, clipboard_manager);
                if toggle_request.get() {
                    w.show();
                    toggle_request.set(false);
                }
                *win = Some(w);
            });
        }

        {
            let tr = toggle_request.clone();
            let window = window.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
                if tr.get() {
                    if let Some(ref w) = *window.borrow() {
                        if w.is_visible() {
                            w.hide();
                        } else {
                            w.show();
                        }
                        tr.set(false);
                    }
                }
                glib::ControlFlow::Continue
            });
        }

        Self { app, _hold }
    }

    pub fn is_running() -> bool {
        let path = socket_path();
        UnixStream::connect(&path).is_ok()
    }

    pub fn show_running_instance() -> bool {
        let path = socket_path();
        if let Ok(mut stream) = UnixStream::connect(&path) {
            use std::io::Write;
            let _ = stream.write_all(b"toggle");
            return true;
        }
        false
    }

    pub fn run_with_args(&self, args: &[&str]) -> glib::ExitCode {
        self.app.run_with_args(args)
    }
}
