use crate::context::AppContext;
use chronos::bridge::{BarType, EditableField, Event};
use chronos::view::AppViewModel;
use gtk4::glib;
use gtk4::prelude::*;

fn build_group_cell_widget(
    group: &chronos::view::GroupSpan,
    field: EditableField,
    start_row_idx: usize,
    doc_rows: &[chronos::view::RowViewModel],
    ctx: &AppContext,
) -> (gtk4::Widget, i32) {
    let mut gtk_span = group.span;
    for k in start_row_idx..(start_row_idx + group.span - 1) {
        if doc_rows[k].is_last_in_group {
            gtk_span += 1;
        }
    }

    let base_widget = crate::ui::widgets::editable::create_editable_widget(&group.value, ctx, {
        let member_ids = group.member_ids.clone();
        let field = field.clone();
        move |new_val| Event::UpdateGroupText {
            member_ids: member_ids.clone(),
            field: field.clone(),
            new_value: new_val,
        }
    });

    // Trigger Merge on single click when Tool::Merge(_) is active
    let merge_click = gtk4::GestureClick::new();
    merge_click.set_button(1);
    let member_ids_merge = group.member_ids.clone();
    let field_merge = field.clone();
    let ctx_merge = ctx.clone();
    merge_click.connect_pressed(move |gesture, n_press, _x, _y| {
        if n_press == 1 {
            if let Some(ev) = ctx_merge
                .tool
                .get()
                .on_group_merge_click(member_ids_merge.clone(), field_merge.clone())
            {
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                let ctx_idle = ctx_merge.clone();
                glib::idle_add_local_once(move || {
                    let effects = ctx_idle.core.process_event(ev);
                    crate::effects::process_effects(&ctx_idle, effects);
                });
            }
        }
    });
    base_widget.add_controller(merge_click);

    if group.span <= 1 {
        return (base_widget, gtk_span as i32);
    }

    base_widget.add_css_class("merged-cell");

    let overlay = gtk4::Overlay::new();
    overlay.set_child(Some(&base_widget));

    // Build a vertical box with homogeneous row slots so each boundary aligns
    // between row `offset - 1` and row `offset` inside the merged cell.
    let split_vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .homogeneous(true)
        .hexpand(true)
        .vexpand(true)
        .can_target(false)
        .visible(ctx.tool.get() == crate::tool::Tool::Split)
        .build();

    for offset in 1..group.span {
        let slot_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .valign(gtk4::Align::End)
            .hexpand(true)
            .vexpand(true)
            .build();

        let split_btn = gtk4::Button::builder()
            .has_frame(false)
            .hexpand(true)
            .focusable(false)
            .can_focus(false)
            .build();
        split_btn.add_css_class("split-gap");

        let tail_ids = group.member_ids[offset..].to_vec();
        let field_clone = field.clone();
        let ctx_btn = ctx.clone();

        split_btn.connect_clicked(move |_| {
            if let Some(ev) = ctx_btn
                .tool
                .get()
                .on_group_split_click(tail_ids.clone(), field_clone.clone())
            {
                let ctx_idle = ctx_btn.clone();
                glib::idle_add_local_once(move || {
                    let effects = ctx_idle.core.process_event(ev);
                    crate::effects::process_effects(&ctx_idle, effects);
                });
            }
        });

        slot_box.append(&split_btn);
        split_vbox.append(&slot_box);
    }

    // Trailing spacer for the bottom row of the merged group
    let bottom_spacer = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .can_target(false)
        .build();
    split_vbox.append(&bottom_spacer);

    overlay.add_overlay(&split_vbox);
    overlay.set_measure_overlay(&split_vbox, false);

    // Enable hit-testing on the split overlay only when `Tool::Split` is active
    let split_vbox_motion = split_vbox.clone();
    let ctx_motion = ctx.clone();
    let motion = gtk4::EventControllerMotion::new();
    motion.connect_enter(move |_, _, _| {
        let is_split = ctx_motion.tool.get() == crate::tool::Tool::Split;
        split_vbox_motion.set_can_target(is_split);
        split_vbox_motion.set_visible(is_split);
    });
    overlay.add_controller(motion);

    (overlay.upcast(), gtk_span as i32)
}

pub fn build_document_table(app_view: &AppViewModel, ctx: &AppContext) -> gtk4::Grid {
    let grid = gtk4::Grid::builder()
        .column_spacing(16)
        .row_spacing(0)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .hexpand(true)
        .vexpand(true)
        .build();

    let current_vis = ctx.vis.borrow();
    let t = &app_view.translations;
    let headers = [
        &t.col_no,
        &t.col_graphical,
        &t.col_woven,
        &t.col_char,
        &t.col_thickness,
        &t.col_chrono,
        &t.col_dating,
        &String::new(),
    ];

    let mut total_visible_cols = 0;
    let mut col_idx = 0;
    for (i, title) in headers.iter().enumerate() {
        let is_visible = if i == 7 { true } else { current_vis.flags[i] };
        if is_visible {
            let label = gtk4::Label::builder()
                .use_markup(true)
                .label(&format!("<b>{}</b>", title))
                .build();
            grid.attach(&label, col_idx, 0, 1, 1);
            col_idx += 1;
            total_visible_cols += 1;
        }
    }

    let mut current_grid_row = 1;
    grid.attach(
        &build_insert_row(None, ctx),
        0,
        current_grid_row,
        total_visible_cols,
        1,
    );
    current_grid_row += 1;

    let doc_rows = &app_view.document.rows;
    for (i, row) in doc_rows.iter().enumerate() {
        let grid_row = current_grid_row;
        let mut col_idx = 0;

        if current_vis.flags[0] {
            let grab_cursor = gtk4::gdk::Cursor::from_name("grab", None);
            let grabbing_cursor = gtk4::gdk::Cursor::from_name("grabbing", None);

            let handle_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .halign(gtk4::Align::Center)
                .valign(gtk4::Align::Center)
                .margin_top(4)
                .margin_bottom(4)
                .tooltip_text("Double-click and drag to reorder")
                .build();

            handle_box.set_cursor(grab_cursor.as_ref());

            let row_num_label = gtk4::Label::new(Some(&row.row_number.to_string()));
            handle_box.append(&row_num_label);

            let is_armed = std::rc::Rc::new(std::cell::Cell::new(false));

            let click_gesture = gtk4::GestureClick::new();
            click_gesture.set_button(1);
            let is_armed_click = is_armed.clone();
            let handle_box_click = handle_box.clone();
            let row_id = row.id;
            let ctx_click = ctx.clone();
            click_gesture.connect_pressed(move |_gesture, n_press, _x, _y| {
                if let Some(ev) = ctx_click.tool.get().on_row_click(row_id) {
                    if n_press == 1 {
                        let ctx_idle = ctx_click.clone();
                        glib::idle_add_local_once(move || {
                            let effects = ctx_idle.core.process_event(ev);
                            crate::effects::process_effects(&ctx_idle, effects);
                        });
                    }
                    return;
                }
                if n_press == 2 {
                    is_armed_click.set(true);
                    handle_box_click.set_cursor(grabbing_cursor.as_ref());
                    handle_box_click.set_opacity(0.6);
                }
            });
            handle_box.add_controller(click_gesture);

            let drag_gesture = gtk4::GestureDrag::new();
            let is_armed_drag = is_armed.clone();
            let handle_box_drag = handle_box.clone();
            let row_id = row.id;
            let current_idx = i;
            let total_rows = doc_rows.len();
            let ctx_drag = ctx.clone();

            drag_gesture.connect_drag_end(move |_gesture, _offset_x, offset_y| {
                if !is_armed_drag.get() {
                    return;
                }
                is_armed_drag.set(false);
                handle_box_drag.set_cursor(grab_cursor.as_ref());
                handle_box_drag.set_opacity(1.0);

                let estimated_row_height = 36.0;
                let delta = (offset_y / estimated_row_height).round() as isize;
                if delta == 0 || total_rows <= 1 {
                    return;
                }

                let target_idx =
                    (current_idx as isize + delta).clamp(0, (total_rows as isize) - 1) as usize;
                if target_idx == current_idx {
                    return;
                }

                let ctx_idle = ctx_drag.clone();
                glib::idle_add_local_once(move || {
                    let effects = ctx_idle.core.process_event(Event::MoveLayer {
                        id: row_id,
                        target_index: target_idx,
                    });
                    crate::effects::process_effects(&ctx_idle, effects);
                });
            });
            handle_box.add_controller(drag_gesture);

            grid.attach(&handle_box, col_idx, grid_row, 1, 1);
            col_idx += 1;
        }

        if current_vis.flags[1] {
            let graph_bar = crate::ui::widgets::bar::build_interactive_condition_bar(
                row.id,
                BarType::GraphicalRepresentation,
                row.graphical_representation.clone(),
                150,
                ctx,
            );
            grid.attach(&graph_bar, col_idx, grid_row, 1, 1);
            col_idx += 1;
        }

        if current_vis.flags[2] {
            let woven_bar = crate::ui::widgets::bar::build_interactive_condition_bar(
                row.id,
                BarType::WovenTrim,
                row.woven_trim.clone(),
                60,
                ctx,
            );
            grid.attach(&woven_bar, col_idx, grid_row, 1, 1);
            col_idx += 1;
        }

        if current_vis.flags[3] {
            let char_widget = crate::ui::widgets::editable::create_editable_widget(
                &row.layer_characteristics,
                ctx,
                {
                    let id = row.id;
                    move |new_val| Event::UpdateText {
                        id,
                        field: EditableField::LayerCharacteristics,
                        new_value: new_val,
                    }
                },
            );
            grid.attach(&char_widget, col_idx, grid_row, 1, 1);
            col_idx += 1;
        }

        if current_vis.flags[4] {
            let thick_widget =
                crate::ui::widgets::editable::create_editable_widget(&row.layer_thickness, ctx, {
                    let id = row.id;
                    move |new_val| Event::UpdateText {
                        id,
                        field: EditableField::LayerThickness,
                        new_value: new_val,
                    }
                });
            grid.attach(&thick_widget, col_idx, grid_row, 1, 1);
            col_idx += 1;
        }

        if current_vis.flags[5] {
            if let Some(chrono) = &row.chronological_phase {
                let (widget, gtk_span) = build_group_cell_widget(
                    chrono,
                    EditableField::ChronologicalPhase,
                    i,
                    doc_rows,
                    ctx,
                );
                grid.attach(&widget, col_idx, grid_row, 1, gtk_span);
            }
            col_idx += 1;
        }

        if current_vis.flags[6] {
            if let Some(dating) = &row.dating_period {
                let (widget, gtk_span) = build_group_cell_widget(
                    dating,
                    EditableField::DatingPeriod,
                    i,
                    doc_rows,
                    ctx,
                );
                grid.attach(&widget, col_idx, grid_row, 1, gtk_span);
            }
            col_idx += 1;
        }

        let delete_btn = gtk4::Button::builder()
            .icon_name("user-trash-symbolic")
            .has_frame(false)
            .focusable(false)
            .can_focus(false)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .margin_top(4)
            .margin_bottom(4)
            .tooltip_text(&t.delete_label)
            .build();
        delete_btn.add_css_class("flat");
        delete_btn.add_css_class("destructive-action");

        let row_id = row.id;
        let ctx_btn = ctx.clone();

        delete_btn.connect_clicked(move |_| {
            let ctx_idle = ctx_btn.clone();
            glib::idle_add_local_once(move || {
                let effects = ctx_idle.core.process_event(Event::DeleteLayer(row_id));
                crate::effects::process_effects(&ctx_idle, effects);
            });
        });

        grid.attach(&delete_btn, col_idx, grid_row, 1, 1);
        current_grid_row += 1;

        if row.is_last_in_group {
            grid.attach(
                &build_insert_row(Some(row.id), ctx),
                0,
                current_grid_row,
                total_visible_cols,
                1,
            );
            current_grid_row += 1;
        }
    }

    grid
}

pub fn build_insert_row(after_id: Option<u32>, ctx: &AppContext) -> gtk4::Widget {
    let btn = gtk4::Button::builder()
        .has_frame(false)
        .hexpand(true)
        .focusable(false)
        .can_focus(false)
        .margin_top(1)
        .margin_bottom(1)
        .build();
    btn.add_css_class("insert-gap");

    let ctx_btn = ctx.clone();

    btn.connect_clicked(move |_| {
        let ctx_idle = ctx_btn.clone();
        glib::idle_add_local_once(move || {
            let effects = ctx_idle
                .core
                .process_event(Event::InsertLayerAfter(after_id));
            crate::effects::process_effects(&ctx_idle, effects);
        });
    });

    btn.upcast()
}
