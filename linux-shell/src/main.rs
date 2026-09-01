use chronos::bridge::{
    BarType, ConservationApp, EditableField, Effect, Event, ExportPdfResponse, SlotEdge,
};
use chronos::storage::{BarPattern, ConditionBar};
use chronos::view::DocumentViewModel;
use crux_core::Core;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, Orientation, ScrolledWindow};
use gtk4::{DrawingArea, GestureDrag, Grid, Label, ToggleButton, glib, prelude::*};
use std::{
    cell::{Cell, RefCell},
    fs,
    path::PathBuf,
    rc::Rc,
};

const APP_ID: &str = "org.conservation.Editor";

#[derive(Clone)]
struct ColumnVisibility {
    flags: [bool; 7],
}

impl Default for ColumnVisibility {
    fn default() -> Self {
        Self { flags: [true; 7] }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum BarTool {
    Edit,
    Create,
    Delete,
}

#[derive(Clone, Copy, PartialEq)]
enum DragTarget {
    None,
    Edge { slot_index: usize, edge: SlotEdge },
    Slot(usize),
}

fn main() {
    gtk4::init().expect("Failed to initialize GTK4");

    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        "button.insert-gap { \
            min-height: 6px; \
            padding: 0; \
            background-color: transparent; \
            transition: all 0.15s ease-out; \
        } \
        button.insert-gap:hover { \
            background-color: alpha(@theme_selected_bg_color, 0.3); \
            min-height: 12px; \
        }",
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        let core = Rc::new(Core::<ConservationApp>::default());
        let vis = Rc::new(RefCell::new(ColumnVisibility::default()));
        let tool = Rc::new(Cell::new(BarTool::Edit));

        let export_btn = Button::builder().label("Export PDF").build();
        let settings_btn = Button::builder().label("⚙️").build();

        let edit_btn = ToggleButton::builder().label("✥ Edit").active(true).build();
        let create_btn = ToggleButton::builder().label("＋ Create").build();
        let delete_btn = ToggleButton::new();

        let delete_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        let trash_icon = Label::builder().label("🗑").margin_top(5).build();
        delete_box.append(&trash_icon);
        delete_box.append(&Label::new(Some("Delete")));

        delete_btn.set_child(Some(&delete_box));
        edit_btn.set_group(Option::<&gtk4::ToggleButton>::None);
        create_btn.set_group(Some(&edit_btn));
        delete_btn.set_group(Some(&edit_btn));

        let tool_clone = tool.clone();
        edit_btn.connect_toggled(move |b| {
            if b.is_active() {
                tool_clone.set(BarTool::Edit);
            }
        });
        let tool_clone = tool.clone();
        create_btn.connect_toggled(move |b| {
            if b.is_active() {
                tool_clone.set(BarTool::Create);
            }
        });
        let tool_clone = tool.clone();
        delete_btn.connect_toggled(move |b| {
            if b.is_active() {
                tool_clone.set(BarTool::Delete);
            }
        });

        let tool_bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();

        tool_bar.append(&edit_btn);
        tool_bar.append(&create_btn);
        tool_bar.append(&delete_btn);

        let left_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .build();
        left_box.append(&settings_btn);
        left_box.append(&export_btn);

        let top_bar = gtk4::CenterBox::new();
        top_bar.set_start_widget(Some(&left_box));
        top_bar.set_center_widget(Some(&tool_bar));

        let status_label = Label::new(Some("Ready"));
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .spacing(12)
            .build();

        container.append(&top_bar);
        container.append(&status_label);

        let scrolled_window = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let initial_view = core.view();
        let initial_table = build_document_table(
            &initial_view.document,
            core.clone(),
            status_label.clone(),
            export_btn.clone(),
            scrolled_window.clone(),
            vis.clone(),
            tool.clone(),
        );
        scrolled_window.set_child(Some(&initial_table));

        container.append(&scrolled_window);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Conservation Editor")
            .default_width(1100)
            .default_height(600)
            .child(&container)
            .build();
        window.present();

        // --- Settings Button Logic ---
        let vis_clone = vis.clone();
        let scrolled_clone = scrolled_window.clone();
        let core_clone = core.clone();
        let status_clone = status_label.clone();
        let btn_clone = export_btn.clone();
        let tool_clone = tool.clone();

        settings_btn.connect_clicked(move |btn| {
            let settings_popover = gtk4::Popover::new();
            let popover_vbox = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(8)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(12)
                .margin_end(12)
                .build();

            let current_vis = vis_clone.borrow().clone();
            let labels = [
                "No.",
                "Graphical representation",
                "Woven trim",
                "Layer characteristics",
                "Layer thickness",
                "Chronological phase",
                "Dating period",
            ];
            for (i, label_text) in labels.iter().enumerate() {
                let cb = gtk4::CheckButton::with_label(label_text);
                cb.set_active(current_vis.flags[i]);
                let vis_toggle = vis_clone.clone();
                let scrolled_toggle = scrolled_clone.clone();
                let core_toggle = core_clone.clone();
                let status_toggle = status_clone.clone();
                let btn_toggle = btn_clone.clone();
                let tool_toggle = tool_clone.clone();
                let idx = i;
                cb.connect_toggled(move |check_btn| {
                    let is_active = check_btn.is_active();
                    vis_toggle.borrow_mut().flags[idx] = is_active;
                    let view = core_toggle.view();
                    let new_table = build_document_table(
                        &view.document,
                        core_toggle.clone(),
                        status_toggle.clone(),
                        btn_toggle.clone(),
                        scrolled_toggle.clone(),
                        vis_toggle.clone(),
                        tool_toggle.clone(),
                    );
                    scrolled_toggle.set_child(Some(&new_table));
                });

                popover_vbox.append(&cb);
            }

            settings_popover.set_child(Some(&popover_vbox));
            settings_popover.set_parent(btn);
            settings_popover.popup();
        });

        // --- Export Button Logic ---
        let core_clone = core.clone();
        let status_clone = status_label.clone();
        let btn_clone = export_btn.clone();
        let scrolled_clone = scrolled_window.clone();
        let vis_export = vis.clone();
        let tool_export = tool.clone();
        export_btn.connect_clicked(move |_| {
            let effects = core_clone.process_event(Event::RequestPdfExport);
            process_effects(
                core_clone.clone(),
                effects,
                status_clone.clone(),
                btn_clone.clone(),
                scrolled_clone.clone(),
                vis_export.clone(),
                tool_export.clone(),
            );
        });
    });

    app.run();
}

fn process_effects(
    core: Rc<Core<ConservationApp>>,
    effects: Vec<Effect>,
    status: gtk4::Label,
    export_btn: gtk4::Button,
    scrolled_window: gtk4::ScrolledWindow,
    vis: Rc<RefCell<ColumnVisibility>>,
    tool: Rc<Cell<BarTool>>,
) {
    for effect in effects {
        match effect {
            Effect::Render(_) => {
                let view = core.view();
                export_btn.set_sensitive(!view.is_exporting);
                export_btn.set_label(if view.is_exporting {
                    "Exporting..."
                } else {
                    "Export PDF"
                });

                if let Some(msg) = &view.last_error {
                    if msg.starts_with("Error") {
                        status.set_markup(&format!("<span foreground='red'>{}</span>", msg));
                    } else {
                        status.set_markup(&format!("<span foreground='green'>{}</span>", msg));
                    }
                } else {
                    status.set_text(&format!("Project: {}", view.document.project_name));
                }

                let new_table = build_document_table(
                    &view.document,
                    core.clone(),
                    status.clone(),
                    export_btn.clone(),
                    scrolled_window.clone(),
                    vis.clone(),
                    tool.clone(),
                );
                scrolled_window.set_child(Some(&new_table));
            }
            Effect::ExportPdf(mut request) => {
                let core = core.clone();
                let status = status.clone();
                let export_btn = export_btn.clone();
                let scrolled_window = scrolled_window.clone();
                let vis = vis.clone();
                let tool = tool.clone();

                glib::spawn_future_local(async move {
                    let file_name = request.operation.file_name.clone();
                    let pdf_bytes = request.operation.pdf_bytes.clone();
                    let save_dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
                    let file_path = save_dir.join(file_name);

                    let response = match fs::write(&file_path, pdf_bytes) {
                        Ok(_) => {
                            let _ = std::process::Command::new("xdg-open")
                                .arg(&file_path)
                                .spawn();

                            ExportPdfResponse::Success(format!("Saved to {}", file_path.display()))
                        }
                        Err(e) => ExportPdfResponse::Error(format!("Error: {}", e)),
                    };

                    let next_effects = core
                        .resolve(&mut request, response)
                        .expect("Failed to resolve effect");
                    process_effects(
                        core,
                        next_effects,
                        status,
                        export_btn,
                        scrolled_window,
                        vis,
                        tool,
                    );
                });
            }
        }
    }
}

fn build_document_table(
    view_model: &DocumentViewModel,
    core: Rc<Core<ConservationApp>>,
    status: gtk4::Label,
    export_btn: gtk4::Button,
    scrolled_window: gtk4::ScrolledWindow,
    vis: Rc<RefCell<ColumnVisibility>>,
    tool: Rc<Cell<BarTool>>,
) -> Grid {
    let grid = Grid::builder()
        .column_spacing(16)
        .row_spacing(0)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .hexpand(true)
        .vexpand(true)
        .build();

    let current_vis = vis.borrow();

    let headers = [
        "No.",
        "Graphical representation\nof the layer",
        "Woven trim",
        "Layer characteristics",
        "Layer thickness",
        "Chronological phase",
        "Dating period",
        "",
    ];

    let mut total_visible_cols = 0;
    let mut col_idx = 0;
    for (i, title) in headers.iter().enumerate() {
        let is_visible = if i == 7 { true } else { current_vis.flags[i] };
        if is_visible {
            let label = Label::builder()
                .use_markup(true)
                .label(&format!("<b>{}</b>", title))
                .build();
            grid.attach(&label, col_idx, 0, 1, 1);
            col_idx += 1;
            total_visible_cols += 1;
        }
    }

    let mut current_grid_row = 1;
    let top_gap = build_insert_row(
        None,
        core.clone(),
        status.clone(),
        export_btn.clone(),
        scrolled_window.clone(),
        vis.clone(),
        tool.clone(),
    );
    grid.attach(&top_gap, 0, current_grid_row, total_visible_cols, 1);
    current_grid_row += 1;

    for row in view_model.rows.iter() {
        let grid_row = current_grid_row;
        let mut col_idx = 0;

        if current_vis.flags[0] {
            let row_num_label = Label::new(Some(&row.row_number.to_string()));
            row_num_label.set_margin_top(4);
            row_num_label.set_margin_bottom(4);
            row_num_label.set_valign(gtk4::Align::Center);
            grid.attach(&row_num_label, col_idx, grid_row, 1, 1);
            col_idx += 1;
        }

        if current_vis.flags[1] {
            let graph_bar = build_interactive_condition_bar(
                row.id,
                BarType::GraphicalRepresentation,
                row.graphical_representation.clone(),
                150,
                core.clone(),
                status.clone(),
                export_btn.clone(),
                scrolled_window.clone(),
                vis.clone(),
                tool.clone(),
            );
            grid.attach(&graph_bar, col_idx, grid_row, 1, 1);
            col_idx += 1;
        }

        if current_vis.flags[2] {
            let woven_bar = build_interactive_condition_bar(
                row.id,
                BarType::WovenTrim,
                row.woven_trim.clone(),
                60,
                core.clone(),
                status.clone(),
                export_btn.clone(),
                scrolled_window.clone(),
                vis.clone(),
                tool.clone(),
            );
            grid.attach(&woven_bar, col_idx, grid_row, 1, 1);
            col_idx += 1;
        }

        if current_vis.flags[3] {
            let char_widget = create_editable_widget(
                &row.layer_characteristics,
                core.clone(),
                status.clone(),
                export_btn.clone(),
                scrolled_window.clone(),
                vis.clone(),
                tool.clone(),
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
            let thick_widget = create_editable_widget(
                &row.layer_thickness,
                core.clone(),
                status.clone(),
                export_btn.clone(),
                scrolled_window.clone(),
                vis.clone(),
                tool.clone(),
                {
                    let id = row.id;
                    move |new_val| Event::UpdateText {
                        id,
                        field: EditableField::LayerThickness,
                        new_value: new_val,
                    }
                },
            );
            grid.attach(&thick_widget, col_idx, grid_row, 1, 1);
            col_idx += 1;
        }

        if current_vis.flags[5] {
            if let Some(chrono) = &row.chronological_phase {
                let chrono_widget = create_editable_widget(
                    &chrono.value,
                    core.clone(),
                    status.clone(),
                    export_btn.clone(),
                    scrolled_window.clone(),
                    vis.clone(),
                    tool.clone(),
                    {
                        let member_ids = chrono.member_ids.clone();
                        move |new_val| Event::UpdateGroupText {
                            member_ids: member_ids.clone(),
                            field: EditableField::ChronologicalPhase,
                            new_value: new_val,
                        }
                    },
                );
                grid.attach(&chrono_widget, col_idx, grid_row, 1, chrono.span as i32);
            }
            col_idx += 1;
        }

        if current_vis.flags[6] {
            if let Some(dating) = &row.dating_period {
                let dating_widget = create_editable_widget(
                    &dating.value,
                    core.clone(),
                    status.clone(),
                    export_btn.clone(),
                    scrolled_window.clone(),
                    vis.clone(),
                    tool.clone(),
                    {
                        let member_ids = dating.member_ids.clone();
                        move |new_val| Event::UpdateGroupText {
                            member_ids: member_ids.clone(),
                            field: EditableField::DatingPeriod,
                            new_value: new_val,
                        }
                    },
                );
                grid.attach(&dating_widget, col_idx, grid_row, 1, dating.span as i32);
            }
            col_idx += 1;
        }

        let delete_btn = gtk4::Button::builder()
            .label("×")
            .has_frame(false)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        // flat destructive button
        delete_btn.add_css_class("flat");
        delete_btn.add_css_class("destructive-action");

        let row_id = row.id;
        let core_btn = core.clone();
        let status_btn = status.clone();
        let export_btn_clone = export_btn.clone();
        let scrolled_btn = scrolled_window.clone();
        let vis_btn = vis.clone();
        let tool_btn = tool.clone();

        delete_btn.connect_clicked(move |_| {
            let core_inner = core_btn.clone();
            let status_inner = status_btn.clone();
            let export_btn_inner = export_btn_clone.clone();
            let scrolled_inner = scrolled_btn.clone();
            let vis_inner = vis_btn.clone();
            let tool_inner = tool_btn.clone();
            glib::idle_add_local_once(move || {
                let effects = core_inner.process_event(Event::DeleteLayer(row_id));
                process_effects(
                    core_inner,
                    effects,
                    status_inner,
                    export_btn_inner,
                    scrolled_inner,
                    vis_inner,
                    tool_inner,
                );
            });
        });

        grid.attach(&delete_btn, col_idx, grid_row, 1, 1);
        current_grid_row += 1;

        if row.is_last_in_group {
            let gap = build_insert_row(
                Some(row.id),
                core.clone(),
                status.clone(),
                export_btn.clone(),
                scrolled_window.clone(),
                vis.clone(),
                tool.clone(),
            );
            grid.attach(&gap, 0, current_grid_row, total_visible_cols, 1);
            current_grid_row += 1;
        }
    }

    grid
}

fn build_insert_row(
    after_id: Option<u32>,
    core: Rc<Core<ConservationApp>>,
    status: gtk4::Label,
    export_btn: gtk4::Button,
    scrolled_window: gtk4::ScrolledWindow,
    vis: Rc<RefCell<ColumnVisibility>>,
    tool: Rc<Cell<BarTool>>,
) -> gtk4::Widget {
    let btn = gtk4::Button::builder()
        .has_frame(false)
        .hexpand(true)
        .margin_top(1)
        .margin_bottom(1)
        .build();
    btn.add_css_class("insert-gap");

    btn.connect_clicked(move |_| {
        let core_inner = core.clone();
        let status_inner = status.clone();
        let export_btn_inner = export_btn.clone();
        let scrolled_inner = scrolled_window.clone();
        let vis_inner = vis.clone();
        let tool_inner = tool.clone();

        glib::idle_add_local_once(move || {
            let effects = core_inner.process_event(Event::InsertLayerAfter(after_id));
            process_effects(
                core_inner,
                effects,
                status_inner,
                export_btn_inner,
                scrolled_inner,
                vis_inner,
                tool_inner,
            );
        });
    });

    btn.upcast()
}

fn build_interactive_condition_bar(
    id: u32,
    bar_type: BarType,
    condition: ConditionBar,
    width_request: i32,
    core: Rc<Core<ConservationApp>>,
    status: gtk4::Label,
    export_btn: gtk4::Button,
    scrolled_window: gtk4::ScrolledWindow,
    vis: Rc<RefCell<ColumnVisibility>>,
    tool: Rc<Cell<BarTool>>,
) -> DrawingArea {
    let drawing_area = DrawingArea::builder()
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

    let draw_slots_clone = draw_slots.clone();
    let draw_pattern_clone = draw_pattern.clone();
    let draw_hover_slot_clone = draw_hover_slot.clone();
    let draw_create_preview_clone = draw_create_preview.clone();
    let tool_clone = tool.clone();

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

        let slots = draw_slots_clone.borrow();
        let pattern = draw_pattern_clone.borrow();
        let current_tool = tool_clone.get();

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
                if let Some(hover_idx) = *draw_hover_slot_clone.borrow() {
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
            if let Some(preview_x) = *draw_create_preview_clone.borrow() {
                cr.set_source_rgba(0.2, 0.5, 0.9, 0.5);
                cr.set_line_width(2.0);
                cr.move_to(preview_x, 0.0);
                cr.line_to(preview_x, h);
                cr.stroke().expect("stroke preview");
            }
        }
    });

    let motion = gtk4::EventControllerMotion::new();
    let motion_area = drawing_area.clone();
    let motion_slots = draw_slots.clone();
    let motion_hover_slot = draw_hover_slot.clone();
    let motion_create_preview = draw_create_preview.clone();
    let motion_tool = tool.clone();

    motion.connect_motion(move |_controller, x, _y| {
        let widget = _controller.widget().unwrap();
        let w = widget.width() as f64;
        let slots = motion_slots.borrow();
        let current_tool = motion_tool.get();
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
            let mut hover_state = motion_hover_slot.borrow_mut();
            if *hover_state != found_hover {
                *hover_state = found_hover;
                motion_area.queue_draw();
            }
        } else if current_tool == BarTool::Create {
            let mut preview_state = motion_create_preview.borrow_mut();
            if *preview_state != Some(x) {
                *preview_state = Some(x);
                motion_area.queue_draw();
            }
        }
    });

    let leave_hover_slot = draw_hover_slot.clone();
    let leave_create_preview = draw_create_preview.clone();
    let leave_area = drawing_area.clone();
    let leave_tool = tool.clone();

    motion.connect_leave(move |_controller| {
        let current_tool = leave_tool.get();
        if current_tool == BarTool::Edit {
            *leave_hover_slot.borrow_mut() = None;
        } else if current_tool == BarTool::Create {
            *leave_create_preview.borrow_mut() = None;
        }
        leave_area.queue_draw();
    });
    drawing_area.add_controller(motion);

    let drag = GestureDrag::new();
    let drag_target_clone = drag_target.clone();
    let drag_is_ambiguous = is_ambiguous.clone();
    let drag_slots = draw_slots.clone();
    let drag_tool = tool.clone();

    drag.connect_drag_begin(move |gesture, start_x, _start_y| {
        let widget = gesture.widget().unwrap();
        let w = widget.width() as f64;
        let slots = drag_slots.borrow();
        let grab_zone = 10.0;
        let current_tool = drag_tool.get();

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
            *drag_target_clone.borrow_mut() = target;
            *drag_is_ambiguous.borrow_mut() = ambiguous;
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
            *drag_target_clone.borrow_mut() = target;
        }
    });

    let drag_update_target = drag_target.clone();
    let drag_update_is_ambiguous = is_ambiguous.clone();
    let drag_update_area = drawing_area.clone();
    let drag_update_slots = draw_slots.clone();

    drag.connect_drag_update(move |gesture, offset_x, _offset_y| {
        let widget = gesture.widget().unwrap();
        let w = widget.width() as f64;
        let mut target = drag_update_target.borrow_mut();

        if let DragTarget::Edge { edge, slot_index } = &mut *target {
            let slot_idx = *slot_index;

            if *drag_update_is_ambiguous.borrow() && offset_x.abs() > 3.0 {
                *edge = if offset_x > 0.0 {
                    SlotEdge::End
                } else {
                    SlotEdge::Start
                };
                *drag_update_is_ambiguous.borrow_mut() = false;
            }

            // Live local preview while dragging
            if let Some((start_x, _)) = gesture.start_point() {
                let current_x = start_x + offset_x;
                let new_pct = (current_x / w * 100.0).clamp(0.0, 100.0) as f32;

                let mut slots = drag_update_slots.borrow_mut();

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
                        SlotEdge::Start => {
                            slot.start_pct = new_pct.clamp(min_start, slot.end_pct);
                        }
                        SlotEdge::End => {
                            slot.end_pct = new_pct.clamp(slot.start_pct, max_end);
                        }
                    }
                }
            }
            drag_update_area.queue_draw();
        }
    });

    let drag_end_target = drag_target.clone();
    let drag_end_is_ambiguous = is_ambiguous.clone();
    let drag_end_slots = draw_slots.clone();
    let drag_end_tool = tool.clone();
    let drag_end_core = core.clone();
    let drag_end_id = id;
    let drag_end_bar_type = bar_type.clone();
    let drag_end_status = status.clone();
    let drag_end_export_btn = export_btn.clone();
    let drag_end_scrolled_window = scrolled_window.clone();
    let drag_end_vis = vis.clone();
    let drag_end_area = drawing_area.clone();

    drag.connect_drag_end(move |gesture, offset_x, _offset_y| {
        let widget = gesture.widget().unwrap();
        let w = widget.width() as f64;
        let _slots = drag_end_slots.borrow();
        let current_tool = drag_end_tool.get();

        let mut target = drag_end_target.borrow_mut();
        let is_amb = *drag_end_is_ambiguous.borrow();

        let event = match (current_tool, *target) {
            (BarTool::Edit, DragTarget::None) => Event::CyclePattern {
                id: drag_end_id,
                bar_type: drag_end_bar_type.clone(),
            },
            (BarTool::Create, _) => {
                let click_x = gesture.start_point().map(|(x, _)| x).unwrap_or(0.0);
                let pct = (click_x / w * 100.0).clamp(0.0, 100.0) as f32;
                Event::AddSlot {
                    id: drag_end_id,
                    bar_type: drag_end_bar_type.clone(),
                    at_pct: pct,
                }
            }
            (BarTool::Delete, DragTarget::Slot(slot_index)) => Event::DeleteSlot {
                id: drag_end_id,
                bar_type: drag_end_bar_type.clone(),
                slot_index,
            },
            (BarTool::Edit, DragTarget::Edge { slot_index, edge }) => {
                if is_amb && offset_x.abs() <= 3.0 {
                    Event::CyclePattern {
                        id: drag_end_id,
                        bar_type: drag_end_bar_type.clone(),
                    }
                } else {
                    let start_x = gesture.start_point().map(|(x, _)| x).unwrap_or(0.0);
                    let current_x = start_x + offset_x;
                    let new_pct = (current_x / w * 100.0).clamp(0.0, 100.0) as f32;
                    Event::UpdateSlotEdge {
                        id: drag_end_id,
                        bar_type: drag_end_bar_type.clone(),
                        slot_index,
                        edge,
                        new_pct,
                    }
                }
            }
            _ => {
                *target = DragTarget::None;
                drag_end_area.queue_draw();
                return;
            }
        };

        *target = DragTarget::None;
        drag_end_area.queue_draw();

        let core_inner = drag_end_core.clone();
        let status_inner = drag_end_status.clone();
        let export_btn_inner = drag_end_export_btn.clone();
        let scrolled_inner = drag_end_scrolled_window.clone();
        let vis_inner = drag_end_vis.clone();
        let tool_inner = tool.clone();

        glib::idle_add_local_once(move || {
            let effects = core_inner.process_event(event);
            process_effects(
                core_inner,
                effects,
                status_inner,
                export_btn_inner,
                scrolled_inner,
                vis_inner,
                tool_inner,
            );
        });
    });

    drawing_area.add_controller(drag);

    drawing_area
}

/// Helper to create an inline editable text cell
fn create_editable_widget(
    initial_text: &str,
    core: Rc<Core<ConservationApp>>,
    status: gtk4::Label,
    export_btn: gtk4::Button,
    scrolled_window: gtk4::ScrolledWindow,
    vis: Rc<RefCell<ColumnVisibility>>,
    tool: Rc<Cell<BarTool>>,
    on_commit: impl Fn(String) -> Event + 'static,
) -> gtk4::Widget {
    let on_commit = Rc::new(on_commit);

    let stack = gtk4::Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::None);

    stack.set_margin_top(4);
    stack.set_margin_bottom(4);
    stack.set_valign(gtk4::Align::Fill);

    let label = gtk4::Label::new(Some(initial_text));
    label.set_wrap(true);
    label.set_max_width_chars(30);
    label.set_width_chars(5);

    label.set_halign(gtk4::Align::Fill);
    label.set_valign(gtk4::Align::Fill);
    label.set_xalign(0.0);
    // Add a slight left margin to match the internal text padding of the Entry
    label.set_margin_start(2);

    let entry = gtk4::Entry::new();
    entry.set_text(initial_text);
    entry.set_has_frame(false);
    entry.set_valign(gtk4::Align::Center);
    entry.set_width_chars(5);

    stack.add_named(&label, Some("label"));
    stack.add_named(&entry, Some("entry"));
    stack.set_visible_child_name("label");

    // --- Double click to enter edit mode ---
    let click_gesture = gtk4::GestureClick::new();
    click_gesture.set_button(1);

    let stack_clone = stack.clone();
    let entry_clone = entry.clone();

    click_gesture.connect_pressed(move |_gesture, n_press, _x, _y| {
        if n_press == 2 {
            stack_clone.set_visible_child_name("entry");
            entry_clone.grab_focus();
            entry_clone.select_region(0, -1);
        }
    });

    stack.add_controller(click_gesture);

    // --- Commit Logic ---
    let commit_and_switch = {
        let stack = stack.clone();
        let entry = entry.clone();
        let label = label.clone();
        let core = core.clone();
        let status = status.clone();
        let export_btn = export_btn.clone();
        let scrolled_window = scrolled_window.clone();
        let on_commit = on_commit.clone();
        let vis = vis.clone();
        let tool = tool.clone();

        move || {
            if stack.visible_child_name().as_deref() == Some("entry") {
                let new_text = entry.text().to_string();
                stack.set_visible_child_name("label");
                label.set_text(&new_text);

                let event = on_commit(new_text);

                let core_inner = core.clone();
                let status_inner = status.clone();
                let export_btn_inner = export_btn.clone();
                let scrolled_inner = scrolled_window.clone();
                let vis_inner = vis.clone();
                let tool_inner = tool.clone();

                glib::idle_add_local_once(move || {
                    let effects = core_inner.process_event(event);
                    process_effects(
                        core_inner,
                        effects,
                        status_inner,
                        export_btn_inner,
                        scrolled_inner,
                        vis_inner,
                        tool_inner,
                    );
                });
            }
        }
    };

    // --- Commit on Enter ---
    let commit_clone = commit_and_switch.clone();
    entry.connect_activate(move |_| {
        commit_clone();
    });

    // --- Cancel on Esc ---
    let key_controller = gtk4::EventControllerKey::new();
    let stack_clone = stack.clone();
    let entry_clone = entry.clone();
    let label_clone = label.clone();

    key_controller.connect_key_pressed(move |_controller, key, _keycode, _state| {
        if key == gtk4::gdk::Key::Escape {
            stack_clone.set_visible_child_name("label");
            entry_clone.set_text(label_clone.text().as_str());
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    entry.add_controller(key_controller);

    // --- Commit on focus out ---
    let focus_controller = gtk4::EventControllerFocus::new();
    let commit_clone2 = commit_and_switch;
    focus_controller.connect_leave(move |_controller| {
        commit_clone2();
    });
    entry.add_controller(focus_controller);

    stack.upcast()
}
