use crate::settings::Settings;
use crate::shortcuts::ShortcutRegistry;
use crate::tool::Tool;
use crate::ui::widgets::history_buttons::HistoryButtons;
use chronos::bridge::{ConservationApp, Event};
use chronos::context::COLUMN_COUNT;
use chronos::i18n::{Language, Translations};
use chronos::project::ExportSettings;
use crux_core::Core;
use gtk4::glib;
use gtk4::{ApplicationWindow, Button, Label, ScrolledWindow, ToggleButton};
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ColumnVisibility {
    pub flags: [bool; COLUMN_COUNT],
}

impl Default for ColumnVisibility {
    fn default() -> Self {
        Self {
            flags: [true; COLUMN_COUNT],
        }
    }
}

pub use chronos::context::CanvasParams;

pub struct MenuLabels {
    pub file_btn: Label,
    pub edit_btn: Label,
    pub help_btn: Label,
    pub help_help: Label,
    pub help_about: Label,
    pub help_check: Label,
    pub file_open: Label,
    pub file_save: Label,
    pub file_save_as: Label,
    pub file_export: Label,
    pub edit_undo: Label,
    pub edit_redo: Label,
    pub edit_canvas: Label,
    pub edit_prefs: Label,
    pub edit_shortcuts: Label,
}

impl MenuLabels {
    pub fn update(&self, t: &Translations) {
        self.file_btn.set_text(t.menu_file);
        self.edit_btn.set_text(t.menu_edit);
        self.help_btn.set_text(t.menu_about);
        self.help_help.set_text(t.menu_help);
        self.help_about.set_text(t.menu_about);
        self.help_check.set_text(t.menu_check_updates);
        self.file_open.set_text(t.menu_open);
        self.file_save.set_text(t.menu_save);
        self.file_save_as.set_text(t.menu_save_as);
        self.file_export.set_text(t.menu_export);
        self.edit_undo.set_text(t.menu_undo);
        self.edit_redo.set_text(t.menu_redo);
        self.edit_canvas.set_text(t.menu_canvas_params);
        self.edit_prefs.set_text(t.menu_preferences);
        self.edit_shortcuts.set_text(t.menu_shortcuts);
    }
}

#[derive(Clone)]
pub struct AppContext {
    pub window: ApplicationWindow,
    pub core: Rc<Core<ConservationApp>>,
    pub vis: Rc<RefCell<ColumnVisibility>>,
    pub canvas_params: Rc<RefCell<CanvasParams>>,
    /// Whether exported PDFs carry the project for re-opening (on by default).
    pub embed_project_data: Rc<Cell<bool>>,
    pub tool: Rc<Cell<Tool>>,
    pub status: Label,
    pub subtitle: Label,
    pub export_btn: Button,
    pub current_lang: Rc<Cell<Language>>,
    pub settings_label: Label,
    pub tool_labels: Rc<Vec<(Tool, Label)>>,
    /// One toggle per toolbar tool family; used by the tool shortcuts so the
    /// toolbar highlight and variant pickers stay in sync with `tool`.
    pub tool_buttons: Rc<Vec<(Tool, ToggleButton)>>,
    /// One toggle per tool variant (Paint patterns, Merge directions); used by
    /// the variant cycling actions to switch through them like mouse clicks.
    pub variant_buttons: Rc<RefCell<Vec<(Tool, ToggleButton)>>>,
    /// Undo / redo buttons to the right of the screen.
    /// Enabled state refreshed by AppViewModel::{can_undo, can_redo}.
    pub history_buttons: Rc<HistoryButtons>,
    pub shortcuts: Rc<ShortcutRegistry>,
    pub menu_labels: Rc<MenuLabels>,
    pub scrolled_window: ScrolledWindow,
    /// The grid currently shown in `scrolled_window` together with handles to
    /// its cells. `None` until the first call to `ui::table::sync_document_table`.
    pub table: Rc<RefCell<Option<crate::ui::table::DocumentTable>>>,
    /// Split-line overlays of the merged cells in the current grid; toggled
    /// by `ui::table::update_split_overlays` whenever `tool` changes.
    pub split_overlays: crate::ui::table::SplitOverlays,
    /// Tracks the last displayed message to avoid re-triggering identical renders.
    pub last_message: Rc<RefCell<Option<String>>>,
    /// Pending fade-out timer; canceled whenever a new message appears.
    pub status_timer: Rc<RefCell<Option<glib::SourceId>>>,
    /// Pending debounce timer for saving settings to disk.
    pub settings_timer: Rc<RefCell<Option<glib::SourceId>>>,
    /// Unix day number of the last automatic update check (see `Settings`).
    pub last_update_check_day: Rc<Cell<Option<u64>>>,
    /// Release version the user chose to skip (see `Settings`).
    pub skipped_version: Rc<RefCell<Option<String>>>,
    /// Crash-recovery autosave state (see `crate::autosave`).
    pub autosave: Rc<RefCell<crate::autosave::AutosaveState>>,
}

impl AppContext {
    pub fn export_settings(&self) -> ExportSettings {
        ExportSettings {
            visible_columns: self.vis.borrow().flags,
            canvas: self.canvas_params.borrow().clone(),
        }
    }

    pub fn pdf_export_event(&self) -> Event {
        Event::RequestPdfExport {
            settings: self.export_settings(),
            embed_project_data: self.embed_project_data.get(),
        }
    }

    /// Saves language, visible columns, canvas parameters and any customised
    /// shortcuts to the user's settings file. Cheap enough to call on every
    /// change; failures are logged and never interrupt the user.
    /// Immediately writes current settings to disk.
    pub fn save_settings_now(&self) {
        let settings = Settings {
            language: Some(Settings::language_code(self.current_lang.get()).to_string()),
            shortcuts: self.shortcuts.overrides(),
            columns: self.vis.borrow().clone(),
            canvas: self.canvas_params.borrow().clone(),
            embed_project_data: Some(self.embed_project_data.get()),
            last_update_check_day: self.last_update_check_day.get(),
            skipped_version: self.skipped_version.borrow().clone(),
        };
        if let Err(e) = settings.save() {
            eprintln!("settings: could not save: {e}");
        }
    }

    pub fn persist_settings(&self) {
        if let Some(source_id) = self.settings_timer.borrow_mut().take() {
            source_id.remove();
        }

        let ctx = self.clone();
        let timer_slot = self.settings_timer.clone();
        let id = glib::timeout_add_local_once(Duration::from_millis(250), move || {
            *timer_slot.borrow_mut() = None;
            ctx.save_settings_now();
        });
        *self.settings_timer.borrow_mut() = Some(id);
    }

    /// Cancels any pending debounce timer and writes settings immediately.
    pub fn flush_pending_settings(&self) {
        if let Some(source_id) = self.settings_timer.borrow_mut().take() {
            source_id.remove();
            self.save_settings_now();
        }
    }
}
