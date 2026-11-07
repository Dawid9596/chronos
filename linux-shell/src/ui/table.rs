use crate::context::AppContext;
use crate::tool::Tool;
use crate::ui::widgets::bar::ConditionBarHandle;
use crate::ui::widgets::editable::EditableCell;
use chronos::bridge::{BarType, EditableField, Event};
use chronos::i18n::Language;
use chronos::view::AppViewModel;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Structure signature + live widget handles
//
// The grid only has to be rebuilt when its *shape* changes.
// Otherwise it is pushed into the existing widgets in place.
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
struct GroupSig {
    member_ids: Vec<u32>,
    span: usize,
}

#[derive(Clone, PartialEq)]
struct RowSig {
    id: u32,
    row_number: String,
    is_last_in_group: bool,
    chrono: Option<GroupSig>,
    dating: Option<GroupSig>,
}

#[derive(Clone, PartialEq)]
struct TableSignature {
    language: Language,
    visible: [bool; 7],
    rows: Vec<RowSig>,
}

impl TableSignature {
    fn from_view(app_view: &AppViewModel, visible: [bool; 7]) -> Self {
        let group = |g: &chronos::view::GroupSpan| GroupSig {
            member_ids: g.member_ids.clone(),
            span: g.span,
        };
        Self {
            language: app_view.translations.language,
            visible,
            rows: app_view
                .document
                .rows
                .iter()
                .map(|row| RowSig {
                    id: row.id,
                    row_number: row.row_number.to_string(),
                    is_last_in_group: row.is_last_in_group,
                    chrono: row.chronological_phase.as_ref().map(group),
                    dating: row.dating_period.as_ref().map(group),
                })
                .collect(),
        }
    }
}

struct RowHandles {
    graph: Option<ConditionBarHandle>,
    woven: Option<ConditionBarHandle>,
    characteristics: Option<EditableCell>,
    thickness: Option<EditableCell>,
    chrono: Option<EditableCell>,
    dating: Option<EditableCell>,
}

pub struct DocumentTable {
    signature: TableSignature,
    grid: gtk4::Grid,
    rows: Vec<RowHandles>,
}

impl DocumentTable {
    fn update_in_place(&self, app_view: &AppViewModel) {
        for (handles, row) in self.rows.iter().zip(&app_view.document.rows) {
            if let Some(h) = &handles.graph {
                h.sync(&row.graphical_representation);
            }
            if let Some(h) = &handles.woven {
                h.sync(&row.woven_trim);
            }
            if let Some(h) = &handles.characteristics {
                h.set_text(&row.layer_characteristics);
            }
            if let Some(h) = &handles.thickness {
                h.set_text(&row.layer_thickness);
            }
            if let (Some(h), Some(g)) = (&handles.chrono, &row.chronological_phase) {
                h.set_text(&g.value);
            }
            if let (Some(h), Some(g)) = (&handles.dating, &row.dating_period) {
                h.set_text(&g.value);
            }
        }
    }
}

/// Weak references to the split-line overlays of every merged cell of
/// the grid.
pub type SplitOverlays = Rc<RefCell<Vec<glib::WeakRef<gtk4::Box>>>>;

pub fn update_split_overlays(overlays: &SplitOverlays, tool: Tool) {
    let active = tool == Tool::Split;
    overlays.borrow_mut().retain(|weak| match weak.upgrade() {
        Some(overlay) => {
            overlay.set_visible(active);
            overlay.set_can_target(active);
            true
        }
        None => false,
    });
}

/// Single entry point for showing the document in `ctx.scrolled_window`.
pub fn sync_document_table(app_view: &AppViewModel, ctx: &AppContext) {
    let visible = ctx.vis.borrow().flags;
    let signature = TableSignature::from_view(app_view, visible);

    if let Some(table) = ctx.table.borrow().as_ref()
        && table.signature == signature
    {
        table.update_in_place(app_view);
        return;
    }

    let vadj = ctx.scrolled_window.vadjustment();
    let saved_y = vadj.value();

    ctx.split_overlays.borrow_mut().clear();
    let table = build_document_table(app_view, ctx, signature);
    ctx.scrolled_window.set_child(Some(&table.grid));
    *ctx.table.borrow_mut() = Some(table);

    glib::idle_add_local_once(move || {
        vadj.set_value(saved_y);
    });
}

// ---------------------------------------------------------------------------
// Row geometry for drag-and-drop reordering
// ---------------------------------------------------------------------------

type RowAnchors = Rc<RefCell<Vec<Vec<glib::WeakRef<gtk4::Widget>>>>>;

fn weak_widget<W: IsA<gtk4::Widget>>(w: &W) -> glib::WeakRef<gtk4::Widget> {
    w.upcast_ref::<gtk4::Widget>().downgrade()
}

fn row_extents(
    grid: &gtk4::Grid,
    anchors: &[Vec<glib::WeakRef<gtk4::Widget>>],
) -> Vec<(usize, f64, f64)> {
    anchors
        .iter()
        .enumerate()
        .filter_map(|(idx, widgets)| {
            let mut top = f64::INFINITY;
            let mut bottom = f64::NEG_INFINITY;
            for widget in widgets.iter().filter_map(|w| w.upgrade()) {
                if let Some(rect) = widget.compute_bounds(grid) {
                    top = top.min(rect.y() as f64);
                    bottom = bottom.max((rect.y() + rect.height()) as f64);
                }
            }
            (top.is_finite() && bottom.is_finite()).then_some((idx, top, bottom))
        })
        .collect()
}

/// Maps a Y coordinate to the row it falls on. Rows own the space up to the
/// midpoint of the gap that separates them from the next row, so dropping on
/// an insert-gap strip or a row margin still resolves to the nearest row.
/// `extents` must be in top-to-bottom order.
fn row_at_y(extents: &[(usize, f64, f64)], y: f64) -> Option<usize> {
    for pair in extents.windows(2) {
        let (idx, _, bottom) = pair[0];
        let (_, next_top, _) = pair[1];
        if y < (bottom + next_top) / 2.0 {
            return Some(idx);
        }
    }
    extents.last().map(|&(idx, _, _)| idx)
}

fn build_group_cell_widget(
    group: &chronos::view::GroupSpan,
    field: EditableField,
    start_row_idx: usize,
    doc_rows: &[chronos::view::RowViewModel],
    ctx: &AppContext,
) -> (gtk4::Widget, i32, EditableCell) {
    let mut gtk_span = group.span;
    for row in doc_rows
        .iter()
        .take(start_row_idx + group.span - 1)
        .skip(start_row_idx)
    {
        if row.is_last_in_group {
            gtk_span += 1;
        }
    }

    let cell = crate::ui::widgets::editable::create_editable_cell(&group.value, ctx, {
        let member_ids = group.member_ids.clone();
        let field = field.clone();
        move |new_val| Event::UpdateGroupText {
            member_ids: member_ids.clone(),
            field: field.clone(),
            new_value: new_val,
        }
    });
    let base_widget = cell.widget();

    // Trigger Merge on single click when Tool::Merge(_) is active
    let merge_click = gtk4::GestureClick::new();
    merge_click.set_button(1);
    let member_ids_merge = group.member_ids.clone();
    let field_merge = field.clone();
    let ctx_merge = ctx.clone();
    merge_click.connect_pressed(move |gesture, n_press, _x, _y| {
        if n_press == 1
            && let Some(ev) = ctx_merge
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
    });
    base_widget.add_controller(merge_click);

    if group.span <= 1 {
        return (base_widget, gtk_span as i32, cell);
    }

    base_widget.add_css_class("merged-cell");

    let overlay = gtk4::Overlay::new();
    overlay.set_child(Some(&base_widget));

    let is_split_tool = ctx.tool.get() == Tool::Split;

    // Build a vertical box with homogeneous row slots so each boundary aligns
    // between row `offset - 1` and row `offset` inside the merged cell.
    let split_vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .homogeneous(true)
        .hexpand(true)
        .vexpand(true)
        .can_target(is_split_tool)
        .visible(is_split_tool)
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

    ctx.split_overlays.borrow_mut().push(split_vbox.downgrade());

    (overlay.upcast(), gtk_span as i32, cell)
}

fn build_document_table(
    app_view: &AppViewModel,
    ctx: &AppContext,
    signature: TableSignature,
) -> DocumentTable {
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
                .label(format!("<b>{}</b>", title))
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
    let row_anchors: RowAnchors = Rc::new(RefCell::new(Vec::with_capacity(doc_rows.len())));
    let mut row_handles: Vec<RowHandles> = Vec::with_capacity(doc_rows.len());
    for (i, row) in doc_rows.iter().enumerate() {
        let grid_row = current_grid_row;
        let mut col_idx = 0;
        let mut anchors: Vec<glib::WeakRef<gtk4::Widget>> = Vec::new();
        let mut handles = RowHandles {
            graph: None,
            woven: None,
            characteristics: None,
            thickness: None,
            chrono: None,
            dating: None,
        };

        if current_vis.flags[0] {
            let fallback_cursor = gtk4::gdk::Cursor::from_name("move", None);
            let grab_cursor = if cfg!(windows) {
                fallback_cursor.clone()
            } else {
                gtk4::gdk::Cursor::from_name("grab", fallback_cursor.as_ref())
            };
            let grabbing_cursor = if cfg!(windows) {
                fallback_cursor
            } else {
                gtk4::gdk::Cursor::from_name("grabbing", fallback_cursor.as_ref())
            };

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
            let ctx_drag = ctx.clone();
            let grid_weak = grid.downgrade();
            let anchors_drag = row_anchors.clone();

            drag_gesture.connect_drag_end(move |gesture, offset_x, offset_y| {
                if !is_armed_drag.get() {
                    return;
                }
                is_armed_drag.set(false);
                handle_box_drag.set_cursor(grab_cursor.as_ref());
                handle_box_drag.set_opacity(1.0);

                let Some((start_x, start_y)) = gesture.start_point() else {
                    return;
                };
                let Some(grid) = grid_weak.upgrade() else {
                    return;
                };
                let release = gtk4::graphene::Point::new(
                    (start_x + offset_x) as f32,
                    (start_y + offset_y) as f32,
                );
                let Some(in_grid) = handle_box_drag.compute_point(&grid, &release) else {
                    return;
                };

                let extents = row_extents(&grid, &anchors_drag.borrow());
                let Some(target_idx) = row_at_y(&extents, in_grid.y() as f64) else {
                    return;
                };
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
            anchors.push(weak_widget(&handle_box));
            col_idx += 1;
        }

        let graph_w = 150;
        let woven_w = 60;
        if current_vis.flags[1] {
            let graph = crate::ui::widgets::bar::build_interactive_condition_bar(
                row.id,
                BarType::GraphicalRepresentation,
                row.graphical_representation.clone(),
                graph_w,
                ctx,
            );
            let graph_bar = graph.widget();
            grid.attach(&graph_bar, col_idx, grid_row, 1, 1);
            anchors.push(weak_widget(&graph_bar));
            handles.graph = Some(graph);
            col_idx += 1;
        }
        if current_vis.flags[2] {
            let woven = crate::ui::widgets::bar::build_interactive_condition_bar(
                row.id,
                BarType::WovenTrim,
                row.woven_trim.clone(),
                woven_w,
                ctx,
            );
            let woven_bar = woven.widget();
            grid.attach(&woven_bar, col_idx, grid_row, 1, 1);
            anchors.push(weak_widget(&woven_bar));
            handles.woven = Some(woven);
            col_idx += 1;
        }

        if current_vis.flags[3] {
            let char_cell = crate::ui::widgets::editable::create_editable_cell(
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
            let char_widget = char_cell.widget();
            grid.attach(&char_widget, col_idx, grid_row, 1, 1);
            anchors.push(weak_widget(&char_widget));
            handles.characteristics = Some(char_cell);
            col_idx += 1;
        }

        if current_vis.flags[4] {
            let thick_cell =
                crate::ui::widgets::editable::create_editable_cell(&row.layer_thickness, ctx, {
                    let id = row.id;
                    move |new_val| Event::UpdateText {
                        id,
                        field: EditableField::LayerThickness,
                        new_value: new_val,
                    }
                });
            let thick_widget = thick_cell.widget();
            grid.attach(&thick_widget, col_idx, grid_row, 1, 1);
            anchors.push(weak_widget(&thick_widget));
            handles.thickness = Some(thick_cell);
            col_idx += 1;
        }

        if current_vis.flags[5] {
            if let Some(chrono) = &row.chronological_phase {
                let (widget, gtk_span, cell) = build_group_cell_widget(
                    chrono,
                    EditableField::ChronologicalPhase,
                    i,
                    doc_rows,
                    ctx,
                );
                grid.attach(&widget, col_idx, grid_row, 1, gtk_span);
                handles.chrono = Some(cell);
            }
            col_idx += 1;
        }

        if current_vis.flags[6] {
            if let Some(dating) = &row.dating_period {
                let (widget, gtk_span, cell) =
                    build_group_cell_widget(dating, EditableField::DatingPeriod, i, doc_rows, ctx);
                grid.attach(&widget, col_idx, grid_row, 1, gtk_span);
                handles.dating = Some(cell);
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
        anchors.push(weak_widget(&delete_btn));
        row_anchors.borrow_mut().push(anchors);
        row_handles.push(handles);
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

    DocumentTable {
        signature,
        grid,
        rows: row_handles,
    }
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

#[cfg(test)]
mod tests {
    use super::row_at_y;

    #[test]
    fn empty_has_no_row() {
        assert_eq!(row_at_y(&[], 10.0), None);
    }

    #[test]
    fn single_row_always_wins() {
        assert_eq!(row_at_y(&[(0, 5.0, 40.0)], -100.0), Some(0));
        assert_eq!(row_at_y(&[(0, 5.0, 40.0)], 1000.0), Some(0));
    }

    #[test]
    fn uneven_row_heights_are_respected() {
        // A tall wrapped row (0..100), a short one (106..136), another tall
        // one (142..242). A fixed 36px row height would get all of these wrong.
        let rows = [(0, 0.0, 100.0), (1, 106.0, 136.0), (2, 142.0, 242.0)];
        assert_eq!(row_at_y(&rows, 90.0), Some(0));
        assert_eq!(row_at_y(&rows, 102.0), Some(0)); // in the gap, before its midpoint (103)
        assert_eq!(row_at_y(&rows, 103.0), Some(1)); // at/after the midpoint
        assert_eq!(row_at_y(&rows, 120.0), Some(1));
        assert_eq!(row_at_y(&rows, 138.0), Some(1)); // in the gap, before its midpoint (139)
        assert_eq!(row_at_y(&rows, 139.0), Some(2));
        assert_eq!(row_at_y(&rows, 230.0), Some(2));
    }

    #[test]
    fn outside_the_table_clamps_to_first_and_last_row() {
        let rows = [(0, 10.0, 50.0), (1, 60.0, 100.0)];
        assert_eq!(row_at_y(&rows, -500.0), Some(0));
        assert_eq!(row_at_y(&rows, 5000.0), Some(1));
    }

    #[test]
    fn keeps_original_indices_when_rows_are_skipped() {
        // Row 1 has no measurable bounds; the indices must stay document indices.
        let rows = [(0, 0.0, 30.0), (2, 40.0, 70.0)];
        assert_eq!(row_at_y(&rows, 50.0), Some(2));
    }
}
