mod app;
mod clipboard;
mod models;
mod ui;
mod utils;

use app::HyprclipApp;
use gtk::glib;

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--version" || a == "--v" || a == "-v") {
        println!("hyprclip {}", env!("CARGO_PKG_VERSION"));
        println!("https://github.com/SterTheStar/hyprclip");
        return glib::ExitCode::SUCCESS;
    }

    let gui = args.iter().any(|a| a == "--gui");

    if gui {
        if HyprclipApp::show_running_instance() {
            return glib::ExitCode::SUCCESS;
        }
    } else if HyprclipApp::is_running() {
        eprintln!("hyprclip is already running in the background.");
        eprintln!("Use 'hyprclip --gui' to toggle the clipboard popup.");
        return glib::ExitCode::SUCCESS;
    }

    let filtered: Vec<&str> = args.iter().filter(|a| *a != "--gui").map(|s| s.as_str()).collect();
    let app = HyprclipApp::new(gui);
    app.run_with_args(&filtered)
}
