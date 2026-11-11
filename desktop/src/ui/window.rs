use crate::context::{AppContext, MenuLabels};
use crate::settings::Settings;
use crate::shortcuts::{Action, ShortcutRegistry};
use crate::tool::Tool;
use crate::ui::widgets::history_buttons::HistoryButtons;
use chronos::bridge::{ConservationApp, Event};
use crux_core::Core;
use gtk4::prelude::*;
use gtk4::{Align, glib};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn editable_label_stack(
    initial_text: &str,
    tooltip: &str,
    width_chars: i32,
    current: impl Fn() -> String + 'static,
    commit: impl Fn(String) + 'static,
) -> (gtk4::Stack, gtk4::Label) {
    let label = gtk4::Label::builder()
        .label(initial_text)
        .halign(Align::Center)
        .tooltip_text(tooltip)
        .build();
    let pointer_cursor = gtk4::gdk::Cursor::from_name("pointer", None);
    label.set_cursor(pointer_cursor.as_ref());

    let entry = gtk4::Entry::builder()
        .halign(Align::Center)
        .width_chars(width_chars)
        .build();

    let stack = gtk4::Stack::builder()
        .transition_type(gtk4::StackTransitionType::None)
        .halign(Align::Center)
        .build();
    stack.add_named(&label, Some("label"));
    stack.add_named(&entry, Some("entry"));
    stack.set_visible_child_name("label");

    let click = gtk4::GestureClick::new();
    click.set_button(1);
    {
        let stack = stack.clone();
        let entry = entry.clone();
        click.connect_pressed(move |_gesture, n_press, _x, _y| {
            if n_press == 2 {
                entry.set_text(&current());
                stack.set_visible_child_name("entry");
                entry.grab_focus();
                entry.select_region(0, -1);
            }
        });
    }
    label.add_controller(click);

    let commit = Rc::new(commit);
    let commit_row = {
        let stack = stack.clone();
        let entry = entry.clone();
        let commit = commit.clone();
        move || {
            if stack.visible_child_name().as_deref() == Some("entry") {
                if let Some(root) = entry.root() {
                    root.set_focus(None::<&gtk4::Widget>);
                }
                let new_text = entry.text().to_string();
                stack.set_visible_child_name("label");
                commit(new_text);
            }
        }
    };

    {
        let commit_row = commit_row.clone();
        entry.connect_activate(move |_| commit_row());
    }

    let key_controller = gtk4::EventControllerKey::new();
    {
        let stack = stack.clone();
        key_controller.connect_key_pressed(move |_controller, key, _keycode, _state| {
            if key == gtk4::gdk::Key::Escape {
                stack.set_visible_child_name("label");
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
    }
    entry.add_controller(key_controller);

    let focus_controller = gtk4::EventControllerFocus::new();
    focus_controller.connect_leave(move |_| commit_row());
    entry.add_controller(focus_controller);

    (stack, label)
}

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

    // Lets widgets built before `AppContext` exists (menu rows, history buttons)
    // send core events the same way the rest of the window does.
    let send_event: Rc<dyn Fn(Event)> = {
        let holder = ctx_holder.clone();
        Rc::new(move |event| {
            if let Some(ctx) = holder.borrow().as_ref() {
                defer_event(ctx, event);
            }
        })
    };

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
                        let event = Event::RequestSave(ctx_idle.export_settings());
                        let effects = ctx_idle.core.process_event(event);
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
                        let event = Event::RequestSaveAs(ctx_idle.export_settings());
                        let effects = ctx_idle.core.process_event(event);
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
                    let event = ctx_idle.pdf_export_event();
                    glib::idle_add_local_once(move || {
                        let effects = ctx_idle.core.process_event(event);
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

    let (undo_item, edit_undo_lbl) = build_menu_row(
        &t.menu_undo,
        Some((&*shortcuts, Action::Undo)),
        &edit_popover,
        {
            let send = send_event.clone();
            move || send(Event::Undo)
        },
    );
    let (redo_item, edit_redo_lbl) = build_menu_row(
        &t.menu_redo,
        Some((&*shortcuts, Action::Redo)),
        &edit_popover,
        {
            let send = send_event.clone();
            move || send(Event::Redo)
        },
    );

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

    edit_vbox.append(&undo_item);
    edit_vbox.append(&redo_item);
    edit_vbox.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
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

    // Help Menu (Help, About, Check for Updates)
    let help_popover = gtk4::Popover::builder().has_arrow(false).build();
    let help_vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .build();

    let (help_help_item, help_help_lbl) = build_menu_row(&t.menu_help, None, &help_popover, {
        let holder = ctx_holder.clone();
        move || {
            if let Some(ctx) = holder.borrow().as_ref() {
                let view = ctx.core.view();
                crate::ui::markdown::show_markdown_window(&ctx.window, &view.translations, "help");
            }
        }
    });

    let (help_about_item, help_about_lbl) = build_menu_row(&t.menu_about, None, &help_popover, {
        let holder = ctx_holder.clone();
        move || {
            if let Some(ctx) = holder.borrow().as_ref() {
                let view = ctx.core.view();
                crate::ui::markdown::show_markdown_window(&ctx.window, &view.translations, "about");
            }
        }
    });

    let (help_check_item, help_check_lbl) =
        build_menu_row(&t.menu_check_updates, None, &help_popover, {
            let holder = ctx_holder.clone();
            move || {
                if let Some(ctx) = holder.borrow().as_ref() {
                    crate::update::check_now(ctx);
                }
            }
        });

    help_vbox.append(&help_help_item);
    help_vbox.append(&help_about_item);
    help_vbox.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    help_vbox.append(&help_check_item);
    help_popover.set_child(Some(&help_vbox));

    let help_btn_label = gtk4::Label::new(Some(&t.menu_help));
    let help_menu_btn = gtk4::MenuButton::builder()
        .child(&help_btn_label)
        .popover(&help_popover)
        .has_frame(false)
        .build();
    help_menu_btn.add_css_class("flat");

    menu_bar.append(&file_menu_btn);
    menu_bar.append(&edit_menu_btn);
    menu_bar.append(&help_menu_btn);

    let menu_labels = Rc::new(MenuLabels {
        file_btn: file_btn_label,
        edit_btn: edit_btn_label,
        help_btn: help_btn_label,
        help_help: help_help_lbl,
        help_about: help_about_lbl,
        help_check: help_check_lbl,
        file_open: file_open_lbl,
        file_save: file_save_lbl,
        file_save_as: file_save_as_lbl,
        file_export: file_export_lbl,
        edit_undo: edit_undo_lbl,
        edit_redo: edit_redo_lbl,
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
        .hhomogeneous(false)
        .vhomogeneous(false)
        .hexpand(true)
        .build();

    let bottom_revealer = gtk4::Revealer::builder()
        .transition_type(gtk4::RevealerTransitionType::SlideDown)
        .reveal_child(false)
        .hexpand(true)
        .child(&bottom_stack)
        .build();

    let mut tool_labels = Vec::with_capacity(Tool::TOOLBAR_DEFAULTS.len());
    let mut tool_buttons = Vec::with_capacity(Tool::TOOLBAR_DEFAULTS.len());
    let variant_buttons: Rc<RefCell<Vec<(Tool, gtk4::ToggleButton)>>> =
        Rc::new(RefCell::new(Vec::new()));
    let mut first_btn: Option<gtk4::ToggleButton> = None;
    let inline_revealers: Rc<RefCell<Vec<(Tool, gtk4::Revealer)>>> =
        Rc::new(RefCell::new(Vec::new()));
    let top_revealers: Rc<RefCell<Vec<(Tool, gtk4::Revealer)>>> = Rc::new(RefCell::new(Vec::new()));

    for (tool_idx, default_tool) in Tool::TOOLBAR_DEFAULTS.into_iter().enumerate() {
        let remembered_variant = Rc::new(Cell::new(default_tool));
        let variants = default_tool.variants();

        let btn = gtk4::ToggleButton::builder()
            .active(default_tool == Tool::Edit)
            .focus_on_click(false)
            .can_focus(false)
            .valign(gtk4::Align::End)
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

        if matches!(default_tool, Tool::Paint(_)) {
            let paint_col = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(0)
                .valign(gtk4::Align::End)
                .build();

            let toolbox_btn = gtk4::Button::builder()
                .icon_name("system-search-symbolic")
                .focus_on_click(false)
                .can_focus(false)
                .halign(gtk4::Align::Fill)
                .margin_bottom(2)
                .build();

            let holder = ctx_holder.clone();
            toolbox_btn.connect_clicked(move |_| {
                if let Some(ctx) = holder.borrow().as_ref() {
                    crate::ui::dialogs::show_paint_toolbox_dialog(ctx);
                }
            });

            let top_rev = gtk4::Revealer::builder()
                .transition_type(gtk4::RevealerTransitionType::SlideDown)
                .reveal_child(false)
                .child(&toolbox_btn)
                .build();

            paint_col.append(&top_rev);
            paint_col.append(&btn);
            tool_bar_row.append(&paint_col);
            top_revealers.borrow_mut().push((default_tool, top_rev));
        } else {
            tool_bar_row.append(&btn);
        }

        tool_labels.push((default_tool, text_label));
        shortcuts.bind_named_tooltip(Action::for_tool(default_tool), &btn, t);
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
                    .focus_on_click(false)
                    .can_focus(false)
                    .build();

                if let Tool::Paint(pattern) = variant {
                    var_btn.set_tooltip_text(Some(pattern.display_name(t.language)));
                }

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

                variant_buttons
                    .borrow_mut()
                    .push((variant, var_btn.clone()));
                selector_box.append(&var_btn);
            }

            if variants.len() <= 3 {
                selector_box.set_margin_start(2);
                selector_box.set_margin_end(4);
                let inline_rev = gtk4::Revealer::builder()
                    .transition_type(gtk4::RevealerTransitionType::SlideRight)
                    .reveal_child(default_tool == Tool::Edit)
                    .valign(gtk4::Align::End)
                    .child(&selector_box)
                    .build();
                tool_bar_row.append(&inline_rev);
                inline_revealers
                    .borrow_mut()
                    .push((default_tool, inline_rev));
            } else {
                selector_box.set_halign(gtk4::Align::Center);
                selector_box.set_valign(gtk4::Align::Center); // Prevent vertical stretching
                // Reserve 18px of empty space below the buttons.
                // The overlay scrollbar will float over this gap instead of the buttons!
                selector_box.set_margin_bottom(18);

                let viewport = gtk4::Viewport::builder()
                    .scroll_to_focus(false)
                    .child(&selector_box)
                    .build();

                let selector_view = gtk4::ScrolledWindow::builder()
                    .hscrollbar_policy(gtk4::PolicyType::Automatic)
                    .vscrollbar_policy(gtk4::PolicyType::Never)
                    // Must be TRUE so GTK doesn't steal layout height from the Viewport
                    .overlay_scrolling(true)
                    .propagate_natural_height(true)
                    .min_content_width(1)
                    .propagate_natural_width(false)
                    .hexpand(true)
                    .child(&viewport)
                    .build();

                let strip_scroll = gtk4::EventControllerScroll::new(
                    gtk4::EventControllerScrollFlags::VERTICAL
                        | gtk4::EventControllerScrollFlags::HORIZONTAL,
                );

                let hadj = selector_view.hadjustment();
                strip_scroll.connect_scroll(move |ctrl, dx, dy| {
                    if ctrl
                        .current_event_state()
                        .intersects(gtk4::gdk::ModifierType::CONTROL_MASK)
                    {
                        return glib::Propagation::Proceed;
                    }

                    // Let ScrolledWindow handle native horizontal swipes / tilt wheels
                    if dx.abs() > 0.0 {
                        return glib::Propagation::Proceed;
                    }

                    // Differentiate smooth touch movements from physical mouse wheel notches
                    let delta = match ctrl.unit() {
                        // Touchpad / smooth scrolling: deltas are already pixel distances (1:1)
                        gtk4::gdk::ScrollUnit::Surface => dy,
                        // Discrete mouse wheel click: advance ~1 button width (36px) per notch
                        gtk4::gdk::ScrollUnit::Wheel => dy * 36.0,
                        _ => dy * 36.0,
                    };

                    let max_val = (hadj.upper() - hadj.page_size()).max(0.0);
                    hadj.set_value((hadj.value() + delta).clamp(0.0, max_val));
                    glib::Propagation::Stop
                });
                selector_view.add_controller(strip_scroll);

                bottom_stack.add_named(&selector_view, Some(&stack_page_name));
            }
        }

        let tool_cell = tool.clone();
        let overlays = split_overlays.clone();
        let remembered = remembered_variant.clone();
        let inline_revs = inline_revealers.clone();
        let top_revs = top_revealers.clone();
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
                for (kind, rev) in top_revs.borrow().iter() {
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

    let left_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .build();
    left_box.set_halign(Align::End);
    left_box.set_valign(Align::Start);
    left_box.append(&settings_btn);
    left_box.append(&export_btn);

    let top_bar = gtk4::CenterBox::new();
    top_bar.set_start_widget(Some(&left_box));
    top_bar.set_center_widget(Some(&tool_section));

    let history_buttons = Rc::new(HistoryButtons::new(
        &shortcuts,
        t,
        send_event.clone(),
        undo_item.clone(),
        redo_item.clone(),
    ));
    top_bar.set_end_widget(Some(history_buttons.widget()));

    let current_project_name = core.view().document.project_name.clone();
    let (status_stack, status_label) = editable_label_stack(
        &format!(
            "{}{}",
            t.project_prefix,
            if current_project_name.is_empty() {
                &t.project_untitled
            } else {
                &current_project_name
            }
        ),
        "Double-click project name to edit",
        25,
        {
            let core = core.clone();
            move || core.view().document.project_name
        },
        {
            let ctx_holder = ctx_holder.clone();
            move |new_name: String| {
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
        },
    );

    let current_subtitle = core.view().document.subtitle.clone();
    let (subtitle_stack, subtitle_label) = editable_label_stack(
        &if current_subtitle.is_empty() {
            t.subtitle_placeholder.clone()
        } else {
            current_subtitle
        },
        "Double-click to edit the subtitle shown in the exported PDF",
        40,
        {
            let core = core.clone();
            move || core.view().document.subtitle
        },
        {
            let ctx_holder = ctx_holder.clone();
            move |new_subtitle: String| {
                if let Some(ctx) = ctx_holder.borrow().as_ref() {
                    let ctx_idle = ctx.clone();
                    glib::idle_add_local_once(move || {
                        let effects = ctx_idle
                            .core
                            .process_event(Event::UpdateSubtitle(new_subtitle));
                        crate::effects::process_effects(&ctx_idle, effects);
                    });
                }
            }
        },
    );

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
    // The variant strip lives outside the (centered) toolbar box so it can
    // span the full window width — the toolbar no longer constrains it.
    container.append(&bottom_revealer);
    container.append(&status_stack);
    container.append(&subtitle_stack);

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
        embed_project_data: Rc::new(Cell::new(settings.embed_project_data.unwrap_or(true))),
        tool: tool.clone(),
        status: status_label.clone(),
        subtitle: subtitle_label.clone(),
        export_btn: export_btn.clone(),
        settings_label: settings_label.clone(),
        current_lang,
        tool_labels: Rc::new(tool_labels),
        tool_buttons: Rc::new(tool_buttons),
        variant_buttons,
        history_buttons: history_buttons.clone(),
        shortcuts: shortcuts.clone(),
        menu_labels,
        scrolled_window: scrolled_window.clone(),
        table: Rc::new(RefCell::new(None)),
        split_overlays: split_overlays.clone(),
        last_message: Rc::new(RefCell::new(None)),
        status_timer: Rc::new(RefCell::new(None)),
        settings_timer: Rc::new(RefCell::new(None)),
        last_update_check_day: Rc::new(Cell::new(settings.last_update_check_day)),
        skipped_version: Rc::new(RefCell::new(settings.skipped_version.clone())),
        autosave: crate::autosave::new_state(),
    };

    *ctx_holder.borrow_mut() = Some(ctx.clone());

    // Dynamically collapse tool button labels into icon-only mode when the
    // available width is too narrow to fit the left controls, full tool bar,
    // and Undo/Redo buttons without overflowing.
    {
        let tool_labels = ctx.tool_labels.clone();
        let left_box = left_box.clone();
        let history_box = history_buttons.widget().clone();
        let tool_bar_row = tool_bar_row.clone();
        let top_bar_clone = top_bar.clone();
        let last_expanded_tools_width = Rc::new(Cell::new(0));
        let labels_visible = Rc::new(Cell::new(true));

        top_bar.add_tick_callback(move |bar, _| {
            let allocated = bar.width();
            if allocated <= 0 {
                return glib::ControlFlow::Continue;
            }

            let (_, left_nat, _, _) = left_box.measure(gtk4::Orientation::Horizontal, -1);
            let (_, right_nat, _, _) = history_box.measure(gtk4::Orientation::Horizontal, -1);
            let (_, tools_nat, _, _) = tool_bar_row.measure(gtk4::Orientation::Horizontal, -1);

            if labels_visible.get() {
                last_expanded_tools_width.set(tools_nat);
            }

            // CenterBox centers the middle widget, so both side slots effectively
            // reserve `2 * max(left, right)` plus spacing, or at minimum `left + right`.
            let side_reserve = left_nat.max(right_nat) * 2 + 32;
            let needed_expanded = last_expanded_tools_width.get().max(tools_nat) + side_reserve;

            // Hysteresis buffer (16px) prevents rapid toggling on the boundary
            let should_show = if labels_visible.get() {
                allocated >= needed_expanded
            } else {
                allocated >= needed_expanded + 16
            };

            if should_show != labels_visible.get() {
                labels_visible.set(should_show);
                for (_, lbl) in tool_labels.iter() {
                    lbl.set_visible(should_show);
                }
                top_bar_clone.queue_resize();
            }

            glib::ControlFlow::Continue
        });
    }

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
        let event = ctx_export.pdf_export_event();
        let effects = ctx_export.core.process_event(event);
        crate::effects::process_effects(&ctx_export, effects);
    });

    {
        let ctx_sc = ctx.clone();
        shortcuts.install(&window, Rc::new(move |action| run_action(&ctx_sc, action)));
    }

    let scroll_ctrl = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
    scroll_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let ctx_scroll = ctx.clone();
    let touchpad_accum = std::rc::Rc::new(std::cell::Cell::new(0.0f64));

    // Uses opinionated, inverted axis relative to GTK, so Up = Next, Down = Prev
    scroll_ctrl.connect_scroll(move |ctrl, _dx, dy| {
        if !ctrl
            .current_event_state()
            .intersects(gtk4::gdk::ModifierType::CONTROL_MASK)
        {
            touchpad_accum.set(0.0);
            return glib::Propagation::Proceed;
        }

        match ctrl.unit() {
            // Touchpad / smooth scrolling: accumulate distance before triggering a step
            gtk4::gdk::ScrollUnit::Surface => {
                const STEP_THRESHOLD_PX: f64 = 40.0;
                let mut accum = touchpad_accum.get();

                // Reset accumulator immediately if swipe direction reverses
                if (accum > 0.0 && dy > 0.0) || (accum < 0.0 && dy < 0.0) {
                    accum = 0.0;
                }

                accum -= dy;

                while accum >= STEP_THRESHOLD_PX {
                    run_action(&ctx_scroll, Action::PrevVariant);
                    accum -= STEP_THRESHOLD_PX;
                }
                while accum <= -STEP_THRESHOLD_PX {
                    run_action(&ctx_scroll, Action::NextVariant);
                    accum += STEP_THRESHOLD_PX;
                }

                touchpad_accum.set(accum);
            }
            // Discrete mouse wheel click: trigger immediately per physical notch
            _ => {
                touchpad_accum.set(0.0);
                let action = if dy > 0.0 {
                    Action::PrevVariant
                } else {
                    Action::NextVariant
                };
                run_action(&ctx_scroll, action);
            }
        }

        glib::Propagation::Stop
    });
    window.add_controller(scroll_ctrl);

    // Extra mouse buttons (M3/M4/M5, ...) can be bound like keys. The gesture
    // runs in the capture phase and claims the press for bound buttons only,
    // so every other click keeps working with the app's own gestures.
    let button_ctrl = gtk4::GestureClick::new();
    button_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);
    // Button 0 listens to every button (the default of 1 only sees the
    // primary button); bound buttons are claimed and dispatched, the rest
    // is explicitly denied so the app's own gestures keep working.
    button_ctrl.set_button(0);
    let ctx_btn = ctx.clone();
    button_ctrl.connect_pressed(move |gesture, _n_press, _x, _y| {
        let button = gesture.current_button();
        let mods = gesture.current_event_state() & gtk4::accelerator_get_default_mod_mask();
        if let Some(action) = ctx_btn.shortcuts.mouse_action(button, mods)
            && !ctx_btn.shortcuts.should_defer(action, &ctx_btn.window)
        {
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            run_action(&ctx_btn, action);
        } else {
            gesture.set_state(gtk4::EventSequenceState::Denied);
        }
    });
    window.add_controller(button_ctrl);

    if let Some(saved) = settings.saved_language()
        && saved != ctx.current_lang.get()
    {
        defer_event(&ctx, Event::ToggleLanguage);
    }

    let ctx_close = ctx.clone();
    window.connect_close_request(move |_| {
        ctx_close.flush_pending_settings();
        // A normal exit must never leave a recoverable autosave behind.
        crate::autosave::on_exit(&ctx_close);
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
        Action::Save => defer_event(ctx, Event::RequestSave(ctx.export_settings())),
        Action::SaveAs => defer_event(ctx, Event::RequestSaveAs(ctx.export_settings())),
        Action::Undo => defer_event(ctx, Event::Undo),
        Action::Redo => defer_event(ctx, Event::Redo),
        Action::Export => defer_event(ctx, ctx.pdf_export_event()),
        Action::NextVariant => cycle_tool_variant(ctx, true),
        Action::PrevVariant => cycle_tool_variant(ctx, false),
        tool_action => {
            if let Some(tool) = tool_action.tool() {
                select_tool(ctx, tool);
            }
        }
    }
}

fn cycle_tool_variant(ctx: &AppContext, forward: bool) {
    let current_tool = ctx.tool.get();
    if let Some(next_variant) = current_tool.cycle_variant(forward)
        && let Some((_, btn)) = ctx
            .variant_buttons
            .borrow()
            .iter()
            .find(|(t, _)| *t == next_variant)
    {
        btn.set_active(true);
        scroll_variant_into_view(btn);
    }
}

pub fn scroll_variant_into_view(btn: &gtk4::ToggleButton) {
    let Some(scrolled) = btn
        .ancestor(gtk4::ScrolledWindow::static_type())
        .and_then(|w| w.downcast::<gtk4::ScrolledWindow>().ok())
    else {
        return;
    };
    let Some(parent) = btn.parent() else {
        return;
    };
    let Some(bounds) = btn.compute_bounds(&parent) else {
        return;
    };

    let hadj = scrolled.hadjustment();
    let btn_center = bounds.x() as f64 + (bounds.width() as f64 / 2.0);
    let page = hadj.page_size();
    let max_val = (hadj.upper() - page).max(0.0);

    // Center the button in the middle of the visible viewport
    let target = btn_center - (page / 2.0);
    hadj.set_value(target.clamp(0.0, max_val));
}
