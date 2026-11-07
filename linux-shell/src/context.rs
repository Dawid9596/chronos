use crate::tool::Tool;
use chronos::bridge::ConservationApp;
use crux_core::Core;
use gtk4::glib;
use gtk4::{Button, Label, ScrolledWindow};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone)]
pub struct ColumnVisibility {
    pub flags: [bool; 7],
}

impl Default for ColumnVisibility {
    fn default() -> Self {
        Self { flags: [true; 7] }
    }
}

#[derive(Clone)]
pub struct AppContext {
    pub core: Rc<Core<ConservationApp>>,
    pub vis: Rc<RefCell<ColumnVisibility>>,
    pub tool: Rc<Cell<Tool>>,
    pub status: Label,
    pub export_btn: Button,
    pub lang_label: Label,
    pub settings_label: Label,
    pub tool_labels: Rc<Vec<(Tool, Label)>>,
    pub scrolled_window: ScrolledWindow,
    /// Tracks the last displayed message to avoid re-triggering identical renders.
    pub last_message: Rc<RefCell<Option<String>>>,
    /// Pending fade-out timer; canceled whenever a new message appears.
    pub status_timer: Rc<RefCell<Option<glib::SourceId>>>,
}
