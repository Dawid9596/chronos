//! Rebindable keyboard shortcuts.
//!
//! * [`Action`] is everything that can be bound and its default key.
//! * [`ShortcutRegistry`] owns the *current* bindings, validates changes.
//!
//! Bindings are stored as GTK accelerator names such as `<Control>o` or `e`
//! (see `gtk::accelerator_parse`). Extra mouse buttons are not keys, so they
//! are stored as pseudo-accelerators like `mouse4` / `<Control>mouse5`
//! (see [`parse_mouse_accel`]) instead; they are captured and dispatched with
//! button gestures, and never installed on the keyboard shortcut controller.

use crate::tool::Tool;
use chronos::i18n::Translations;
use gtk4::prelude::*;
use gtk4::{gdk, glib};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Open,
    Save,
    SaveAs,
    Export,
    Undo,
    Redo,
    ToolEdit,
    ToolCreate,
    ToolDelete,
    ToolPaint,
    ToolSplit,
    ToolDuplicate,
    ToolMerge,
    NextVariant,
    PrevVariant,
}

impl Action {
    pub const FILE: [Action; 4] = [Action::Open, Action::Save, Action::SaveAs, Action::Export];

    /// Document history; listed under the "Edit" heading in the shortcuts dialog.
    pub const EDIT: [Action; 2] = [Action::Undo, Action::Redo];

    pub const TOOLS: [Action; Tool::TOOLBAR_DEFAULTS.len()] = {
        let mut out = [Action::Open; Tool::TOOLBAR_DEFAULTS.len()];
        let mut i = 0;
        while i < Tool::TOOLBAR_DEFAULTS.len() {
            out[i] = Self::tool_action(Tool::TOOLBAR_DEFAULTS[i]);
            i += 1;
        }
        out
    };

    /// Steps through a tool family's variants (Paint patterns, Merge directions).
    pub const VARIANTS: [Action; 2] = [Action::NextVariant, Action::PrevVariant];

    /// (File, Edit, Tools, Variants).
    pub const ALL: [Action; Self::FILE.len() + Self::EDIT.len() + Self::TOOLS.len() + Self::VARIANTS.len()] = {
        let mut out = [Action::Open; Self::FILE.len() + Self::EDIT.len() + Self::TOOLS.len() + Self::VARIANTS.len()];
        let mut i = 0;
        let mut src = 0;
        while src < Self::FILE.len() {
            out[i] = Self::FILE[src];
            i += 1;
            src += 1;
        }
        let mut src = 0;
        while src < Self::EDIT.len() {
            out[i] = Self::EDIT[src];
            i += 1;
            src += 1;
        }
        let mut src = 0;
        while src < Self::TOOLS.len() {
            out[i] = Self::TOOLS[src];
            i += 1;
            src += 1;
        }
        let mut src = 0;
        while src < Self::VARIANTS.len() {
            out[i] = Self::VARIANTS[src];
            i += 1;
            src += 1;
        }
        out
    };

    pub fn id(self) -> &'static str {
        match self {
            Action::Open => "open",
            Action::Save => "save",
            Action::SaveAs => "save_as",
            Action::Export => "export",
            Action::Undo => "undo",
            Action::Redo => "redo",
            Action::ToolEdit => "tool_edit",
            Action::ToolCreate => "tool_create",
            Action::ToolDelete => "tool_delete",
            Action::ToolPaint => "tool_paint",
            Action::ToolSplit => "tool_split",
            Action::ToolDuplicate => "tool_duplicate",
            Action::ToolMerge => "tool_merge",
            Action::NextVariant => "next_variant",
            Action::PrevVariant => "prev_variant",
        }
    }

    pub fn from_id(id: &str) -> Option<Action> {
        Self::ALL.into_iter().find(|a| a.id() == id)
    }

    fn default_accel(self) -> &'static str {
        match self {
            Action::Open => "<Control>o",
            Action::Save => "<Control>s",
            Action::SaveAs => "<Control><Shift>s",
            Action::Export => "<Control>e",
            Action::Undo => "<Control>z",
            Action::Redo => "<Control>y",
            Action::ToolEdit => "e",
            Action::ToolCreate => "c",
            Action::ToolDelete => "d",
            Action::ToolPaint => "p",
            Action::ToolSplit => "s",
            Action::ToolDuplicate => "r",
            Action::ToolMerge => "m",
            // Bare bracket characters are not valid accelerators (`accelerator_parse`
            // accepts single letters/numbers and keyval names only), so the keyval
            // names must be used here.
            Action::NextVariant => "bracketright",
            Action::PrevVariant => "bracketleft",
        }
    }

    /// Ensures a shorcut needs a modifier as bare keys
    /// are too easy to press accidentally.
    pub fn needs_modifier(self) -> bool {
        Self::FILE.contains(&self) || Self::EDIT.contains(&self)
    }

    pub fn defers_to_text_input(self) -> bool {
        Self::EDIT.contains(&self)
    }

    const fn tool_action(tool: Tool) -> Action {
        match tool {
            Tool::Edit => Action::ToolEdit,
            Tool::Create => Action::ToolCreate,
            Tool::Delete => Action::ToolDelete,
            Tool::Paint(_) => Action::ToolPaint,
            Tool::Split => Action::ToolSplit,
            Tool::Duplicate => Action::ToolDuplicate,
            Tool::Merge(_) => Action::ToolMerge,
        }
    }

    pub fn tool(self) -> Option<Tool> {
        Self::TOOLS
            .iter()
            .position(|a| *a == self)
            .map(|i| Tool::TOOLBAR_DEFAULTS[i])
    }

    pub fn for_tool(tool: Tool) -> Action {
        let idx = Tool::TOOLBAR_DEFAULTS
            .iter()
            .position(|t| t.same_kind(tool))
            .unwrap_or(0);
        Self::TOOLS[idx]
    }

    pub fn display_name(self, t: &Translations) -> String {
        match self {
            Action::Open => t.menu_open.to_string(),
            Action::Save => t.menu_save.to_string(),
            Action::SaveAs => t.menu_save_as.to_string(),
            Action::Export => t.menu_export.to_string(),
            Action::Undo => t.menu_undo.to_string(),
            Action::Redo => t.menu_redo.to_string(),
            Action::NextVariant => t.action_next_variant.to_string(),
            Action::PrevVariant => t.action_prev_variant.to_string(),
            _ => self
                .tool()
                .map(|tool| tool.label(t).to_string())
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignError {
    Invalid,
    /// File actions must include Ctrl or Alt.
    NeedsModifier,
    Conflict(Action),
}

fn command_mods() -> gdk::ModifierType {
    gdk::ModifierType::CONTROL_MASK
        | gdk::ModifierType::ALT_MASK
        | gdk::ModifierType::SUPER_MASK
        | gdk::ModifierType::META_MASK
}

pub fn has_command_modifier(accel: &str) -> bool {
    if let Some(b) = parse_mouse_accel(accel) {
        return b.mods.intersects(command_mods());
    }
    gtk4::accelerator_parse(accel)
        .map(|(_, mods)| mods.intersects(command_mods()))
        .unwrap_or(false)
}

/// Re-expresses an accelerator in GTK's canonical spelling.
pub fn normalize(accel: &str) -> Option<String> {
    if let Some(b) = parse_mouse_accel(accel) {
        return Some(mouse_string(b.button, b.mods));
    }
    let (key, mods) = gtk4::accelerator_parse(accel)?;
    Some(gtk4::accelerator_name(key.to_lower(), mods).to_string())
}

/// Human-readable form, e.g. `Ctrl+Shift+S`.
pub fn accel_label(accel: &str) -> String {
    if let Some(b) = parse_mouse_accel(accel) {
        return mouse_label(&b);
    }
    gtk4::accelerator_parse(accel)
        .map(|(key, mods)| gtk4::accelerator_get_label(key, mods).to_string())
        .unwrap_or_else(|| accel.to_string())
}

/// Bindings for extra mouse buttons, e.g. `mouse4`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseBinding {
    pub button: u32,
    pub mods: gdk::ModifierType,
}

/// Button range covered by the helper: buttons 2..=32. The primary button is
/// excluded — binding it would claim every click in the app — and higher
/// numbers do not exist as physical buttons.
const MOUSE_BUTTON_MIN: u32 = 2;
const MOUSE_BUTTON_MAX: u32 = 32;

/// Parses our mouse-button pseudo-accelerators. `mouse4` and
/// `<Control>mouse4` are supported, canonical modifiers only. These are not
/// keyval accelerators, so `gtk::accelerator_parse` cannot be used here.
pub fn parse_mouse_accel(accel: &str) -> Option<MouseBinding> {
    let mut mods = gdk::ModifierType::empty();
    let mut rest = accel;
    while let Some(inner) = rest.strip_prefix('<').and_then(|s| s.split_once('>')) {
        mods |= match inner.0.to_ascii_lowercase().as_str() {
            "control" | "ctrl" | "primary" => gdk::ModifierType::CONTROL_MASK,
            "shift" => gdk::ModifierType::SHIFT_MASK,
            "alt" => gdk::ModifierType::ALT_MASK,
            "super" | "meta" | "hyper" => gdk::ModifierType::SUPER_MASK,
            _ => return None,
        };
        rest = inner.1;
    }
    let button = rest.strip_prefix("mouse")?.parse::<u32>().ok()?;
    (MOUSE_BUTTON_MIN..=MOUSE_BUTTON_MAX).contains(&button)
        .then_some(MouseBinding { button, mods })
}

/// The canonical spelling of a mouse binding, e.g. `<Control>mouse4`.
pub fn mouse_string(button: u32, mods: gdk::ModifierType) -> String {
    let mut s = String::new();
    if mods.contains(gdk::ModifierType::CONTROL_MASK) {
        s.push_str("<Control>");
    }
    if mods.contains(gdk::ModifierType::SHIFT_MASK) {
        s.push_str("<Shift>");
    }
    if mods.contains(gdk::ModifierType::ALT_MASK) {
        s.push_str("<Alt>");
    }
    if mods.intersects(
        gdk::ModifierType::SUPER_MASK
            | gdk::ModifierType::META_MASK
            | gdk::ModifierType::HYPER_MASK,
    ) {
        s.push_str("<Super>");
    }
    format!("{s}mouse{button}")
}

/// What a mouse binding looks like to a human, e.g. `Ctrl+Mouse 4`.
fn mouse_label(&MouseBinding { button, mods }: &MouseBinding) -> String {
    let mut parts = Vec::new();
    if mods.contains(gdk::ModifierType::CONTROL_MASK) {
        parts.push("Ctrl".into());
    }
    if mods.contains(gdk::ModifierType::SHIFT_MASK) {
        parts.push("Shift".into());
    }
    if mods.contains(gdk::ModifierType::ALT_MASK) {
        parts.push("Alt".into());
    }
    if mods.intersects(
        gdk::ModifierType::SUPER_MASK
            | gdk::ModifierType::META_MASK
            | gdk::ModifierType::HYPER_MASK,
    ) {
        parts.push("Super".into());
    }
    parts.push(format!("Mouse {button}"));
    parts.join("+")
}

/// Matches both the lower- and upper-case keyval.
fn build_trigger(accel: &str) -> Option<gtk4::ShortcutTrigger> {
    if parse_mouse_accel(accel).is_some() {
        // Mouse buttons are dispatched by the window's button gesture, not by
        // the keyboard shortcut controller.
        return None;
    }
    let (key, mods) = gtk4::accelerator_parse(accel)?;
    let (lower, upper) = (key.to_lower(), key.to_upper());
    let first = gtk4::KeyvalTrigger::new(lower, mods);
    if lower == upper {
        return Some(first.upcast());
    }
    let second = gtk4::KeyvalTrigger::new(upper, mods);
    Some(gtk4::AlternativeTrigger::new(first, second).upcast())
}

/// Whether the window's focus sits on a text entry (the inline editing of
/// cell text and the project name are `gtk4::Editable` widgets).
fn window_has_text_focus(window: &gtk4::ApplicationWindow) -> bool {
    gtk4::prelude::GtkWindowExt::focus(window)
        .is_some_and(|focused| focused.is::<gtk4::Editable>())
}

fn text_input_has_focus(window: &glib::WeakRef<gtk4::ApplicationWindow>) -> bool {
    window
        .upgrade()
        .is_some_and(|win| window_has_text_focus(&win))
}

/// Whether `action` bound to `accel` must yield while a text entry has focus:
/// plain keys and tool keys would type into the entry or disturb the edit
/// underneath it. Mouse buttons never insert characters, so only actions that
/// directly conflict with editing (document Undo/Redo under an Entry's own
/// undo stack) defer.
fn conflicts_with_text_edit(action: Action, accel: &str) -> bool {
    if parse_mouse_accel(accel).is_some() {
        action.defers_to_text_input()
    } else {
        action.tool().is_some()
            || !has_command_modifier(accel)
            || action.defers_to_text_input()
    }
}

pub struct ShortcutRegistry {
    defaults: HashMap<Action, String>,
    /// Shared with the GTK callbacks so they always see the current binding.
    bindings: Rc<RefCell<HashMap<Action, String>>>,
    shortcuts: RefCell<Vec<(Action, gtk4::Shortcut)>>,
    hint_labels: RefCell<Vec<(Action, gtk4::Label)>>,
    tooltips: RefCell<Vec<(Action, gtk4::Widget, bool)>>,
    shortcut_prefix: RefCell<String>,
    action_names: RefCell<HashMap<Action, String>>,
}

impl ShortcutRegistry {
    pub fn new(overrides: &BTreeMap<String, String>) -> Self {
        let defaults: HashMap<Action, String> = Action::ALL
            .iter()
            .map(|&a| {
                // A default that does not normalize is a bug: the raw string is stored
                // and the binding gets no keyboard trigger and cannot ever be reset.
                let accel =
                    normalize(a.default_accel()).unwrap_or_else(|| a.default_accel().into());
                (a, accel)
            })
            .collect();

        let mut bindings = defaults.clone();
        for (id, accel) in overrides {
            let (Some(action), Some(accel)) = (Action::from_id(id), normalize(accel)) else {
                continue;
            };
            if action.needs_modifier() && !has_command_modifier(&accel) {
                continue;
            }
            bindings.insert(action, accel);
        }

        // Verify for and remove and clashes in shortcuts.
        loop {
            let mut seen: HashMap<String, Action> = HashMap::new();
            let mut clashing: Vec<Action> = Vec::new();
            for &action in &Action::ALL {
                if let Some(prev) = seen.insert(bindings[&action].clone(), action) {
                    clashing.push(prev);
                    clashing.push(action);
                }
            }
            let mut changed = false;
            for action in clashing {
                if bindings[&action] != defaults[&action] {
                    bindings.insert(action, defaults[&action].clone());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        Self {
            defaults,
            bindings: Rc::new(RefCell::new(bindings)),
            shortcuts: RefCell::new(Vec::new()),
            hint_labels: RefCell::new(Vec::new()),
            tooltips: RefCell::new(Vec::new()),
            shortcut_prefix: RefCell::new("Shortcut: ".to_string()),
            action_names: RefCell::new(HashMap::new()),
        }
    }

    pub fn accel(&self, action: Action) -> String {
        self.bindings.borrow()[&action].clone()
    }

    pub fn label(&self, action: Action) -> String {
        accel_label(&self.accel(action))
    }

    pub fn is_default(&self, action: Action) -> bool {
        self.bindings.borrow()[&action] == self.defaults[&action]
    }

    /// The action currently bound to `button` with `mods`, if any. Mouse
    /// bindings live in the same string map as keyboard ones, but only mouse
    /// pseudo-accelerators can match this form.
    pub fn mouse_action(&self, button: u32, mods: gdk::ModifierType) -> Option<Action> {
        let accel = mouse_string(button, mods);
        Action::ALL
            .into_iter()
            .find(|&a| self.accel(a) == accel)
    }

    /// Whether `action` must let the focused text entry handle the event
    /// instead. Shared by the keyboard shortcuts (see `install`) and by the
    /// window's mouse-button dispatch; see [`conflicts_with_text_edit`] for
    /// the exact rules.
    pub fn should_defer(&self, action: Action, window: &gtk4::ApplicationWindow) -> bool {
        window_has_text_focus(window) && conflicts_with_text_edit(action, &self.accel(action))
    }

    /// Shortcuts that differ from the defaults.
    pub fn overrides(&self) -> BTreeMap<String, String> {
        let bindings = self.bindings.borrow();
        Action::ALL
            .iter()
            .filter(|a| bindings[*a] != self.defaults[*a])
            .map(|a| (a.id().to_string(), bindings[a].clone()))
            .collect()
    }

    pub fn try_set(&self, action: Action, accel: &str) -> Result<(), AssignError> {
        let accel = normalize(accel).ok_or(AssignError::Invalid)?;
        if action.needs_modifier() && !has_command_modifier(&accel) {
            return Err(AssignError::NeedsModifier);
        }
        if let Some(other) = Action::ALL
            .into_iter()
            .find(|&other| other != action && self.accel(other) == accel)
        {
            return Err(AssignError::Conflict(other));
        }
        self.bindings.borrow_mut().insert(action, accel);
        self.apply(action);
        Ok(())
    }

    /// Restore default.
    pub fn reset(&self, action: Action) -> Result<(), AssignError> {
        let default = self.defaults[&action].clone();
        self.try_set(action, &default)
    }

    /// Restore default for all.
    pub fn reset_all(&self) {
        *self.bindings.borrow_mut() = self.defaults.clone();
        for action in Action::ALL {
            self.apply(action);
        }
    }

    /// Shows this action's shortcut in `label` and keeps it up to date.
    pub fn bind_hint_label(&self, action: Action, label: &gtk4::Label) {
        label.set_text(&self.label(action));
        self.hint_labels.borrow_mut().push((action, label.clone()));
    }

    /// Adds "Shortcut: X" as `widget`'s tooltip and keeps it up to date.
    pub fn bind_tooltip(&self, action: Action, widget: &impl IsA<gtk4::Widget>, t: &Translations) {
        self.bind_tooltip_inner(action, widget, t, false);
    }

    /// Adds "<Action Name> (Shortcut: X)" as `widget`'s tooltip and keeps it up to date.
    pub fn bind_named_tooltip(
        &self,
        action: Action,
        widget: &impl IsA<gtk4::Widget>,
        t: &Translations,
    ) {
        self.bind_tooltip_inner(action, widget, t, true);
    }

    fn bind_tooltip_inner(
        &self,
        action: Action,
        widget: &impl IsA<gtk4::Widget>,
        t: &Translations,
        include_name: bool,
    ) {
        self.sync_translation_cache(t);
        let widget = widget.upcast_ref::<gtk4::Widget>().clone();
        let text = self.format_tooltip(action, include_name);
        widget.set_tooltip_text(Some(&text));
        self.tooltips.borrow_mut().push((action, widget, include_name));
    }

    fn sync_translation_cache(&self, t: &Translations) {
        *self.shortcut_prefix.borrow_mut() = t.shortcut_tooltip_prefix.clone();
        let mut names = self.action_names.borrow_mut();
        for a in Action::ALL {
            names.insert(a, a.display_name(t));
        }
    }

    fn format_tooltip(&self, action: Action, include_name: bool) -> String {
        let prefix = self.shortcut_prefix.borrow();
        let accel = self.label(action);
        if include_name {
            if let Some(name) = self.action_names.borrow().get(&action) {
                return format!("{name} ({prefix}{accel})");
            }
        }
        tooltip_text(&prefix, &accel)
    }

    pub fn update_translations(&self, t: &Translations) {
        self.sync_translation_cache(t);
        for (a, widget, include_name) in self.tooltips.borrow().iter() {
            let text = self.format_tooltip(*a, *include_name);
            widget.set_tooltip_text(Some(&text));
        }
    }

    pub fn install(&self, window: &gtk4::ApplicationWindow, dispatch: Rc<dyn Fn(Action)>) {
        let controller = gtk4::ShortcutController::new();
        controller.set_scope(gtk4::ShortcutScope::Managed);
        // Capture phase: shortcuts are handled before the focused child
        // widget reacts to the same key press, so e.g. activating a tool
        // with its key never lights up focus rings on the toolbar buttons.
        controller.set_propagation_phase(gtk4::PropagationPhase::Capture);

        let win_weak = window.downgrade();
        let mut installed = self.shortcuts.borrow_mut();
        installed.clear();

        for action in Action::ALL {
            let bindings = self.bindings.clone();
            let dispatch = dispatch.clone();
            let win_weak = win_weak.clone();

            let callback = gtk4::CallbackAction::new(move |_, _| {
                // Same rule as [`ShortcutRegistry::should_defer`] uses for the mouse
                // gesture. The registry itself cannot be captured in this 'static
                // callback, so the rule runs on this controller's cloned bindings.
                let defer = bindings
                    .borrow()
                    .get(&action)
                    .map(|accel| conflicts_with_text_edit(action, accel))
                    .unwrap_or(true);
                if defer && text_input_has_focus(&win_weak) {
                    return glib::Propagation::Proceed;
                }
                dispatch(action);
                glib::Propagation::Stop
            });

            let shortcut = gtk4::Shortcut::new(build_trigger(&self.accel(action)), Some(callback));
            controller.add_shortcut(shortcut.clone());
            installed.push((action, shortcut));
        }

        window.add_controller(controller);
    }

    fn apply(&self, action: Action) {
        let accel = self.accel(action);
        let label = accel_label(&accel);

        for (a, shortcut) in self.shortcuts.borrow().iter() {
            if *a == action {
                shortcut.set_trigger(build_trigger(&accel));
            }
        }
        for (a, hint) in self.hint_labels.borrow().iter() {
            if *a == action {
                hint.set_text(&label);
            }
        }
        for (a, widget, include_name) in self.tooltips.borrow().iter() {
            if *a == action {
                let text = self.format_tooltip(*a, *include_name);
                widget.set_tooltip_text(Some(&text));
            }
        }
    }
}

fn tooltip_text(prefix: &str, label: &str) -> String {
    format!("{prefix}{label}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // One thread due to gtk.
    #[test]
    fn registry_behaviour() {
        if gtk4::init().is_err() {
            eprintln!("no display; skipping shortcut registry test");
            return;
        }

        // Defaults: every action bound, all distinct.
        let reg = ShortcutRegistry::new(&BTreeMap::new());
        let mut all: Vec<String> = Action::ALL.iter().map(|a| reg.accel(*a)).collect();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), Action::ALL.len());
        assert!(reg.overrides().is_empty());
        assert_eq!(reg.label(Action::Save), "Ctrl+S");
        let save_as = reg.label(Action::SaveAs); // GTK decides modifier order/wording
        assert!(save_as.contains("Shift") && save_as.contains("Ctrl") && save_as.ends_with('S'));
        assert_eq!(reg.label(Action::ToolEdit), "E");

        // Letter case and spelling are normalised.
        assert_eq!(normalize("<Control>O"), normalize("<ctrl>o"));

        // Undo/redo: bound by default, need a modifier, don't clash with anything.
        assert_eq!(reg.label(Action::Undo), "Ctrl+Z");
        let redo = reg.label(Action::Redo);
        assert!(redo.contains("Ctrl") && redo.ends_with('Y'));
        assert_eq!(
            reg.try_set(Action::Undo, "z"),
            Err(AssignError::NeedsModifier)
        );
        assert_eq!(
            reg.try_set(Action::Redo, "<Control>z"),
            Err(AssignError::Conflict(Action::Undo))
        );
        assert!(Action::Undo.defers_to_text_input() && !Action::Open.defers_to_text_input());
        assert_eq!(Action::from_id("redo"), Some(Action::Redo));

        // Validation.
        assert_eq!(
            reg.try_set(Action::Open, "j"),
            Err(AssignError::NeedsModifier)
        );
        assert_eq!(
            reg.try_set(Action::Open, "<Control>s"),
            Err(AssignError::Conflict(Action::Save))
        );
        assert_eq!(
            reg.try_set(Action::ToolEdit, "s"),
            Err(AssignError::Conflict(Action::ToolSplit))
        );
        assert_eq!(
            reg.try_set(Action::Open, "not a key"),
            Err(AssignError::Invalid)
        );
        assert!(reg.try_set(Action::Open, "<Control>o").is_ok()); // same key as itself is fine

        // Rebinding, overrides, reset.
        assert!(reg.try_set(Action::Open, "<Control>j").is_ok());
        assert_eq!(reg.label(Action::Open), "Ctrl+J");
        assert_eq!(reg.overrides().len(), 1);
        assert!(!reg.is_default(Action::Open));
        assert!(reg.try_set(Action::ToolDelete, "<Control>o").is_ok()); // freed key can be reused
        assert_eq!(
            reg.reset(Action::Open),
            Err(AssignError::Conflict(Action::ToolDelete))
        );
        reg.reset_all();
        assert!(reg.overrides().is_empty());

        // A saved file that is a legitimate swap round-trips unchanged...
        let mut swap = BTreeMap::new();
        swap.insert("open".to_string(), normalize("<Control>s").unwrap());
        swap.insert("save".to_string(), normalize("<Control>o").unwrap());
        let reg = ShortcutRegistry::new(&swap);
        assert_eq!(reg.label(Action::Open), "Ctrl+S");
        assert_eq!(reg.label(Action::Save), "Ctrl+O");

        // Every default must normalize: a raw unparsable string (e.g. "]")
        // gets no keyboard trigger and can never be reset.
        for a in Action::ALL {
            assert!(
                normalize(a.default_accel()).is_some(),
                "{}",
                a.id()
            );
        }
        assert_eq!(
            normalize("bracketright"),
            Some("bracketright".to_string())
        );
        reg.reset(Action::NextVariant).unwrap();
        reg.reset(Action::PrevVariant).unwrap();

        // ...while a hand-edited clash, junk keys and unknown actions are dropped.
        let mut bad = BTreeMap::new();
        bad.insert("open".to_string(), "<Control>s".to_string()); // clashes with Save's default
        bad.insert("tool_paint".to_string(), "garbage!!".to_string());
        bad.insert("no_such_action".to_string(), "x".to_string());
        bad.insert("export".to_string(), "x".to_string()); // file action without modifier
        let reg = ShortcutRegistry::new(&bad);
        assert!(reg.overrides().is_empty());
    }

    /// Mouse-button pseudo-accelerators: parse, canonicalize and label,
    /// without needing GTK initialized.
    #[test]
    fn mouse_bindings_parse_canonicalize_and_label() {
        let ctrl = gdk::ModifierType::CONTROL_MASK;
        let none = gdk::ModifierType::empty();

        // Bare and modifier forms parse, with the canonical spelling idempotent.
        assert_eq!(
            parse_mouse_accel("mouse4"),
            Some(MouseBinding { button: 4, mods: none })
        );
        for spelling in ["<Control>mouse4", "<ctrl>mouse4", "<Ctrl><Control>mouse4"] {
            assert_eq!(
                parse_mouse_accel(spelling),
                Some(MouseBinding { button: 4, mods: ctrl })
            );
            assert_eq!(normalize(spelling), Some("<Control>mouse4".to_string()));
        }
        // Round-trips.
        let b = parse_mouse_accel("<Control>mouse5").unwrap();
        assert_eq!(normalize("mouse5"), Some("mouse5".to_string()));
        assert_eq!(normalize(&mouse_string(b.button, b.mods)), Some("<Control>mouse5".into()));

        // Labels.
        assert_eq!(accel_label("mouse4"), "Mouse 4");
        assert_eq!(accel_label("<Control>mouse4"), "Ctrl+Mouse 4");

        // Buttons outside the usable range, wrong words and real accelerators
        // are not mouse bindings.
        assert!(parse_mouse_accel("mouse1").is_none());
        assert!(parse_mouse_accel("mouse33").is_none());
        assert!(parse_mouse_accel("mouse").is_none());
        assert!(parse_mouse_accel("m4").is_none());
        assert!(parse_mouse_accel("<Ctrl>mouse4").is_some());
        assert!(parse_mouse_accel("<Foo>mouse4").is_none());
        assert!(parse_mouse_accel("e").is_none());
        assert!(parse_mouse_accel("<Control>o").is_none());

        // Keyboard accelerators must look up by key spelling, never as buttons;
        // buttons match only via the registry's button lookup.
        assert_ne!(parse_mouse_accel("bracketright"), parse_mouse_accel("mouse4"));
    }
}
