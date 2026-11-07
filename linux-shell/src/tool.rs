use chronos::bridge::{BarType, CycleDirection, Event, SlotEdge};
use chronos::i18n::Translations;
use chronos::storage::{BarPattern, Slot};
use gtk4::cairo;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Edit,
    Create,
    Delete,
    Paint(BarPattern),
    Split,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragTarget {
    None,
    Edge { slot_index: usize, edge: SlotEdge },
    Slot(usize),
}

impl Default for DragTarget {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
pub struct BarInteractionState {
    pub hover_slot: Option<usize>,
    pub create_preview: Option<f64>,
    pub drag_target: DragTarget,
    pub is_ambiguous: bool,
    pub paint_hover: bool,
}

impl Tool {
    pub const TOOLBAR_DEFAULTS: [Tool; 5] = [
        Tool::Edit,
        Tool::Create,
        Tool::Delete,
        Tool::Paint(BarPattern::VerticalHatch),
        Tool::Split,
    ];

    pub fn icon_name(self) -> &'static str {
        match self {
            Tool::Edit => "document-edit-symbolic",
            Tool::Create => "list-add-symbolic",
            Tool::Delete => "user-trash-symbolic",
            Tool::Paint(_) => "applications-graphics-symbolic",
            Tool::Split => "edit-cut-symbolic",
        }
    }

    pub fn label(self, t: &Translations) -> &str {
        match self {
            Tool::Edit => &t.tool_edit,
            Tool::Create => &t.tool_create,
            Tool::Delete => &t.tool_delete,
            Tool::Paint(_) => &t.tool_paint,
            Tool::Split => &t.tool_split,
        }
    }

    // --- Rendering Overlays ---

    pub fn draw_slot_overlay(
        self,
        cr: &cairo::Context,
        state: &BarInteractionState,
        slot_idx: usize,
        x0: f64,
        x1: f64,
        h: f64,
    ) {
        if self == Tool::Edit && state.hover_slot == Some(slot_idx) {
            let handle_w = 6.0;
            let handle_h = h + 8.0;
            let handle_y = -4.0;
            cr.set_source_rgb(0.2, 0.5, 0.9);
            cr.rectangle(x0 - (handle_w / 2.0), handle_y, handle_w, handle_h);
            cr.fill().expect("fill left handle");
            cr.rectangle(x1 - (handle_w / 2.0), handle_y, handle_w, handle_h);
            cr.fill().expect("fill right handle");
        }
    }

    pub fn draw_bar_overlay(
        self,
        cr: &cairo::Context,
        state: &BarInteractionState,
        w: f64,
        h: f64,
    ) {
        match self {
            Tool::Create => {
                if let Some(preview_x) = state.create_preview {
                    cr.set_source_rgba(0.2, 0.5, 0.9, 0.5);
                    cr.set_line_width(2.0);
                    cr.move_to(preview_x, 0.0);
                    cr.line_to(preview_x, h);
                    cr.stroke().expect("stroke preview");
                }
            }
            Tool::Paint(_) => {
                if state.paint_hover {
                    cr.set_source_rgba(0.2, 0.5, 0.9, 0.8);
                    cr.set_line_width(2.0);
                    cr.rectangle(1.0, 1.0, (w - 2.0).max(0.0), (h - 2.0).max(0.0));
                    cr.stroke().expect("stroke paint hover border");
                }
            }
            _ => {}
        }
    }

    // --- Pointer / Gesture Handlers ---

    pub fn on_motion(
        self,
        state: &mut BarInteractionState,
        slots: &[Slot],
        w: f64,
        x: f64,
        _y: f64,
    ) -> bool {
        let grab_zone = 10.0;
        match self {
            Tool::Edit => {
                let mut found_hover = None;
                for (i, slot) in slots.iter().enumerate() {
                    let x0 = w * (slot.start_pct as f64 / 100.0);
                    let x1 = w * (slot.end_pct as f64 / 100.0);
                    if (x - x0).abs() <= grab_zone || (x - x1).abs() <= grab_zone {
                        found_hover = Some(i);
                        break;
                    }
                }
                if state.hover_slot != found_hover {
                    state.hover_slot = found_hover;
                    return true;
                }
            }
            Tool::Create => {
                if state.create_preview != Some(x) {
                    state.create_preview = Some(x);
                    return true;
                }
            }
            Tool::Paint(_) => {
                if !state.paint_hover {
                    state.paint_hover = true;
                    return true;
                }
            }
            Tool::Delete | Tool::Split => {}
        }
        false
    }

    pub fn on_leave(self, state: &mut BarInteractionState) {
        match self {
            Tool::Edit => state.hover_slot = None,
            Tool::Create => state.create_preview = None,
            Tool::Paint(_) => state.paint_hover = false,
            Tool::Delete | Tool::Split => {}
        }
    }

    pub fn on_drag_begin(
        self,
        state: &mut BarInteractionState,
        slots: &[Slot],
        w: f64,
        start_x: f64,
    ) {
        let grab_zone = 10.0;
        match self {
            Tool::Edit => {
                let mut target = DragTarget::None;
                let mut ambiguous = false;
                for (i, slot) in slots.iter().enumerate() {
                    let x0 = w * (slot.start_pct as f64 / 100.0);
                    let x1 = w * (slot.end_pct as f64 / 100.0);
                    if (start_x - x0).abs() <= grab_zone && (start_x - x1).abs() <= grab_zone {
                        target = DragTarget::Edge {
                            slot_index: i,
                            edge: SlotEdge::Start,
                        };
                        ambiguous = true;
                        break;
                    }
                    if (start_x - x0).abs() <= grab_zone {
                        target = DragTarget::Edge {
                            slot_index: i,
                            edge: SlotEdge::Start,
                        };
                        break;
                    }
                    if (start_x - x1).abs() <= grab_zone {
                        target = DragTarget::Edge {
                            slot_index: i,
                            edge: SlotEdge::End,
                        };
                        break;
                    }
                }
                state.drag_target = target;
                state.is_ambiguous = ambiguous;
            }
            Tool::Delete => {
                let mut target = DragTarget::None;
                for (i, slot) in slots.iter().enumerate() {
                    let x0 = w * (slot.start_pct as f64 / 100.0);
                    let x1 = w * (slot.end_pct as f64 / 100.0);
                    if start_x >= x0 - 5.0 && start_x <= x1 + 5.0 {
                        target = DragTarget::Slot(i);
                        break;
                    }
                }
                state.drag_target = target;
            }
            Tool::Create | Tool::Paint(_) | Tool::Split => {
                state.drag_target = DragTarget::None;
            }
        }
    }

    pub fn on_drag_update(
        self,
        state: &mut BarInteractionState,
        slots: &mut [Slot],
        w: f64,
        start_x: Option<f64>,
        offset_x: f64,
    ) -> bool {
        if self != Tool::Edit {
            return false;
        }
        if let DragTarget::Edge { edge, slot_index } = &mut state.drag_target {
            let slot_idx = *slot_index;
            if state.is_ambiguous && offset_x.abs() > 3.0 {
                *edge = if offset_x > 0.0 {
                    SlotEdge::End
                } else {
                    SlotEdge::Start
                };
                state.is_ambiguous = false;
            }
            if let Some(sx) = start_x {
                let current_x = sx + offset_x;
                let new_pct = (current_x / w * 100.0).clamp(0.0, 100.0) as f32;
                let min_start = if slot_idx > 0 {
                    slots[slot_idx - 1].end_pct + 2.0
                } else {
                    0.0
                };
                let max_end = if slot_idx + 1 < slots.len() {
                    slots[slot_idx + 1].start_pct - 2.0
                } else {
                    100.0
                };
                if let Some(slot) = slots.get_mut(slot_idx) {
                    match edge {
                        SlotEdge::Start => slot.start_pct = new_pct.clamp(min_start, slot.end_pct),
                        SlotEdge::End => slot.end_pct = new_pct.clamp(slot.start_pct, max_end),
                    }
                }
            }
            return true;
        }
        false
    }

    pub fn on_drag_end(
        self,
        state: &mut BarInteractionState,
        id: u32,
        bar_type: BarType,
        w: f64,
        start_x: f64,
        offset_x: f64,
    ) -> Option<Event> {
        let target = state.drag_target;
        let is_amb = state.is_ambiguous;
        state.drag_target = DragTarget::None;
        state.is_ambiguous = false;

        match (self, target) {
            (Tool::Edit, DragTarget::None) => Some(Event::CyclePattern {
                id,
                bar_type,
                direction: CycleDirection::Forward,
            }),
            (Tool::Create, _) => Some(Event::AddSlot {
                id,
                bar_type,
                at_pct: (start_x / w * 100.0).clamp(0.0, 100.0) as f32,
            }),
            (Tool::Delete, DragTarget::Slot(slot_index)) => Some(Event::DeleteSlot {
                id,
                bar_type,
                slot_index,
            }),
            (Tool::Paint(pattern), _) => Some(Event::SetPattern {
                id,
                bar_type,
                pattern,
            }),
            (Tool::Edit, DragTarget::Edge { slot_index, edge }) => {
                if is_amb && offset_x.abs() <= 3.0 {
                    Some(Event::CyclePattern {
                        id,
                        bar_type,
                        direction: CycleDirection::Forward,
                    })
                } else {
                    Some(Event::UpdateSlotEdge {
                        id,
                        bar_type,
                        slot_index,
                        edge,
                        new_pct: ((start_x + offset_x) / w * 100.0).clamp(0.0, 100.0) as f32,
                    })
                }
            }
            _ => None,
        }
    }

    pub fn on_secondary_click(self, id: u32, bar_type: BarType) -> Option<Event> {
        match self {
            Tool::Edit => Some(Event::CyclePattern {
                id,
                bar_type,
                direction: CycleDirection::Backward,
            }),
            Tool::Create | Tool::Delete | Tool::Paint(_) | Tool::Split => None,
        }
    }

    pub fn on_group_split_click(
        self,
        tail_member_ids: Vec<u32>,
        field: chronos::bridge::EditableField,
    ) -> Option<Event> {
        if self == Tool::Split && !tail_member_ids.is_empty() {
            Some(Event::SplitGroupCell {
                tail_member_ids,
                field,
            })
        } else {
            None
        }
    }
}
