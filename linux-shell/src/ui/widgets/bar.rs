use crate::context::{AppContext, BarTool, DragTarget};
use chronos::bridge::{BarType, Event, SlotEdge};
use chronos::storage::{BarPattern, ConditionBar};
use gtk4::{glib, prelude::*};
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_interactive_condition_bar(
    id: u32,
    bar_type: BarType,
    condition: ConditionBar,
    width_request: i32,
    ctx: &AppContext,
) -> gtk4::DrawingArea {
    let drawing_area = gtk4::DrawingArea::builder()
        .hexpand(true)
        .width_request(width_request)
        .height_request(24)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    let draw_slots = Rc::new(RefCell::new(condition.slots.clone()));
    let draw_pattern = Rc::new(RefCell::new(condition.pattern.clone()));
    let draw_hover_slot = Rc::new(RefCell::new(None::<usize>));
    let draw_create_preview = Rc::new(RefCell::new(None::<f64>));
    let drag_target = Rc::new(RefCell::new(DragTarget::None));
    let is_ambiguous = Rc::new(RefCell::new(false));

    // --- Draw Function ---
    let draw_slots_draw = draw_slots.clone();
    let draw_pattern_draw = draw_pattern.clone();
    let draw_hover_slot_draw = draw_hover_slot.clone();
    let draw_create_preview_draw = draw_create_preview.clone();
    let ctx_draw = ctx.clone();

    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;
        if w == 0.0 || h == 0.0 {
            return;
        }

        cr.set_source_rgb(0.9, 0.9, 0.9);
        cr.rectangle(0.0, 0.0, w, h);
        cr.fill().expect("fill track");
        cr.set_source_rgb(0.7, 0.7, 0.7);
        cr.rectangle(0.0, 0.0, w, h);
        cr.stroke().expect("stroke track");

        let slots = draw_slots_draw.borrow();
        let pattern = draw_pattern_draw.borrow();
        let current_tool = ctx_draw.tool.get();

        for (idx, slot) in slots.iter().enumerate() {
            let x0 = w * (slot.start_pct as f64 / 100.0);
            let x1 = w * (slot.end_pct as f64 / 100.0);
            let seg_w = (x1 - x0).max(1.5);
            let is_zero_thin = (x1 - x0).abs() < 0.5;

            if is_zero_thin {
                cr.set_source_rgb(0.3, 0.3, 0.3);
                cr.set_line_width(2.0);
                cr.move_to(x0, h);
                cr.line_to(x0, 0.0);
                cr.stroke().expect("stroke horizontal line");
            } else {
                cr.rectangle(x0, 0.0, seg_w, h);
                cr.clip();
                cr.set_source_rgb(0.3, 0.3, 0.3);
                cr.set_line_width(1.5);
                match &*pattern {
                    BarPattern::Solid => {
                        cr.rectangle(x0, 0.0, seg_w, h);
                        cr.fill().expect("fill solid");
                    }
                    BarPattern::DiagonalHatch => {
                        let step = 8.0;
                        let mut x = x0 - h;
                        while x < x0 + seg_w {
                            cr.move_to(x, h);
                            cr.line_to(x + h, 0.0);
                            x += step;
                        }
                        cr.stroke().expect("stroke hatch");
                    }
                    BarPattern::CrossedOut => {
                        let step = 12.0;
                        let mut x = x0;
                        while x < x0 + seg_w {
                            cr.move_to(x, 0.0);
                            cr.line_to(x + step, h);
                            cr.move_to(x, h);
                            cr.line_to(x + step, 0.0);
                            x += step;
                        }
                        cr.stroke().expect("stroke cross");
                    }
                    BarPattern::Wavy => {
                        let step = 16.0;
                        let mut x = x0;
                        let mid_y = h / 2.0;
                        let amp = (h / 2.0) - 4.0;
                        cr.move_to(x0, mid_y);
                        while x < x0 + seg_w {
                            cr.line_to(x + (step * 0.25), mid_y - amp);
                            cr.line_to(x + (step * 0.75), mid_y + amp);
                            cr.line_to(x + step, mid_y);
                            x += step;
                        }
                        cr.stroke().expect("stroke wavy");
                    }
                }
                cr.reset_clip();
            }

            if current_tool == BarTool::Edit {
                if let Some(hover_idx) = *draw_hover_slot_draw.borrow() {
                    if hover_idx == idx {
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
            }
        }

        if current_tool == BarTool::Create {
            if let Some(preview_x) = *draw_create_preview_draw.borrow() {
                cr.set_source_rgba(0.2, 0.5, 0.9, 0.5);
                cr.set_line_width(2.0);
                cr.move_to(preview_x, 0.0);
                cr.line_to(preview_x, h);
                cr.stroke().expect("stroke preview");
            }
        }
    });

    // --- Motion Controller ---
    let drawing_area_motion = drawing_area.clone();
    let draw_slots_motion = draw_slots.clone();
    let draw_hover_slot_motion = draw_hover_slot.clone();
    let draw_create_preview_motion = draw_create_preview.clone();
    let ctx_motion = ctx.clone();

    let motion = gtk4::EventControllerMotion::new();
    motion.connect_motion(move |_controller, x, _y| {
        let widget = _controller.widget().unwrap();
        let w = widget.width() as f64;
        let slots = draw_slots_motion.borrow();
        let current_tool = ctx_motion.tool.get();
        let grab_zone = 10.0;

        if current_tool == BarTool::Edit {
            let mut found_hover = None;
            for (i, slot) in slots.iter().enumerate() {
                let x0 = w * (slot.start_pct as f64 / 100.0);
                let x1 = w * (slot.end_pct as f64 / 100.0);
                if (x - x0).abs() <= grab_zone || (x - x1).abs() <= grab_zone {
                    found_hover = Some(i);
                    break;
                }
            }
            let mut hover_state = draw_hover_slot_motion.borrow_mut();
            if *hover_state != found_hover {
                *hover_state = found_hover;
                drawing_area_motion.queue_draw();
            }
        } else if current_tool == BarTool::Create {
            let mut preview_state = draw_create_preview_motion.borrow_mut();
            if *preview_state != Some(x) {
                *preview_state = Some(x);
                drawing_area_motion.queue_draw();
            }
        }
    });

    let draw_hover_slot_leave = draw_hover_slot.clone();
    let draw_create_preview_leave = draw_create_preview.clone();
    let drawing_area_leave = drawing_area.clone();
    let ctx_leave = ctx.clone();

    motion.connect_leave(move |_controller| {
        let current_tool = ctx_leave.tool.get();
        if current_tool == BarTool::Edit {
            *draw_hover_slot_leave.borrow_mut() = None;
        } else if current_tool == BarTool::Create {
            *draw_create_preview_leave.borrow_mut() = None;
        }
        drawing_area_leave.queue_draw();
    });
    drawing_area.add_controller(motion);

    // --- Drag Controller ---
    let drag_target_begin = drag_target.clone();
    let is_ambiguous_begin = is_ambiguous.clone();
    let draw_slots_begin = draw_slots.clone();
    let ctx_begin = ctx.clone();

    let drag = gtk4::GestureDrag::new();
    drag.connect_drag_begin(move |gesture, start_x, _start_y| {
        let widget = gesture.widget().unwrap();
        let w = widget.width() as f64;
        let slots = draw_slots_begin.borrow();
        let grab_zone = 10.0;
        let current_tool = ctx_begin.tool.get();

        if current_tool == BarTool::Edit {
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
            *drag_target_begin.borrow_mut() = target;
            *is_ambiguous_begin.borrow_mut() = ambiguous;
        } else if current_tool == BarTool::Delete {
            let mut target = DragTarget::None;
            for (i, slot) in slots.iter().enumerate() {
                let x0 = w * (slot.start_pct as f64 / 100.0);
                let x1 = w * (slot.end_pct as f64 / 100.0);
                if start_x >= x0 - 5.0 && start_x <= x1 + 5.0 {
                    target = DragTarget::Slot(i);
                    break;
                }
            }
            *drag_target_begin.borrow_mut() = target;
        }
    });

    let drag_target_update = drag_target.clone();
    let is_ambiguous_update = is_ambiguous.clone();
    let drawing_area_update = drawing_area.clone();
    let draw_slots_update = draw_slots.clone();

    drag.connect_drag_update(move |gesture, offset_x, _offset_y| {
        let widget = gesture.widget().unwrap();
        let w = widget.width() as f64;
        let mut target = drag_target_update.borrow_mut();

        if let DragTarget::Edge { edge, slot_index } = &mut *target {
            let slot_idx = *slot_index;
            if *is_ambiguous_update.borrow() && offset_x.abs() > 3.0 {
                *edge = if offset_x > 0.0 {
                    SlotEdge::End
                } else {
                    SlotEdge::Start
                };
                *is_ambiguous_update.borrow_mut() = false;
            }
            if let Some((start_x, _)) = gesture.start_point() {
                let current_x = start_x + offset_x;
                let new_pct = (current_x / w * 100.0).clamp(0.0, 100.0) as f32;
                let mut slots = draw_slots_update.borrow_mut();
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
            drawing_area_update.queue_draw();
        }
    });

    let drag_target_end = drag_target.clone();
    let is_ambiguous_end = is_ambiguous.clone();
    let ctx_end = ctx.clone();
    let drawing_area_end = drawing_area.clone();

    drag.connect_drag_end(move |gesture, offset_x, _offset_y| {
        let widget = gesture.widget().unwrap();
        let w = widget.width() as f64;
        let current_tool = ctx_end.tool.get();
        let mut target = drag_target_end.borrow_mut();
        let is_amb = *is_ambiguous_end.borrow();

        let event = match (current_tool, *target) {
            (BarTool::Edit, DragTarget::None) => Event::CyclePattern {
                id,
                bar_type: bar_type.clone(),
            },
            (BarTool::Create, _) => {
                let click_x = gesture.start_point().map(|(x, _)| x).unwrap_or(0.0);
                Event::AddSlot {
                    id,
                    bar_type: bar_type.clone(),
                    at_pct: (click_x / w * 100.0).clamp(0.0, 100.0) as f32,
                }
            }
            (BarTool::Delete, DragTarget::Slot(slot_index)) => Event::DeleteSlot {
                id,
                bar_type: bar_type.clone(),
                slot_index,
            },
            (BarTool::Edit, DragTarget::Edge { slot_index, edge }) => {
                if is_amb && offset_x.abs() <= 3.0 {
                    Event::CyclePattern {
                        id,
                        bar_type: bar_type.clone(),
                    }
                } else {
                    let start_x = gesture.start_point().map(|(x, _)| x).unwrap_or(0.0);
                    Event::UpdateSlotEdge {
                        id,
                        bar_type: bar_type.clone(),
                        slot_index,
                        edge,
                        new_pct: ((start_x + offset_x) / w * 100.0).clamp(0.0, 100.0) as f32,
                    }
                }
            }
            _ => {
                *target = DragTarget::None;
                drawing_area_end.queue_draw();
                return;
            }
        };

        *target = DragTarget::None;
        drawing_area_end.queue_draw();

        let ctx_idle = ctx_end.clone();
        glib::idle_add_local_once(move || {
            let effects = ctx_idle.core.process_event(event);
            crate::effects::process_effects(&ctx_idle, effects);
        });
    });

    drawing_area.add_controller(drag);
    drawing_area
}
