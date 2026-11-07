use crate::context::{AppContext, MenuLabels};
use crate::settings::Settings;
use crate::shortcuts::{Action, ShortcutRegistry};
use crate::tool::Tool;
use chronos::bridge::{ConservationApp, Event};
use chronos::i18n::Language;
use crux_core::Core;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn build_menu_row(
    text: &str,
    shortcut: Option<(&ShortcutRegistry, Action)>,
    popover: &gtk4::Popover,
    on_click: impl Fn() + 'static,
) -> (gtk4::Button, gtk4::Label) {
    let row_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(16)
        .hexpand(true)
        .build();

    let title_label = gtk4::Label::builder()
        .label(text)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .xalign(0.0)
        .build();
    row_box.append(&title_label);

    if let Some((registry, action)) = shortcut {
        let sc_label = gtk4::Label::builder().halign(gtk4::Align::End).build();
        registry.bind_hint_label(action, &sc_label);
        sc_label.add_css_class("shortcut-hint");
        row_box.append(&sc_label);
    }

    let btn = gtk4::Button::builder()
        .child(&row_box)
        .has_frame(false)
        .hexpand(true)
        .build();
    btn.add_css_class("flat");
    btn.add_css_class("menu-item-btn");

    let pop = popover.clone();
    btn.connect_clicked(move |_| {
        pop.popdown();
        on_click();
    });

    (btn, title_label)
}

pub fn build_main_window(app: &gtk4::Application) -> (gtk4::ApplicationWindow, AppContext) {
    let core = Rc::new(Core::<ConservationApp>::default());
    let settings = Settings::load();
    let vis = Rc::new(RefCell::new(settings.columns.clone()));
    let canvas_params = Rc::new(RefCell::new(settings.canvas.clone()));
    let shortcuts = Rc::new(ShortcutRegistry::new(&settings.shortcuts));
    let tool = Rc::new(Cell::new(Tool::Edit));
    let split_overlays: crate::ui::table::SplitOverlays = Rc::new(RefCell::new(Vec::new()));
    let t = &core.view().translations;

    let ctx_holder: Rc<RefCell<Option<AppContext>>> = Rc::new(RefCell::new(None));

    // --- Menu Bar (File, Edit, About, Help) ---
    let menu_bar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .build();
    menu_bar.add_css_class("menu-bar");

    // File Menu
    let file_popover = gtk4::Popover::builder().has_arrow(false).build();
    let file_vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .build();

    let (open_item, file_open_lbl) = build_menu_row(
        &t.menu_open,
        Some((&*shortcuts, Action::Open)),
        &file_popover,
        {
            let holder = ctx_holder.clone();
            move || {
                if let Some(ctx) = holder.borrow().as_ref() {
                    let ctx_idle = ctx.clone();
                    glib::idle_add_local_once(move || {
                        let effects = ctx_idle.core.process_event(Event::RequestOpen);
                        crate::effects::process_effects(&ctx_idle, effects);
                    });
                }
            }
        },
    );

    let (save_item, file_save_lbl) = build_menu_row(
        &t.menu_save,
        Some((&*shortcuts, Action::Save)),
        &file_popover,
        {
            let holder = ctx_holder.clone();
            move || {
                if let Some(ctx) = holder.borrow().as_ref() {
                    let ctx_idle = ctx.clone();
                    glib::idle_add_local_once(move || {
                        let effects = ctx_idle.core.process_event(Event::RequestSave);
                        crate::effects::process_effects(&ctx_idle, effects);
                    });
                }
            }
        },
    );

    let (save_as_item, file_save_as_lbl) = build_menu_row(
        &t.menu_save_as,
        Some((&*shortcuts, Action::SaveAs)),
        &file_popover,
        {
            let holder = ctx_holder.clone();
            move || {
                if let Some(ctx) = holder.borrow().as_ref() {
                    let ctx_idle = ctx.clone();
                    glib::idle_add_local_once(move || {
                        let effects = ctx_idle.core.process_event(Event::RequestSaveAs);
                        crate::effects::process_effects(&ctx_idle, effects);
                    });
                }
            }
        },
    );

    let (export_item, file_export_lbl) = build_menu_row(
        &t.menu_export,
        Some((&*shortcuts, Action::Export)),
        &file_popover,
        {
            let holder = ctx_holder.clone();
            move || {
                if let Some(ctx) = holder.borrow().as_ref() {
                    let ctx_idle = ctx.clone();
                    let flags = ctx_idle.vis.borrow().flags;
                    let canvas_params = ctx_idle.canvas_params.borrow().clone();
                    glib::idle_add_local_once(move || {
                        let effects = ctx_idle
                            .core
                            .process_event(Event::RequestPdfExport(flags, canvas_params));
                        crate::effects::process_effects(&ctx_idle, effects);
                    });
                }
            }
        },
    );

    file_vbox.append(&open_item);
    file_vbox.append(&save_item);
    file_vbox.append(&save_as_item);
    file_vbox.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    file_vbox.append(&export_item);
    file_popover.set_child(Some(&file_vbox));

    let file_btn_label = gtk4::Label::new(Some(&t.menu_file));
    let file_menu_btn = gtk4::MenuButton::builder()
        .child(&file_btn_label)
        .popover(&file_popover)
        .has_frame(false)
        .build();
    file_menu_btn.add_css_class("flat");

    // Edit Menu
    let edit_popover = gtk4::Popover::builder().has_arrow(false).build();
    let edit_vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .build();

    let (canvas_item, edit_canvas_lbl) =
        build_menu_row(&t.menu_canvas_params, None, &edit_popover, {
            let holder = ctx_holder.clone();
            move || {
                if let Some(ctx) = holder.borrow().as_ref() {
                    crate::ui::dialogs::show_canvas_params_dialog(ctx);
                }
            }
        });

    let (prefs_item, edit_prefs_lbl) = build_menu_row(&t.menu_preferences, None, &edit_popover, {
        let holder = ctx_holder.clone();
        move || {
            if let Some(ctx) = holder.borrow().as_ref() {
                crate::ui::dialogs::show_preferences_dialog(ctx);
            }
        }
    });

    let (shortcuts_item, edit_shortcuts_lbl) =
        build_menu_row(&t.menu_shortcuts, None, &edit_popover, {
            let holder = ctx_holder.clone();
            move || {
                if let Some(ctx) = holder.borrow().as_ref() {
                    crate::ui::dialogs::show_shortcuts_dialog(ctx);
                }
            }
        });

    edit_vbox.append(&canvas_item);
    edit_vbox.append(&prefs_item);
    edit_vbox.append(&shortcuts_item);
    edit_popover.set_child(Some(&edit_vbox));

    let edit_btn_label = gtk4::Label::new(Some(&t.menu_edit));
    let edit_menu_btn = gtk4::MenuButton::builder()
        .child(&edit_btn_label)
        .popover(&edit_popover)
        .has_frame(false)
        .build();
    edit_menu_btn.add_css_class("flat");

    // About Menu Button
    let about_menu_btn = gtk4::Button::builder()
        .label(&t.menu_about)
        .has_frame(false)
        .build();
    about_menu_btn.add_css_class("flat");
    {
        let holder = ctx_holder.clone();
        about_menu_btn.connect_clicked(move |_| {
            if let Some(ctx) = holder.borrow().as_ref() {
                let title = ctx.core.view().translations.menu_about;
                crate::ui::markdown::show_markdown_window(&ctx.window, &title, "about");
            }
        });
    }

    // Help Menu Button
    let help_menu_btn = gtk4::Button::builder()
        .label(&t.menu_help)
        .has_frame(false)
        .build();
    help_menu_btn.add_css_class("flat");
    {
        let holder = ctx_holder.clone();
        help_menu_btn.connect_clicked(move |_| {
            if let Some(ctx) = holder.borrow().as_ref() {
                let title = ctx.core.view().translations.menu_help;
                crate::ui::markdown::show_markdown_window(&ctx.window, &title, "help");
            }
        });
    }

    menu_bar.append(&file_menu_btn);
    menu_bar.append(&edit_menu_btn);
    menu_bar.append(&about_menu_btn);
    menu_bar.append(&help_menu_btn);

    let menu_labels = Rc::new(MenuLabels {
        file_btn: file_btn_label,
        edit_btn: edit_btn_label,
        about_btn: about_menu_btn,
        help_btn: help_menu_btn,
        file_open: file_open_lbl,
        file_save: file_save_lbl,
        file_save_as: file_save_as_lbl,
        file_export: file_export_lbl,
        edit_canvas: edit_canvas_lbl,
        edit_prefs: edit_prefs_lbl,
        edit_shortcuts: edit_shortcuts_lbl,
    });

    // --- Existing Toolbar & Controls ---
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
    let mut tool_buttons = Vec::with_capacity(Tool::TOOLBAR_DEFAULTS.len());
    let mut first_btn: Option<gtk4::ToggleButton> = None;
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
        shortcuts.bind_tooltip(Action::for_tool(default_tool), &btn, t);
        tool_buttons.push((default_tool, btn.clone()));

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
                let overlays = split_overlays.clone();
                let remembered = remembered_variant.clone();
                let parent_btn = btn.clone();
                var_btn.connect_toggled(move |b| {
                    if b.is_active() {
                        remembered.set(variant);
                        if !parent_btn.is_active() {
                            parent_btn.set_active(true);
                        }
                        tool_cell.set(variant);
                        crate::ui::table::update_split_overlays(&overlays, variant);
                    }
                });

                selector_box.append(&var_btn);
            }

            if variants.len() <= 3 {
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
                bottom_stack.add_named(&selector_box, Some(&stack_page_name));
            }
        }

        let tool_cell = tool.clone();
        let overlays = split_overlays.clone();
        let remembered = remembered_variant.clone();
        let inline_revs = inline_revealers.clone();
        let bottom_rev = bottom_revealer.clone();
        let b_stack = bottom_stack.clone();
        let variant_count = variants.len();

        btn.connect_toggled(move |b| {
            if b.is_active() {
                let active_tool = remembered.get();
                tool_cell.set(active_tool);
                crate::ui::table::update_split_overlays(&overlays, active_tool);
                for (kind, rev) in inline_revs.borrow().iter() {
                    rev.set_reveal_child(kind.same_kind(active_tool));
                }
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
                if !new_name.trim().is_empty()
                    && let Some(ctx) = ctx_holder.borrow().as_ref()
                {
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
        .margin_top(10)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .spacing(12)
        .build();
    container.append(&menu_bar);
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
        canvas_params,
        tool: tool.clone(),
        status: status_label.clone(),
        export_btn: export_btn.clone(),
        settings_label: settings_label.clone(),
        current_lang,
        flag_area,
        tool_labels: Rc::new(tool_labels),
        tool_buttons: Rc::new(tool_buttons),
        shortcuts: shortcuts.clone(),
        menu_labels,
        scrolled_window: scrolled_window.clone(),
        table: Rc::new(RefCell::new(None)),
        split_overlays: split_overlays.clone(),
        last_message: Rc::new(RefCell::new(None)),
        status_timer: Rc::new(RefCell::new(None)),
        settings_timer: Rc::new(RefCell::new(None)),
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
    crate::ui::table::sync_document_table(&initial_view, &ctx);
    container.append(&scrolled_window);

    let settings_popover = gtk4::Popover::builder().has_arrow(false).build();
    settings_popover.set_parent(&settings_btn);

    let ctx_settings = ctx.clone();
    settings_btn.connect_clicked(move |_btn| {
        let settings_popover = settings_popover.clone();
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
                crate::ui::table::sync_document_table(&view, &ctx_cb);
                ctx_cb.persist_settings();
            });
            popover_vbox.append(&cb);
        }

        settings_popover.set_child(Some(&popover_vbox));
        settings_popover.popup();
    });

    let ctx_export = ctx.clone();
    export_btn.connect_clicked(move |_| {
        let current_vis = ctx_export.vis.borrow().flags;
        let canvas_params = ctx_export.canvas_params.borrow().clone();
        let effects = ctx_export
            .core
            .process_event(Event::RequestPdfExport(current_vis, canvas_params));
        crate::effects::process_effects(&ctx_export, effects);
    });

    {
        let ctx_sc = ctx.clone();
        shortcuts.install(&window, Rc::new(move |action| run_action(&ctx_sc, action)));
    }

    if let Some(saved) = settings.saved_language()
        && saved != ctx.current_lang.get()
    {
        defer_event(&ctx, Event::ToggleLanguage);
    }

    let ctx_close = ctx.clone();
    window.connect_close_request(move |_| {
        ctx_close.flush_pending_settings();
        glib::Propagation::Proceed
    });
    (window, ctx)
}

fn defer_event(ctx: &AppContext, event: Event) {
    let ctx = ctx.clone();
    glib::idle_add_local_once(move || {
        let effects = ctx.core.process_event(event);
        crate::effects::process_effects(&ctx, effects);
    });
}

fn select_tool(ctx: &AppContext, tool: Tool) {
    if let Some((_, btn)) = ctx.tool_buttons.iter().find(|(t, _)| t.same_kind(tool)) {
        btn.set_active(true);
    }
}

fn run_action(ctx: &AppContext, action: Action) {
    match action {
        Action::Open => defer_event(ctx, Event::RequestOpen),
        Action::Save => defer_event(ctx, Event::RequestSave),
        Action::SaveAs => defer_event(ctx, Event::RequestSaveAs),
        Action::Export => {
            let flags = ctx.vis.borrow().flags;
            let canvas_params = ctx.canvas_params.borrow().clone();
            defer_event(ctx, Event::RequestPdfExport(flags, canvas_params));
        }
        tool_action => {
            if let Some(tool) = tool_action.tool() {
                select_tool(ctx, tool);
            }
        }
    }
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
