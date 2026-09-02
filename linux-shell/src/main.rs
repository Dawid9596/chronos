mod context;
mod effects;
mod ui;

use gtk4::prelude::*;

const APP_ID: &str = "org.conservation.Editor";

fn main() {
    gtk4::init().expect("Failed to initialize GTK4");

    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        "button.insert-gap { \
            min-height: 6px; \
            padding: 0; \
            background-color: transparent; \
            transition: all 0.15s ease-out; \
        } \
        button.insert-gap:hover { \
            background-color: alpha(@theme_selected_bg_color, 0.3); \
            min-height: 12px; \
        }",
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let app = gtk4::Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        let (window, _ctx) = crate::ui::window::build_main_window(app);
        window.present();
    });

    app.run();
}
