#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod context;
mod effects;
mod ui;

use gtk4::prelude::*;

const APP_ID: &str = "org.conservation.Editor";

fn main() -> gtk4::glib::ExitCode {
    let app = gtk4::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        let display = gtk4::gdk::Display::default().expect("Could not connect to a display.");

        let icon_theme = gtk4::IconTheme::for_display(&display);
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                icon_theme.add_search_path(exe_dir.join("share/icons"));
            }
        }
        let settings = gtk4::Settings::for_display(&display);
        settings.set_gtk_icon_theme_name(Some("Adwaita"));
        gtk4::Window::set_default_icon_name("chronos");

        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_string(
            "label.emoji-label { \
                font-family: 'Segoe UI Emoji', 'Noto Color Emoji', sans-serif; \
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
            }",
        );

        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(|app| {
        let (window, _ctx) = crate::ui::window::build_main_window(app);
        window.present();
    });

    app.run()
}
