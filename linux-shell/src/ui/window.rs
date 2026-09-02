use crate::context::{AppContext, BarTool, ColumnVisibility};
use chronos::bridge::{ConservationApp, Event};
use crux_core::Core;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub fn build_main_window(app: &gtk4::Application) -> (gtk4::ApplicationWindow, AppContext) {
    let core = Rc::new(Core::<ConservationApp>::default());
    let vis = Rc::new(RefCell::new(ColumnVisibility::default()));
    let tool = Rc::new(Cell::new(BarTool::Edit));
    let t = &core.view().translations;

    let export_btn = gtk4::Button::builder().label(&t.export_pdf).build();
    let settings_btn = gtk4::Button::builder().label(&t.settings_tooltip).build();
    let lang_btn = gtk4::Button::builder().label(&t.flag_emoji).build();

    let edit_btn = gtk4::ToggleButton::builder()
        .label(&t.tool_edit)
        .active(true)
        .build();
    let create_btn = gtk4::ToggleButton::builder().label(&t.tool_create).build();
    let delete_btn = gtk4::ToggleButton::new();

    let delete_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();
    let trash_icon = gtk4::Label::builder().label("🗑").margin_top(5).build();
    delete_box.append(&trash_icon);
    let delete_text_label = gtk4::Label::new(Some(&t.tool_delete));
    delete_box.append(&delete_text_label);
    delete_btn.set_child(Some(&delete_box));

    edit_btn.set_group(Option::<&gtk4::ToggleButton>::None);
    create_btn.set_group(Some(&edit_btn));
    delete_btn.set_group(Some(&edit_btn));

    let tool_edit = tool.clone();
    edit_btn.connect_toggled(move |b| {
        if b.is_active() {
            tool_edit.set(BarTool::Edit);
        }
    });
    let tool_edit = tool.clone();
    create_btn.connect_toggled(move |b| {
        if b.is_active() {
            tool_edit.set(BarTool::Create);
        }
    });
    let tool_edit = tool.clone();
    delete_btn.connect_toggled(move |b| {
        if b.is_active() {
            tool_edit.set(BarTool::Delete);
        }
    });

    let tool_bar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .build();
    tool_bar.append(&edit_btn);
    tool_bar.append(&create_btn);
    tool_bar.append(&delete_btn);

    let left_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .build();
    left_box.append(&lang_btn);
    left_box.append(&settings_btn);
    left_box.append(&export_btn);

    let top_bar = gtk4::CenterBox::new();
    top_bar.set_start_widget(Some(&left_box));
    top_bar.set_center_widget(Some(&tool_bar));

    let status_label = gtk4::Label::new(Some("Ready"));
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .spacing(12)
        .build();
    container.append(&top_bar);
    container.append(&status_label);

    let scrolled_window = gtk4::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

    let ctx = AppContext {
        core: core.clone(),
        vis: vis.clone(),
        tool: tool.clone(),
        status: status_label.clone(),
        export_btn: export_btn.clone(),
        lang_btn: lang_btn.clone(),
        settings_btn: settings_btn.clone(),
        edit_btn: edit_btn.clone(),
        create_btn: create_btn.clone(),
        delete_text_label: delete_text_label.clone(),
        scrolled_window: scrolled_window.clone(),
    };

    let ctx_btn = ctx.clone();
    lang_btn.connect_clicked(move |_| {
        let ctx_idle = ctx_btn.clone();
        glib::idle_add_local_once(move || {
            let effects = ctx_idle.core.process_event(Event::ToggleLanguage);
            crate::effects::process_effects(&ctx_idle, effects);
        });
    });

    let initial_view = core.view();
    let initial_table = crate::ui::table::build_document_table(&initial_view, &ctx);
    scrolled_window.set_child(Some(&initial_table));
    container.append(&scrolled_window);

    let ctx_settings = ctx.clone();
    settings_btn.connect_clicked(move |btn| {
        let settings_popover = gtk4::Popover::new();
        let popover_vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(8)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(12)
            .margin_end(12)
            .build();

        let current_vis = ctx_settings.vis.borrow().clone();
        let view = ctx_settings.core.view();
        let t = &view.translations;

        let labels = [
            &t.col_no,
            &t.col_graphical,
            &t.col_woven,
            &t.col_char,
            &t.col_thickness,
            &t.col_chrono,
            &t.col_dating,
        ];

        for (i, label_text) in labels.iter().enumerate() {
            let cb = gtk4::CheckButton::with_label(label_text);
            cb.set_active(current_vis.flags[i]);
            let ctx_cb = ctx_settings.clone();
            cb.connect_toggled(move |check_btn| {
                ctx_cb.vis.borrow_mut().flags[i] = check_btn.is_active();
                let view = ctx_cb.core.view();
                let new_table = crate::ui::table::build_document_table(&view, &ctx_cb);
                ctx_cb.scrolled_window.set_child(Some(&new_table));
            });
            popover_vbox.append(&cb);
        }
        settings_popover.set_child(Some(&popover_vbox));
        settings_popover.set_parent(btn);
        settings_popover.popup();
    });

    let ctx_export = ctx.clone();
    export_btn.connect_clicked(move |_| {
        let effects = ctx_export.core.process_event(Event::RequestPdfExport);
        crate::effects::process_effects(&ctx_export, effects);
    });

    let shortcut_controller = gtk4::ShortcutController::new();
    shortcut_controller.set_scope(gtk4::ShortcutScope::Managed);

    fn add_shortcut(
        controller: &gtk4::ShortcutController,
        keys: &[gtk4::gdk::Key],
        mods: gtk4::gdk::ModifierType,
        ctx: &AppContext,
        event: chronos::bridge::Event,
    ) {
        for &key in keys {
            let ctx_clone = ctx.clone();
            let event_clone = event.clone();
            let trigger = gtk4::KeyvalTrigger::new(key, mods);
            let action = gtk4::CallbackAction::new(move |_, _| {
                let ctx_idle = ctx_clone.clone();
                let event_idle = event_clone.clone();
                glib::idle_add_local_once(move || {
                    let effects = ctx_idle.core.process_event(event_idle);
                    crate::effects::process_effects(&ctx_idle, effects);
                });
                glib::Propagation::Stop
            });
            controller.add_shortcut(gtk4::Shortcut::new(Some(trigger), Some(action)));
        }
    }

    // Ctrl+O (Open)
    add_shortcut(
        &shortcut_controller,
        &[gtk4::gdk::Key::o, gtk4::gdk::Key::O],
        gtk4::gdk::ModifierType::CONTROL_MASK,
        &ctx,
        Event::RequestOpen,
    );

    // Ctrl+S (Save)
    add_shortcut(
        &shortcut_controller,
        &[gtk4::gdk::Key::s, gtk4::gdk::Key::S],
        gtk4::gdk::ModifierType::CONTROL_MASK,
        &ctx,
        Event::RequestSave,
    );

    // Ctrl+Shift+S (Save As)
    add_shortcut(
        &shortcut_controller,
        &[gtk4::gdk::Key::s, gtk4::gdk::Key::S],
        gtk4::gdk::ModifierType::CONTROL_MASK | gtk4::gdk::ModifierType::SHIFT_MASK,
        &ctx,
        Event::RequestSaveAs,
    );

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Conservation Editor")
        .default_width(1100)
        .default_height(600)
        .child(&container)
        .build();
    window.add_controller(shortcut_controller);
    (window, ctx)
}
