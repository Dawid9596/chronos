use chronos::bridge::{ConservationApp, SlotEdge};
use crux_core::Core;
use gtk4::{Button, Label, ScrolledWindow};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub enum BarTool {
    Edit,
    Create,
    Delete,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DragTarget {
    None,
    Edge { slot_index: usize, edge: SlotEdge },
    Slot(usize),
}

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
    pub tool: Rc<Cell<BarTool>>,
    pub status: Label,
    pub export_btn: Button,
    pub lang_label: Label,
    pub settings_label: Label,
    pub edit_text_label: Label,
    pub create_text_label: Label,
    pub delete_text_label: Label,
    pub scrolled_window: ScrolledWindow,
}
