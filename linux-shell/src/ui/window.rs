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
    export_btn.set_icon_name("document-save-symbolic");

    let settings_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .build();
    let settings_icon = gtk4::Image::from_icon_name("emblem-system-symbolic");
    let settings_label = gtk4::Label::builder()
        .label(&t.settings_tooltip)
        .use_markup(true)
        .visible(false)
        .build();
    settings_box.append(&settings_icon);
    settings_box.append(&settings_label);
    let settings_btn = gtk4::Button::builder()
        .child(&settings_box)
        .tooltip_text("Columns")
        .build();

    let lang_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    let lang_icon = gtk4::Image::from_icon_name("preferences-desktop-locale-symbolic");
    let flag_label = gtk4::Label::builder()
        .label(&t.flag_emoji)
        .use_markup(true)
        .build();
    lang_box.append(&lang_icon);
    lang_box.append(&flag_label);
    let lang_btn = gtk4::Button::builder().child(&lang_box).build();

    let edit_btn = gtk4::ToggleButton::builder()
        .label(&t.tool_edit)
        .icon_name("document-edit-symbolic")
        .active(true)
        .build();

    let create_btn = gtk4::ToggleButton::builder()
        .label(&t.tool_create)
        .icon_name("list-add-symbolic")
        .build();

    let delete_btn = gtk4::ToggleButton::new();
    let delete_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();
    let trash_icon = gtk4::Image::from_icon_name("user-trash-symbolic");
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

    let status_stack = gtk4::Stack::new();
    status_stack.set_transition_type(gtk4::StackTransitionType::None);
    status_stack.set_halign(gtk4::Align::Center);

    let status_label = gtk4::Label::builder()
        .label("Ready")
        .halign(gtk4::Align::Center)
        .tooltip_text("Double-click project name to edit")
        .build();

    let pointer_cursor = gtk4::gdk::Cursor::from_name("pointer", None);
    status_label.set_cursor(pointer_cursor.as_ref());

    let status_entry = gtk4::Entry::builder()
        .halign(gtk4::Align::Center)
        .width_chars(25)
        .build();

    status_stack.add_named(&status_label, Some("label"));
    status_stack.add_named(&status_entry, Some("entry"));
    status_stack.set_visible_child_name("label");

    let click_gesture = gtk4::GestureClick::new();
    click_gesture.set_button(1);
    let status_stack_click = status_stack.clone();
    let status_entry_click = status_entry.clone();
    let core_click = core.clone();
    click_gesture.connect_pressed(move |_gesture, n_press, _x, _y| {
        if n_press == 2 {
            let current_project = core_click.view().document.project_name;
            status_entry_click.set_text(&current_project);
            status_stack_click.set_visible_child_name("entry");
            status_entry_click.grab_focus();
            status_entry_click.select_region(0, -1);
        }
    });
    status_label.add_controller(click_gesture);

    let ctx_holder: Rc<RefCell<Option<AppContext>>> = Rc::new(RefCell::new(None));

    let commit_project_name = {
        let status_stack = status_stack.clone();
        let status_entry = status_entry.clone();
        let ctx_holder = ctx_holder.clone();
        move || {
            if status_stack.visible_child_name().as_deref() == Some("entry") {
                if let Some(root) = status_entry.root() {
                    root.set_focus(None::<&gtk4::Widget>);
                }
                let new_name = status_entry.text().to_string();
                status_stack.set_visible_child_name("label");
                if !new_name.trim().is_empty() {
                    if let Some(ctx) = ctx_holder.borrow().as_ref() {
                        let ctx_idle = ctx.clone();
                        glib::idle_add_local_once(move || {
                            let effects = ctx_idle
                                .core
                                .process_event(Event::UpdateProjectName(new_name));
                            crate::effects::process_effects(&ctx_idle, effects);
                        });
                    }
                }
            }
        }
    };

    let commit_activate = commit_project_name.clone();
    status_entry.connect_activate(move |_| {
        commit_activate();
    });

    let key_controller = gtk4::EventControllerKey::new();
    let status_stack_key = status_stack.clone();
    key_controller.connect_key_pressed(move |_controller, key, _keycode, _state| {
        if key == gtk4::gdk::Key::Escape {
            status_stack_key.set_visible_child_name("label");
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    status_entry.add_controller(key_controller);

    let focus_controller = gtk4::EventControllerFocus::new();
    let commit_focus = commit_project_name;
    focus_controller.connect_leave(move |_controller| {
        commit_focus();
    });
    status_entry.add_controller(focus_controller);

    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .spacing(12)
        .build();

    container.append(&top_bar);
    container.append(&status_stack);

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
        lang_label: flag_label.clone(),
        settings_label: settings_label.clone(),
        edit_btn: edit_btn.clone(),
        create_btn: create_btn.clone(),
        delete_text_label: delete_text_label.clone(),
        scrolled_window: scrolled_window.clone(),
    };

    *ctx_holder.borrow_mut() = Some(ctx.clone());

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
        let current_vis = ctx_export.vis.borrow().flags;
        let effects = ctx_export
            .core
            .process_event(Event::RequestPdfExport(current_vis));
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
