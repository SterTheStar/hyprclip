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
}

impl HyprclipApp {
    pub fn new(gui: bool) -> Self {
        let app = adw::Application::builder()
            .application_id("com.github.hyprclip")
            .flags(gio::ApplicationFlags::FLAGS_NONE)
            .build();

        let window: Rc<RefCell<Option<MainWindow>>> = Rc::new(RefCell::new(None));
        let cm: Rc<RefCell<Option<Rc<ClipboardManager>>>> = Rc::new(RefCell::new(None));
        let show_request = Rc::new(Cell::new(gui));

        if !gui {
            let path = socket_path();
            let _ = std::fs::remove_file(&path);
            if let Ok(listener) = UnixListener::bind(&path) {
                listener.set_nonblocking(true).ok();
                let sr = show_request.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    if let Ok((mut stream, _)) = listener.accept() {
                        let mut buf = [0u8; 16];
                        if stream.read(&mut buf).is_ok() {
                            let msg = std::str::from_utf8(&buf).unwrap_or("");
                            if msg.starts_with("show") {
                                sr.set(true);
                            }
                        }
                    }
                    glib::ControlFlow::Continue
                });
            }
        }

        {
            let show_request = show_request.clone();
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
                    if show_request.get() {
                        w.show();
                        show_request.set(false);
                    }
                    return;
                }

                let w = MainWindow::new(app, clipboard_manager);
                if show_request.get() {
                    w.show();
                    show_request.set(false);
                }
                *win = Some(w);
            });
        }

        {
            let sr = show_request.clone();
            let window = window.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
                if sr.get() {
                    if let Some(ref w) = *window.borrow() {
                        w.show();
                        sr.set(false);
                    }
                }
                glib::ControlFlow::Continue
            });
        }

        Self { app }
    }

    pub fn show_running_instance() -> bool {
        let path = socket_path();
        if let Ok(mut stream) = UnixStream::connect(&path) {
            use std::io::Write;
            let _ = stream.write_all(b"show");
            return true;
        }
        false
    }

    pub fn run_with_args(&self, args: &[&str]) -> glib::ExitCode {
        self.app.run_with_args(args)
    }
}
