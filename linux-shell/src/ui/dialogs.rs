use crate::context::AppContext;
use crate::context::CanvasParams;
use crate::shortcuts::{Action, AssignError};
use crate::ui::project_io::{project_file_filters, read_project_text};
use chronos::bridge::Event;
use chronos::i18n::Language;
use chronos::project::{ExportSettings, ProjectFile};
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const PAPER_SIZES: [&str; 5] = ["A4", "A3", "A5", "us-letter", "us-legal"];

fn paper_size_index(size: &str) -> u32 {
    let normalized = size.trim().to_ascii_lowercase();
    PAPER_SIZES
        .iter()
        .position(|s| s.to_ascii_lowercase() == normalized)
        .unwrap_or(0) as u32
}

pub fn show_canvas_params_dialog(ctx: &AppContext) {
    let view = ctx.core.view();
    let t = &view.translations;

    let win = gtk4::Window::builder()
        .transient_for(&ctx.window)
        .modal(true)
        .title(&t.menu_canvas_params)
        .default_width(380)
        .build();

    let vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let params = ctx.canvas_params.borrow().clone();
    let flags = ctx.vis.borrow().flags;

    // Paper Size
    let paper_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let paper_lbl = gtk4::Label::builder()
        .label(&t.dialog_paper_size)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    let paper_dropdown = gtk4::DropDown::from_strings(&PAPER_SIZES);
    paper_dropdown.set_selected(paper_size_index(&params.paper_size));
    paper_row.append(&paper_lbl);
    paper_row.append(&paper_dropdown);
    vbox.append(&paper_row);

    // Landscape Orientation
    let landscape_check = gtk4::CheckButton::with_label(&t.dialog_landscape);
    landscape_check.set_active(params.landscape);
    vbox.append(&landscape_check);

    // Page Margin
    let margin_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let margin_lbl = gtk4::Label::builder()
        .label(&t.dialog_page_margin)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    let margin_spin = gtk4::SpinButton::with_range(5.0, 50.0, 1.0);
    margin_spin.set_value(params.page_margin_mm);
    margin_row.append(&margin_lbl);
    margin_row.append(&margin_spin);
    vbox.append(&margin_row);

    // Cell Padding
    let padding_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let padding_lbl = gtk4::Label::builder()
        .label(&t.dialog_cell_padding)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    let padding_spin = gtk4::SpinButton::with_range(0.0, 20.0, 0.5);
    padding_spin.set_value(params.cell_padding_mm);
    padding_row.append(&padding_lbl);
    padding_row.append(&padding_spin);
    vbox.append(&padding_row);

    // Bar / Pattern Cell Padding
    let bar_padding_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let bar_padding_lbl = gtk4::Label::builder()
        .label(&t.dialog_bar_padding)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    let bar_padding_spin = gtk4::SpinButton::with_range(0.0, 20.0, 0.5);
    bar_padding_spin.set_value(params.bar_padding_mm);
    bar_padding_row.append(&bar_padding_lbl);
    bar_padding_row.append(&bar_padding_spin);
    vbox.append(&bar_padding_row);

    // Zero-Width Line
    let zero_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let zero_lbl = gtk4::Label::builder()
        .label(&t.dialog_zero_width_line)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    let zero_spin = gtk4::SpinButton::with_range(0.1, 5.0, 0.1);
    zero_spin.set_value(params.zero_width_line_pt);
    zero_row.append(&zero_lbl);
    zero_row.append(&zero_spin);
    vbox.append(&zero_row);

    // Embed the project data to exported PDFs
    let embed_check = gtk4::CheckButton::with_label(&t.dialog_embed_project_data);
    embed_check.set_active(ctx.embed_project_data.get());
    let ctx_embed = ctx.clone();
    embed_check.connect_toggled(move |c| {
        ctx_embed.embed_project_data.set(c.is_active());
        ctx_embed.persist_settings();
    });
    vbox.append(&embed_check);

    // Clone the top-level widgets to use inside the restore closure
    let paper_dropdown_clone = paper_dropdown.clone();
    let landscape_check_clone = landscape_check.clone();
    let margin_spin_clone = margin_spin.clone();
    let padding_spin_clone = padding_spin.clone();
    let bar_padding_spin_clone = bar_padding_spin.clone();
    let zero_spin_clone = zero_spin.clone();
    let mut col_checks = Vec::new();
    let mut col_spins = Vec::new();

    // Connect standard handlers
    let ctx_paper = ctx.clone();
    paper_dropdown.connect_selected_notify(move |dd| {
        let idx = dd.selected() as usize;
        if let Some(&size) = PAPER_SIZES.get(idx) {
            ctx_paper.canvas_params.borrow_mut().paper_size = size.to_string();
            ctx_paper.persist_settings();
        }
    });

    let ctx_landscape = ctx.clone();
    landscape_check.connect_toggled(move |chk| {
        ctx_landscape.canvas_params.borrow_mut().landscape = chk.is_active();
        ctx_landscape.persist_settings();
    });

    let ctx_margin = ctx.clone();
    margin_spin.connect_value_changed(move |s| {
        ctx_margin.canvas_params.borrow_mut().page_margin_mm = s.value();
        ctx_margin.persist_settings();
    });

    let ctx_padding = ctx.clone();
    padding_spin.connect_value_changed(move |s| {
        ctx_padding.canvas_params.borrow_mut().cell_padding_mm = s.value();
        ctx_padding.persist_settings();
    });

    let ctx_bar_padding = ctx.clone();
    bar_padding_spin.connect_value_changed(move |s| {
        ctx_bar_padding.canvas_params.borrow_mut().bar_padding_mm = s.value();
        ctx_bar_padding.persist_settings();
    });

    let ctx_zero = ctx.clone();
    zero_spin.connect_value_changed(move |s| {
        ctx_zero.canvas_params.borrow_mut().zero_width_line_pt = s.value();
        ctx_zero.persist_settings();
    });

    // Separator before columns
    let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    separator.set_margin_top(12);
    separator.set_margin_bottom(12);
    vbox.append(&separator);

    let col_heading = gtk4::Label::builder()
        .label(format!(
            "<b>{}</b>",
            glib::markup_escape_text(&t.dialog_visible_columns_heading)
        ))
        .use_markup(true)
        .halign(gtk4::Align::Start)
        .margin_bottom(8)
        .build();
    vbox.append(&col_heading);

    let col_names = [
        t.col_no.clone(),
        t.col_graphical.clone(),
        t.col_woven.clone(),
        t.col_char.clone(),
        t.col_thickness.clone(),
        t.col_chrono.clone(),
        t.col_dating.clone(),
    ];

    for i in 0..7 {
        let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let check = gtk4::CheckButton::new();
        check.set_active(flags[i]);

        let label = gtk4::Label::builder()
            .label(&col_names[i])
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .hexpand(true)
            .build();

        let spin = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
        spin.set_value(params.column_ratios[i] as f64);

        // Store clones for the restore button
        col_checks.push(check.clone());
        col_spins.push(spin.clone());

        row_box.append(&check);
        row_box.append(&label);
        row_box.append(&spin);
        vbox.append(&row_box);

        let ctx_chk = ctx.clone();
        check.connect_toggled(move |c| {
            ctx_chk.vis.borrow_mut().flags[i] = c.is_active();
            let view = ctx_chk.core.view();
            crate::ui::table::sync_document_table(&view, &ctx_chk);
            ctx_chk.persist_settings();
        });

        let ctx_ratio = ctx.clone();
        spin.connect_value_changed(move |s| {
            ctx_ratio.canvas_params.borrow_mut().column_ratios[i] = s.value() as u32;
            ctx_ratio.persist_settings();
        });
    }

    // Create the Restore Defaults button
    let restore_btn = gtk4::Button::builder()
        .label(&t.dialog_restore_defaults)
        .build();

    let paper_dropdown_clone2 = paper_dropdown_clone.clone();
    let landscape_check_clone2 = landscape_check_clone.clone();
    let margin_spin_clone2 = margin_spin_clone.clone();
    let padding_spin_clone2 = padding_spin_clone.clone();
    let bar_padding_spin_clone2 = bar_padding_spin_clone.clone();
    let zero_spin_clone2 = zero_spin_clone.clone();
    let col_checks2 = col_checks.clone();
    let col_spins2 = col_spins.clone();
    let ctx_restore = ctx.clone();
    let embed_restore = embed_check.clone();

    restore_btn.connect_clicked(move |_| {
        let default_params = CanvasParams::default();
        paper_dropdown_clone2.set_selected(paper_size_index(&default_params.paper_size));
        landscape_check_clone2.set_active(default_params.landscape);
        margin_spin_clone2.set_value(default_params.page_margin_mm);
        padding_spin_clone2.set_value(default_params.cell_padding_mm);
        bar_padding_spin_clone2.set_value(default_params.bar_padding_mm);
        zero_spin_clone2.set_value(default_params.zero_width_line_pt);
        for i in 0..7 {
            col_checks2[i].set_active(true);
            col_spins2[i].set_value(default_params.column_ratios[i] as f64);
        }
        embed_restore.set_active(true);
        ctx_restore.persist_settings();
    });

    // Import the export settings
    let import_btn = gtk4::Button::with_label(&t.dialog_import_from_file);
    let canvas_widgets = CanvasWidgets {
        paper: paper_dropdown.clone(),
        landscape: landscape_check.clone(),
        margin: margin_spin.clone(),
        padding: padding_spin.clone(),
        bar_padding: bar_padding_spin.clone(),
        zero_width: zero_spin.clone(),
        col_checks: col_checks.clone(),
        col_spins: col_spins.clone(),
    };
    let ctx_import = ctx.clone();
    let win_import = win.clone();
    import_btn.connect_clicked(move |_| {
        let ctx = ctx_import.clone();
        let win = win_import.clone();
        let widgets = canvas_widgets.clone();
        glib::spawn_future_local(async move {
            import_export_settings(&ctx, &win, &widgets).await;
        });
    });

    let button_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    button_row.set_margin_top(12);
    button_row.set_halign(gtk4::Align::Start);
    button_row.append(&restore_btn);
    button_row.append(&import_btn);
    vbox.append(&button_row);

    let close_btn = gtk4::Button::with_label(&t.dialog_close);
    close_btn.set_halign(gtk4::Align::End);
    let win_close = win.clone();
    close_btn.connect_clicked(move |_| {
        win_close.close();
    });
    vbox.append(&close_btn);

    win.set_child(Some(&vbox));
    win.present();
}

#[derive(Clone)]
struct CanvasWidgets {
    paper: gtk4::DropDown,
    landscape: gtk4::CheckButton,
    margin: gtk4::SpinButton,
    padding: gtk4::SpinButton,
    bar_padding: gtk4::SpinButton,
    zero_width: gtk4::SpinButton,
    col_checks: Vec<gtk4::CheckButton>,
    col_spins: Vec<gtk4::SpinButton>,
}

impl CanvasWidgets {
    fn show(&self, settings: &ExportSettings) {
        let canvas = &settings.canvas;
        self.paper.set_selected(paper_size_index(&canvas.paper_size));
        self.landscape.set_active(canvas.landscape);
        self.margin.set_value(canvas.page_margin_mm);
        self.padding.set_value(canvas.cell_padding_mm);
        self.bar_padding.set_value(canvas.bar_padding_mm);
        self.zero_width.set_value(canvas.zero_width_line_pt);
        for i in 0..7 {
            self.col_checks[i].set_active(settings.visible_columns[i]);
            self.col_spins[i].set_value(canvas.column_ratios[i] as f64);
        }
    }
}

fn show_message(parent: &gtk4::Window, message: &str) {
    gtk4::AlertDialog::builder()
        .message(message)
        .modal(true)
        .build()
        .show(Some(parent));
}

async fn import_export_settings(ctx: &AppContext, parent: &gtk4::Window, widgets: &CanvasWidgets) {
    let t = ctx.core.view().translations;

    let dialog = gtk4::FileDialog::new();
    dialog.set_title(&t.dialog_open_title);
    dialog.set_filters(Some(&project_file_filters(&t.filter_project_name)));
    let Ok(file) = dialog.open_future(Some(parent)).await else {
        return; // cancelled
    };
    let Some(path) = file.path() else {
        return;
    };

    let outcome = read_project_text(&path)
        .map_err(|e| format!("{}{}", t.error_open_prefix, e.describe(&t)))
        .and_then(|text| {
            ProjectFile::parse(&text).map_err(|e| format!("{}{}", t.error_parse_prefix, e))
        })
        .and_then(|project| {
            project
                .export_settings
                .ok_or_else(|| t.error_no_export_settings.clone())
        });

    match outcome {
        Ok(settings) => {
            *ctx.canvas_params.borrow_mut() = settings.canvas.clone();
            widgets.show(&settings);
            ctx.persist_settings();
        }
        Err(message) => show_message(parent, &message),
    }
}

pub fn show_preferences_dialog(ctx: &AppContext) {
    let view = ctx.core.view();
    let t = &view.translations;
    let win = gtk4::Window::builder()
        .transient_for(&ctx.window)
        .modal(true)
        .title(&t.menu_preferences)
        .default_width(340)
        .build();

    let vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let lang_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let lang_lbl = gtk4::Label::builder()
        .label(&t.dialog_interface_language)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();

    let lang_dropdown = gtk4::DropDown::from_strings(&["English", "Polski"]);
    lang_dropdown.set_selected(match ctx.current_lang.get() {
        Language::English => 0,
        Language::Polish => 1,
    });

    let close_btn = gtk4::Button::with_label(&t.dialog_close);
    close_btn.set_halign(gtk4::Align::End);

    let ctx_lang = ctx.clone();
    let win_lang = win.clone();
    let lang_lbl_clone = lang_lbl.clone();
    let close_btn_clone = close_btn.clone();
    lang_dropdown.connect_selected_notify(move |dd| {
        let selected_lang = if dd.selected() == 0 {
            Language::English
        } else {
            Language::Polish
        };
        if selected_lang != ctx_lang.current_lang.get() {
            let ctx_idle = ctx_lang.clone();
            let win_idle = win_lang.clone();
            let lbl_idle = lang_lbl_clone.clone();
            let btn_idle = close_btn_clone.clone();
            glib::idle_add_local_once(move || {
                let effects = ctx_idle.core.process_event(Event::ToggleLanguage);
                crate::effects::process_effects(&ctx_idle, effects);
                let new_t = &ctx_idle.core.view().translations;
                win_idle.set_title(Some(&new_t.menu_preferences));
                lbl_idle.set_text(&new_t.dialog_interface_language);
                btn_idle.set_label(&new_t.dialog_close);
            });
        }
    });

    lang_row.append(&lang_lbl);
    lang_row.append(&lang_dropdown);
    vbox.append(&lang_row);

    let win_close = win.clone();
    close_btn.connect_clicked(move |_| {
        win_close.close();
    });
    vbox.append(&close_btn);

    win.set_child(Some(&vbox));
    win.present();
}

// --- Keyboard shortcuts ----------------------------------------------------

/// Widgets and state shared by the rows of the shortcuts dialog.
#[derive(Clone)]
struct ShortcutDialogState {
    /// The action currently waiting for a key press, if any.
    capturing: Rc<Cell<Option<Action>>>,
    message: gtk4::Label,
    key_buttons: Rc<RefCell<Vec<(Action, gtk4::Button)>>>,
    reset_buttons: Rc<RefCell<Vec<(Action, gtk4::Button)>>>,
}

impl ShortcutDialogState {
    /// Re-reads every binding from the registry into the buttons.
    fn refresh(&self, ctx: &AppContext) {
        let view = ctx.core.view();
        let t = &view.translations;
        let capturing = self.capturing.get();
        for (action, btn) in self.key_buttons.borrow().iter() {
            if capturing == Some(*action) {
                btn.set_label(&t.shortcuts_press_new);
                btn.add_css_class("suggested-action");
            } else {
                btn.set_label(&ctx.shortcuts.label(*action));
                btn.remove_css_class("suggested-action");
            }
        }
        for (action, btn) in self.reset_buttons.borrow().iter() {
            btn.set_sensitive(!ctx.shortcuts.is_default(*action));
        }
    }

    fn set_message(&self, text: &str, is_error: bool) {
        self.message.set_text(text);
        if is_error {
            self.message.add_css_class("error");
        } else {
            self.message.remove_css_class("error");
        }
    }

    fn show_error(&self, ctx: &AppContext, err: AssignError) {
        let view = ctx.core.view();
        let t = &view.translations;
        let text = match err {
            AssignError::Invalid => t.shortcuts_err_invalid.clone(),
            AssignError::NeedsModifier => t.shortcuts_err_needs_modifier.clone(),
            AssignError::Conflict(other) => {
                format!(
                    "{}\"{}\".",
                    t.shortcuts_err_conflict_prefix,
                    other.display_name(t).replace('\n', " ")
                )
            }
        };
        self.set_message(&text, true);
    }
}

fn build_shortcut_row(
    ctx: &AppContext,
    state: &ShortcutDialogState,
    action: Action,
    name: &str,
) -> gtk4::Box {
    let view = ctx.core.view();
    let t = &view.translations;
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let name_lbl = gtk4::Label::builder()
        .label(name)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .xalign(0.0)
        .build();

    let key_btn = gtk4::Button::builder()
        .width_request(170)
        .tooltip_text(&t.shortcuts_click_tooltip)
        .build();

    let reset_btn = gtk4::Button::builder()
        .icon_name("edit-undo-symbolic")
        .has_frame(false)
        .tooltip_text(&t.shortcuts_restore_default_tooltip)
        .build();
    reset_btn.add_css_class("flat");

    {
        let ctx = ctx.clone();
        let state = state.clone();
        key_btn.connect_clicked(move |_| {
            state.capturing.set(Some(action));
            let prompt = ctx.core.view().translations.shortcuts_prompt_cancel.clone();
            state.set_message(&prompt, false);
            state.refresh(&ctx);
        });
    }

    {
        let ctx = ctx.clone();
        let state = state.clone();
        reset_btn.connect_clicked(move |_| match ctx.shortcuts.reset(action) {
            Ok(()) => {
                if state.capturing.get() == Some(action) {
                    state.capturing.set(None);
                }
                state.set_message("", false);
                ctx.persist_settings();
                state.refresh(&ctx);
            }
            Err(err) => state.show_error(&ctx, err),
        });
    }

    state
        .key_buttons
        .borrow_mut()
        .push((action, key_btn.clone()));
    state
        .reset_buttons
        .borrow_mut()
        .push((action, reset_btn.clone()));

    row.append(&name_lbl);
    row.append(&key_btn);
    row.append(&reset_btn);
    row
}

fn section_header(title: &str) -> gtk4::Label {
    gtk4::Label::builder()
        .label(format!("<b>{}</b>", glib::markup_escape_text(title)))
        .use_markup(true)
        .halign(gtk4::Align::Start)
        .margin_top(6)
        .build()
}

pub fn show_shortcuts_dialog(ctx: &AppContext) {
    let view = ctx.core.view();
    let t = &view.translations;

    let win = gtk4::Window::builder()
        .transient_for(&ctx.window)
        .modal(true)
        .title(&t.menu_shortcuts)
        .default_width(460)
        .build();

    let state = ShortcutDialogState {
        capturing: Rc::new(Cell::new(None)),
        message: gtk4::Label::builder()
            .halign(gtk4::Align::Start)
            .xalign(0.0)
            .wrap(true)
            .build(),
        key_buttons: Rc::new(RefCell::new(Vec::new())),
        reset_buttons: Rc::new(RefCell::new(Vec::new())),
    };

    // Right margin keeps the (overlay) scrollbar clear of the restore buttons.
    let list = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_end(18)
        .build();

    let sections: [(&str, &[Action]); 3] = [
        (t.menu_file.as_str(), &Action::FILE[..]),
        (t.menu_edit.as_str(), &Action::EDIT[..]),
        (t.shortcuts_section_tools.as_str(), &Action::TOOLS[..]),
    ];
    for (title, actions) in sections {
        list.append(&section_header(title));
        for &action in actions {
            let name = action.display_name(t).replace('\n', " ");
            list.append(&build_shortcut_row(ctx, &state, action, &name));
        }
    }

    // Fixed gestures: shown for reference, not editable.
    list.append(&section_header(&t.shortcuts_section_fixed));
    for (action, keys) in [
        (
            t.shortcuts_fixed_inline_edit.as_str(),
            t.shortcuts_key_double_click.as_str(),
        ),
        (t.shortcuts_fixed_cancel_edit.as_str(), "Esc"),
    ] {
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        let act_lbl = gtk4::Label::builder()
            .label(action)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .xalign(0.0)
            .build();
        let key_lbl = gtk4::Label::builder()
            .label(keys)
            .halign(gtk4::Align::End)
            .build();
        key_lbl.add_css_class("shortcut-hint");
        row.append(&act_lbl);
        row.append(&key_lbl);
        list.append(&row);
    }

    let scroller = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .propagate_natural_height(true)
        .max_content_height(520)
        .child(&list)
        .build();

    // While a row is waiting for a key, grab every key press before the
    // focused button (or anything else) can react to it.
    let key_ctrl = gtk4::EventControllerKey::new();
    key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);
    {
        let ctx = ctx.clone();
        let state = state.clone();
        key_ctrl.connect_key_pressed(move |_, key, _keycode, modifiers| {
            let Some(action) = state.capturing.get() else {
                return glib::Propagation::Proceed;
            };
            let mods = modifiers & gtk4::accelerator_get_default_mod_mask();

            if key == gtk4::gdk::Key::Escape && mods.is_empty() {
                state.capturing.set(None);
                state.set_message("", false);
                state.refresh(&ctx);
                return glib::Propagation::Stop;
            }
            // A bare Ctrl/Shift/Alt press: keep waiting for the real key.
            if !gtk4::accelerator_valid(key, mods) {
                return glib::Propagation::Stop;
            }

            let accel = gtk4::accelerator_name(key.to_lower(), mods).to_string();
            match ctx.shortcuts.try_set(action, &accel) {
                Ok(()) => {
                    state.capturing.set(None);
                    state.set_message("", false);
                    ctx.persist_settings();
                }
                // Stay in capture mode so another combination can be tried.
                Err(err) => state.show_error(&ctx, err),
            }
            state.refresh(&ctx);
            glib::Propagation::Stop
        });
    }
    win.add_controller(key_ctrl);

    let reset_all_btn = gtk4::Button::with_label(&t.shortcuts_restore_all);
    {
        let ctx = ctx.clone();
        let state = state.clone();
        reset_all_btn.connect_clicked(move |_| {
            ctx.shortcuts.reset_all();
            state.capturing.set(None);
            state.set_message("", false);
            ctx.persist_settings();
            state.refresh(&ctx);
        });
    }
    let close_btn = gtk4::Button::with_label(&t.dialog_close);
    let win_close = win.clone();
    close_btn.connect_clicked(move |_| {
        win_close.close();
    });
    let buttons = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .margin_top(8)
        .build();
    reset_all_btn.set_hexpand(true);
    reset_all_btn.set_halign(gtk4::Align::Start);
    buttons.append(&reset_all_btn);
    buttons.append(&close_btn);

    let vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(10)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(20)
        .margin_end(20)
        .build();
    vbox.append(&scroller);
    vbox.append(&state.message);
    vbox.append(&buttons);

    state.refresh(ctx);
    win.set_child(Some(&vbox));
    win.present();
}
