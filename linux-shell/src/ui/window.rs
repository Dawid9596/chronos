use crate::context::{AppContext, ColumnVisibility};
use crate::tool::Tool;
use chronos::bridge::{ConservationApp, Event};
use chronos::i18n::Language;
use crux_core::Core;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub fn build_main_window(app: &gtk4::Application) -> (gtk4::ApplicationWindow, AppContext) {
    let core = Rc::new(Core::<ConservationApp>::default());
    let vis = Rc::new(RefCell::new(ColumnVisibility::default()));
    let tool = Rc::new(Cell::new(Tool::Edit));

    let t = &core.view().translations;

    let export_btn = gtk4::Button::builder()
        .icon_name("media-floppy-symbolic")
        .tooltip_text(&t.export_pdf)
        .build();

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

    let current_lang = Rc::new(Cell::new(t.language));
    let flag_area = build_flag_widget(current_lang.clone());

    let lang_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    let lang_icon = gtk4::Image::from_icon_name("preferences-desktop-locale-symbolic");
    lang_box.append(&lang_icon);
    lang_box.append(&flag_area);
    let lang_btn = gtk4::Button::builder().child(&lang_box).build();

    let tool_section = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .halign(gtk4::Align::Center)
        .build();

    let tool_bar_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .halign(gtk4::Align::Center)
        .build();

    let bottom_stack = gtk4::Stack::builder()
        .transition_type(gtk4::StackTransitionType::Crossfade)
        .halign(gtk4::Align::Center)
        .build();

    let bottom_revealer = gtk4::Revealer::builder()
        .transition_type(gtk4::RevealerTransitionType::SlideDown)
        .reveal_child(false)
        .halign(gtk4::Align::Center)
        .child(&bottom_stack)
        .build();

    let mut tool_labels = Vec::with_capacity(Tool::TOOLBAR_DEFAULTS.len());
    let mut first_btn: Option<gtk4::ToggleButton> = None;

    // Keep track of inline revealers so they close when another tool is chosen
    let inline_revealers: Rc<RefCell<Vec<(Tool, gtk4::Revealer)>>> =
        Rc::new(RefCell::new(Vec::new()));

    for (tool_idx, default_tool) in Tool::TOOLBAR_DEFAULTS.into_iter().enumerate() {
        let remembered_variant = Rc::new(Cell::new(default_tool));
        let variants = default_tool.variants();

        let btn = gtk4::ToggleButton::builder()
            .active(default_tool == Tool::Edit)
            .build();

        let btn_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        let icon = gtk4::Image::from_icon_name(default_tool.icon_name());
        let text_label = gtk4::Label::new(Some(default_tool.label(t)));
        btn_box.append(&icon);
        btn_box.append(&text_label);
        btn.set_child(Some(&btn_box));

        if let Some(ref group_leader) = first_btn {
            btn.set_group(Some(group_leader));
        } else {
            btn.set_group(Option::<&gtk4::ToggleButton>::None);
            first_btn = Some(btn.clone());
        }

        tool_bar_row.append(&btn);
        tool_labels.push((default_tool, text_label));

        // Build variant selector box if this tool has variants
        let stack_page_name = format!("tool-variants-{tool_idx}");
        if !variants.is_empty() {
            let selector_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(2)
                .build();

            let mut first_var_btn: Option<gtk4::ToggleButton> = None;
            for &variant in variants {
                let var_widget = variant.build_variant_widget();
                let var_btn = gtk4::ToggleButton::builder()
                    .child(&var_widget)
                    .active(variant == default_tool)
                    .build();

                if let Some(ref leader) = first_var_btn {
                    var_btn.set_group(Some(leader));
                } else {
                    first_var_btn = Some(var_btn.clone());
                }

                let tool_cell = tool.clone();
                let remembered = remembered_variant.clone();
                let parent_btn = btn.clone();
                var_btn.connect_toggled(move |b| {
                    if b.is_active() {
                        remembered.set(variant);
                        if !parent_btn.is_active() {
                            parent_btn.set_active(true);
                        }
                        tool_cell.set(variant);
                    }
                });

                selector_box.append(&var_btn);
            }

            if variants.len() <= 3 {
                // Case 1: <= 3 variants -> inline immediately to the right of this tool button
                selector_box.set_margin_start(2);
                selector_box.set_margin_end(4);
                let inline_rev = gtk4::Revealer::builder()
                    .transition_type(gtk4::RevealerTransitionType::SlideRight)
                    .reveal_child(default_tool == Tool::Edit)
                    .child(&selector_box)
                    .build();
                tool_bar_row.append(&inline_rev);
                inline_revealers
                    .borrow_mut()
                    .push((default_tool, inline_rev));
            } else {
                // Case 2: > 3 variants -> placed in the bottom revealer stack
                bottom_stack.add_named(&selector_box, Some(&stack_page_name));
            }
        }

        let tool_cell = tool.clone();
        let remembered = remembered_variant.clone();
        let inline_revs = inline_revealers.clone();
        let bottom_rev = bottom_revealer.clone();
        let b_stack = bottom_stack.clone();
        let variant_count = variants.len();

        btn.connect_toggled(move |b| {
            if b.is_active() {
                let active_tool = remembered.get();
                tool_cell.set(active_tool);

                // Update inline revealers (<= 3 variants)
                for (kind, rev) in inline_revs.borrow().iter() {
                    rev.set_reveal_child(kind.same_kind(active_tool));
                }

                // Update bottom revealer (> 3 variants)
                if variant_count > 3 {
                    b_stack.set_visible_child_name(&stack_page_name);
                    bottom_rev.set_reveal_child(true);
                } else {
                    bottom_rev.set_reveal_child(false);
                }
            }
        });
    }

    tool_section.append(&tool_bar_row);
    tool_section.append(&bottom_revealer);

    let left_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .build();
    left_box.append(&lang_btn);
    left_box.append(&settings_btn);
    left_box.append(&export_btn);

    let top_bar = gtk4::CenterBox::new();
    top_bar.set_start_widget(Some(&left_box));
    top_bar.set_center_widget(Some(&tool_section));

    let status_stack = gtk4::Stack::new();
    status_stack.set_transition_type(gtk4::StackTransitionType::None);
    status_stack.set_halign(gtk4::Align::Center);

    let status_label = gtk4::Label::builder()
        .label(format!(
            "{}{}",
            t.project_prefix,
            core.view().document.project_name
        ))
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

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Conservation Editor")
        .icon_name("chronos")
        .default_width(1100)
        .default_height(600)
        .child(&container)
        .build();

    let ctx = AppContext {
        window: window.clone(),
        core: core.clone(),
        vis: vis.clone(),
        tool: tool.clone(),
        status: status_label.clone(),
        export_btn: export_btn.clone(),
        settings_label: settings_label.clone(),
        current_lang,
        flag_area,
        tool_labels: Rc::new(tool_labels),
        scrolled_window: scrolled_window.clone(),
        last_message: Rc::new(RefCell::new(None)),
        status_timer: Rc::new(RefCell::new(None)),
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

    window.add_controller(shortcut_controller);
    (window, ctx)
}

pub fn build_flag_widget(lang: Rc<Cell<Language>>) -> gtk4::DrawingArea {
    let area = gtk4::DrawingArea::builder()
        .width_request(22)
        .height_request(15)
        .valign(gtk4::Align::Center)
        .build();

    area.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        match lang.get() {
            Language::Polish => {
                // Top half: White
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.rectangle(0.0, 0.0, w, h / 2.0);
                let _ = cr.fill();

                // Bottom half: Crimson (#DC143C)
                cr.set_source_rgb(0.863, 0.078, 0.235);
                cr.rectangle(0.0, h / 2.0, w, h / 2.0);
                let _ = cr.fill();
            }
            Language::English => {
                // Clip all diagonals and crosses to the flag bounds
                cr.rectangle(0.0, 0.0, w, h);
                cr.clip();

                // 1. Navy blue background (#012169)
                cr.set_source_rgb(0.004, 0.129, 0.412);
                cr.rectangle(0.0, 0.0, w, h);
                let _ = cr.fill();

                // 2. White diagonal cross
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_line_width(2.8);
                cr.move_to(0.0, 0.0);
                cr.line_to(w, h);
                cr.move_to(w, 0.0);
                cr.line_to(0.0, h);
                let _ = cr.stroke();

                // 3. Red diagonal cross (#C8102E)
                cr.set_source_rgb(0.784, 0.063, 0.180);
                cr.set_line_width(1.3);
                cr.move_to(0.0, 0.0);
                cr.line_to(w, h);
                cr.move_to(w, 0.0);
                cr.line_to(0.0, h);
                let _ = cr.stroke();

                // 4. White central cross
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_line_width(4.6);
                cr.move_to(w / 2.0, 0.0);
                cr.line_to(w / 2.0, h);
                cr.move_to(0.0, h / 2.0);
                cr.line_to(w, h / 2.0);
                let _ = cr.stroke();

                // 5. Red central cross (#C8102E)
                cr.set_source_rgb(0.784, 0.063, 0.180);
                cr.set_line_width(2.6);
                cr.move_to(w / 2.0, 0.0);
                cr.line_to(w / 2.0, h);
                cr.move_to(0.0, h / 2.0);
                cr.line_to(w, h / 2.0);
                let _ = cr.stroke();

                cr.reset_clip();
            }
        }

        // Subtle 1px outer frame so the white edges stay defined against light/dark themes
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
        cr.set_line_width(1.0);
        cr.rectangle(0.5, 0.5, w - 1.0, h - 1.0);
        let _ = cr.stroke();
    });

    area
}
