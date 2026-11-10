#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod context;
mod effects;
mod settings;
mod shortcuts;
mod tool;
mod ui;

use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

const APP_ID: &str = "org.conservation.Editor";

fn main() -> gtk4::glib::ExitCode {
    // Detach from the terminal in release builds (`windows_subsystem` does
    // not apply on Linux).
    #[cfg(all(not(debug_assertions), target_os = "linux"))]
    {
        use std::process;
        // SAFETY: fork before any GTK/GLib state is created.
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            // Fork failed; continue in the foreground.
        } else if pid > 0 {
            // Parent: hand off to the child and exit.
            process::exit(0);
        } else {
            // Child: become a session leader and detach from the controlling
            // terminal, then drop the inherited standard streams.
            unsafe {
                libc::setsid();
            }
            if let Ok(devnull) = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/null")
            {
                use std::os::unix::io::AsRawFd;
                let fd = devnull.as_raw_fd();
                unsafe {
                    libc::dup2(fd, 0);
                    libc::dup2(fd, 1);
                    libc::dup2(fd, 2);
                }
            }
        }
    }

    #[cfg(windows)]
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let schema_dir = exe_dir.join("share/glib-2.0/schemas");
            unsafe {
                std::env::set_var("GSETTINGS_SCHEMA_DIR", &schema_dir);
            }
        }
    }

    let app = gtk4::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        let display = gtk4::gdk::Display::default().expect("Could not connect to a display.");

        let icon_theme = gtk4::IconTheme::for_display(&display);
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            icon_theme.add_search_path(exe_dir.join("share/icons"));
        }
        let settings = gtk4::Settings::for_display(&display);
        settings.set_gtk_icon_theme_name(Some("Adwaita"));
        gtk4::Window::set_default_icon_name("chronos");

        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_string(
            "label.emoji-label { \
                font-family: 'Noto Color Emoji', 'Segoe UI Emoji', sans-serif; \
            } \
            button.insert-gap { \
                min-height: 6px; \
                padding: 0; \
                background-color: transparent; \
                transition: all 0.15s ease-out; \
            } \
            button.insert-gap:hover { \
                background-color: alpha(@theme_selected_bg_color, 0.3); \
                min-height: 12px; \
            } \
            .merged-cell { \
                border: 1px dashed alpha(@theme_fg_color, 0.3); \
                border-radius: 3px; \
                background-color: alpha(@theme_fg_color, 0.03); \
            } \
            button.split-gap { \
                min-height: 6px; \
                padding: 0; \
                background-color: transparent; \
                border: none; \
                transition: all 0.15s ease-out; \
            } \
            button.split-gap:hover { \
                background-color: alpha(@theme_selected_bg_color, 0.55); \
                min-height: 10px; \
            } \
            .menu-bar { \
                border-bottom: 1px solid alpha(@theme_fg_color, 0.12); \
                padding-bottom: 6px; \
            } \
            button.menu-item-btn { \
                padding: 6px 10px; \
                border-radius: 5px; \
            } \
            label.shortcut-hint { \
                color: alpha(@theme_fg_color, 0.45); \
                font-size: 0.88em; \
                margin-left: 24px; \
            } \
            .md-h1 { \
                font-size: 1.6em; \
                font-weight: bold; \
                margin-top: 8px; \
                margin-bottom: 4px; \
            } \
            .md-h2 { \
                font-size: 1.3em; \
                font-weight: bold; \
                margin-top: 10px; \
                margin-bottom: 2px; \
            } \
            .md-h3 { \
                font-size: 1.1em; \
                font-weight: bold; \
                margin-top: 6px; \
            } \
            .md-code { \
                font-family: monospace; \
                background-color: alpha(@theme_fg_color, 0.06); \
                padding: 8px; \
                border-radius: 4px; \
            }",
        );

        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    // GTK single-instance registration means a second launch with the same
    // APP_ID silently hands its windows over to the already-running primary
    // (an old build, an unreaped AppImage, another checkout) and exits 0.
    // Track activation so that handoff can at least be reported to whoever
    // is watching the terminal — otherwise an outdated instance keeps
    // running looks like a no-op launch.
    let activated = Rc::new(Cell::new(false));
    let app_activated = activated.clone();
    app.connect_activate(move |app| {
        app_activated.set(true);
        let (window, _ctx) = crate::ui::window::build_main_window(app);
        window.present();
    });

    let exit = app.run();

    if !activated.get() {
        eprintln!(
            "chronos: window forwarded to an already-running instance \
             with the same application id ({APP_ID}), which is probably \
             an older build; kill it to run this build in the foreground."
        );
    }

    exit
}
