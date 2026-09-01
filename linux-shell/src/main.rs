use chronos::bridge::{BarType, ConservationApp, EditableField, Effect, Event, ExportPdfResponse};
use chronos::storage::{BarPattern, ConditionBar};
use chronos::view::DocumentViewModel;
use crux_core::Core;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, Orientation, ScrolledWindow};
use gtk4::{DrawingArea, GestureDrag, Grid, Label, glib, prelude::*};
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

const APP_ID: &str = "org.conservation.Editor";

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
        let export_btn = Button::builder().label("Export PDF").build();
        let status_label = Label::new(Some("Ready"));

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .spacing(12)
            .build();

        container.append(&export_btn);
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

        let core_clone = core.clone();
        let status_clone = status_label.clone();
        let btn_clone = export_btn.clone();
        let scrolled_clone = scrolled_window.clone();

        export_btn.connect_clicked(move |_| {
            let effects = core_clone.process_event(Event::RequestPdfExport);
            process_effects(
                core_clone.clone(),
                effects,
                status_clone.clone(),
                btn_clone.clone(),
                scrolled_clone.clone(),
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

                if let Some(err) = &view.last_error {
                    status.set_markup(&format!("<span foreground='red'>Error: {}</span>", err));
                } else {
                    status.set_text(&format!("Project: {}", view.document.project_name));
                }

                let new_table = build_document_table(
                    &view.document,
                    core.clone(),
                    status.clone(),
                    export_btn.clone(),
                    scrolled_window.clone(),
                );
                scrolled_window.set_child(Some(&new_table));
            }
            Effect::ExportPdf(mut request) => {
                let core = core.clone();
                let status = status.clone();
                let export_btn = export_btn.clone();
                let scrolled_window = scrolled_window.clone();

                glib::spawn_future_local(async move {
                    let file_name = request.operation.file_name.clone();
                    let pdf_bytes = request.operation.pdf_bytes.clone();
                    let save_dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
                    let file_path = save_dir.join(file_name);

                    let response = match fs::write(&file_path, pdf_bytes) {
                        Ok(_) => {
                            ExportPdfResponse::Success(file_path.to_string_lossy().to_string())
                        }
                        Err(e) => ExportPdfResponse::Error(e.to_string()),
                    };

                    let next_effects = core
                        .resolve(&mut request, response)
                        .expect("Failed to resolve effect");
                    process_effects(core, next_effects, status, export_btn, scrolled_window);
                });
            }
        }
    }
}

pub fn build_document_table(
    view_model: &DocumentViewModel,
    core: Rc<Core<ConservationApp>>,
    status: gtk4::Label,
    export_btn: gtk4::Button,
    scrolled_window: gtk4::ScrolledWindow,
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

    let headers = [
        "No.",
        "Graphical representation\nof the",
        "Woven trim",
        "Layer characteristics",
        "Layer thickness",
        "Chronological phase",
        "Dating period",
        "", // Delete
    ];

    for (col, title) in headers.iter().enumerate() {
        let label = Label::builder()
            .use_markup(true)
            .label(&format!("<b>{}</b>", title))
            .build();
        grid.attach(&label, col as i32, 0, 1, 1);
    }

    let mut current_grid_row = 1;

    let top_gap = build_insert_row(
        None,
        core.clone(),
        status.clone(),
        export_btn.clone(),
        scrolled_window.clone(),
    );
    grid.attach(&top_gap, 0, current_grid_row, 8, 1);
    current_grid_row += 1;

    for row in view_model.rows.iter() {
        let grid_row = current_grid_row;

        let row_num_label = Label::new(Some(&row.row_number.to_string()));
        row_num_label.set_margin_top(4);
        row_num_label.set_margin_bottom(4);
        row_num_label.set_valign(gtk4::Align::Center);
        grid.attach(&row_num_label, 0, grid_row, 1, 1);

        let graph_bar = build_interactive_condition_bar(
            row.id,
            BarType::GraphicalRepresentation,
            row.graphical_representation.clone(),
            150,
            core.clone(),
            status.clone(),
            export_btn.clone(),
            scrolled_window.clone(),
        );
        grid.attach(&graph_bar, 1, grid_row, 1, 1);

        let woven_bar = build_interactive_condition_bar(
            row.id,
            BarType::WovenTrim,
            row.woven_trim.clone(),
            60,
            core.clone(),
            status.clone(),
            export_btn.clone(),
            scrolled_window.clone(),
        );
        grid.attach(&woven_bar, 2, grid_row, 1, 1);

        let char_widget = create_editable_widget(
            &row.layer_characteristics,
            core.clone(),
            status.clone(),
            export_btn.clone(),
            scrolled_window.clone(),
            {
                let id = row.id;
                move |new_val| Event::UpdateText {
                    id,
                    field: EditableField::LayerCharacteristics,
                    new_value: new_val,
                }
            },
        );
        grid.attach(&char_widget, 3, grid_row, 1, 1);

        let thick_widget = create_editable_widget(
            &row.layer_thickness,
            core.clone(),
            status.clone(),
            export_btn.clone(),
            scrolled_window.clone(),
            {
                let id = row.id;
                move |new_val| Event::UpdateText {
                    id,
                    field: EditableField::LayerThickness,
                    new_value: new_val,
                }
            },
        );
        grid.attach(&thick_widget, 4, grid_row, 1, 1);

        if let Some(chrono) = &row.chronological_phase {
            let chrono_widget = create_editable_widget(
                &chrono.value,
                core.clone(),
                status.clone(),
                export_btn.clone(),
                scrolled_window.clone(),
                {
                    let member_ids = chrono.member_ids.clone();
                    move |new_val| Event::UpdateGroupText {
                        member_ids: member_ids.clone(),
                        field: EditableField::ChronologicalPhase,
                        new_value: new_val,
                    }
                },
            );
            grid.attach(&chrono_widget, 5, grid_row, 1, chrono.span as i32);
        }

        if let Some(dating) = &row.dating_period {
            let dating_widget = create_editable_widget(
                &dating.value,
                core.clone(),
                status.clone(),
                export_btn.clone(),
                scrolled_window.clone(),
                {
                    let member_ids = dating.member_ids.clone();
                    move |new_val| Event::UpdateGroupText {
                        member_ids: member_ids.clone(),
                        field: EditableField::DatingPeriod,
                        new_value: new_val,
                    }
                },
            );
            grid.attach(&dating_widget, 6, grid_row, 1, dating.span as i32);
        }

        let delete_btn = gtk4::Button::builder()
            .label("×")
            .has_frame(false)
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

        delete_btn.connect_clicked(move |_| {
            let core_inner = core_btn.clone();
            let status_inner = status_btn.clone();
            let export_btn_inner = export_btn_clone.clone();
            let scrolled_inner = scrolled_btn.clone();

            glib::idle_add_local_once(move || {
                let effects = core_inner.process_event(Event::DeleteLayer(row_id));
                process_effects(
                    core_inner,
                    effects,
                    status_inner,
                    export_btn_inner,
                    scrolled_inner,
                );
            });
        });
        grid.attach(&delete_btn, 7, grid_row, 1, 1);

        current_grid_row += 1;

        if row.is_last_in_group {
            let gap = build_insert_row(
                Some(row.id),
                core.clone(),
                status.clone(),
                export_btn.clone(),
                scrolled_window.clone(),
            );
            grid.attach(&gap, 0, current_grid_row, 8, 1);
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

        glib::idle_add_local_once(move || {
            let effects = core_inner.process_event(Event::InsertLayerAfter(after_id));
            process_effects(
                core_inner,
                effects,
                status_inner,
                export_btn_inner,
                scrolled_inner,
            );
        });
    });
    btn.upcast()
}

pub fn build_interactive_condition_bar(
    id: u32,
    bar_type: BarType,
    condition: ConditionBar,
    width_request: i32,
    core: Rc<Core<ConservationApp>>,
    status: gtk4::Label,
    export_btn: gtk4::Button,
    scrolled_window: gtk4::ScrolledWindow,
) -> DrawingArea {
    let drawing_area = DrawingArea::builder()
        .hexpand(true)
        .width_request(width_request)
        .height_request(24)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    let local_pct = Rc::new(RefCell::new(condition.fill_percentage));
    let is_hovering_handle = Rc::new(RefCell::new(false));
    let is_dragging = Rc::new(RefCell::new(false));

    let draw_pct = local_pct.clone();
    let draw_hover = is_hovering_handle.clone();
    let draw_dragging = is_dragging.clone();

    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;
        let current_pct = *draw_pct.borrow();
        let fill_w = w * (current_pct as f64 / 100.0);

        if w == 0.0 || h == 0.0 {
            return;
        }

        cr.set_source_rgb(0.9, 0.9, 0.9);
        cr.rectangle(0.0, 0.0, w, h);
        cr.fill().expect("fill track");
        cr.set_source_rgb(0.7, 0.7, 0.7);
        cr.rectangle(0.0, 0.0, w, h);
        cr.stroke().expect("stroke track");

        if fill_w > 0.0 {
            cr.rectangle(0.0, 0.0, fill_w, h);
            cr.clip();
            cr.set_source_rgb(0.3, 0.3, 0.3);
            cr.set_line_width(1.5);
            match condition.pattern {
                BarPattern::Solid => {
                    cr.rectangle(0.0, 0.0, fill_w, h);
                    cr.fill().expect("fill solid");
                }
                BarPattern::DiagonalHatch => {
                    let step = 8.0;
                    let mut x = -h;
                    while x < fill_w {
                        cr.move_to(x, h);
                        cr.line_to(x + h, 0.0);
                        x += step;
                    }
                    cr.stroke().expect("stroke hatch");
                }
                BarPattern::CrossedOut => {
                    let step = 12.0;
                    let mut x = 0.0;
                    while x < fill_w {
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
                    let mut x = 0.0;
                    let mid_y = h / 2.0;
                    let amp = (h / 2.0) - 4.0;
                    cr.move_to(0.0, mid_y);
                    while x < fill_w {
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

        let hovering = *draw_hover.borrow();
        let dragging = *draw_dragging.borrow();

        if hovering || dragging {
            let handle_w = 6.0;
            let handle_h = h + 8.0;
            let handle_x = fill_w - (handle_w / 2.0);
            let handle_y = -4.0;
            cr.set_source_rgb(0.2, 0.5, 0.9);
            cr.rectangle(handle_x, handle_y, handle_w, handle_h);
            cr.fill().expect("fill handle");
        } else {
            cr.set_source_rgb(0.4, 0.4, 0.4);
            cr.set_line_width(2.0);
            cr.move_to(fill_w, 0.0);
            cr.line_to(fill_w, h);
            cr.stroke().expect("stroke edge");
        }
    });

    // --- HOVER DETECTION ---
    let motion = gtk4::EventControllerMotion::new();
    let motion_area = drawing_area.clone();
    let motion_hover = is_hovering_handle.clone();
    let motion_pct = local_pct.clone();

    motion.connect_motion(move |_controller, x, _y| {
        let widget = _controller.widget().unwrap();
        let w = widget.width() as f64;
        let fill_w = w * (*motion_pct.borrow() as f64 / 100.0);
        let grab_zone = 10.0;
        let is_over_handle = x >= (fill_w - grab_zone) && x <= (fill_w + grab_zone);

        let mut hover_state = motion_hover.borrow_mut();
        if *hover_state != is_over_handle {
            *hover_state = is_over_handle;
            motion_area.queue_draw();
        }
    });

    let leave_hover = is_hovering_handle.clone();
    let leave_area = drawing_area.clone();
    let leave_dragging = is_dragging.clone();
    motion.connect_leave(move |_controller| {
        if !*leave_dragging.borrow() {
            *leave_hover.borrow_mut() = false;
            leave_area.queue_draw();
        }
    });

    drawing_area.add_controller(motion);

    // --- DRAG / CLICK GESTURE ---
    let drag = GestureDrag::new();

    let started_on_handle = Rc::new(RefCell::new(false));
    let did_drag = Rc::new(RefCell::new(false));

    let drag_begin_pct = local_pct.clone();
    let drag_begin_is_dragging = is_dragging.clone();
    let drag_begin_started = started_on_handle.clone();
    let drag_begin_did_drag = did_drag.clone();

    drag.connect_drag_begin(move |gesture, start_x, _start_y| {
        let widget = gesture.widget().unwrap();
        let w = widget.width() as f64;
        let fill_w = w * (*drag_begin_pct.borrow() as f64 / 100.0);
        let grab_zone = 10.0;
        let on_handle = start_x >= (fill_w - grab_zone) && start_x <= (fill_w + grab_zone);

        *drag_begin_started.borrow_mut() = on_handle;
        *drag_begin_did_drag.borrow_mut() = false;

        if on_handle {
            *drag_begin_is_dragging.borrow_mut() = true;
        }
    });

    let drag_update_area = drawing_area.clone();
    let drag_update_pct = local_pct.clone();
    let drag_update_started = started_on_handle.clone();
    let drag_update_did_drag = did_drag.clone();

    drag.connect_drag_update(move |gesture, offset_x, _offset_y| {
        if !*drag_update_started.borrow() {
            return;
        }
        *drag_update_did_drag.borrow_mut() = true;

        if let Some((start_x, _)) = gesture.start_point() {
            let widget = gesture.widget().unwrap();
            let w = widget.width() as f64;
            let current_x = start_x + offset_x;
            let mut new_pct = (current_x / w * 100.0) as f32;
            new_pct = new_pct.clamp(0.0, 100.0);
            *drag_update_pct.borrow_mut() = new_pct;
            drag_update_area.queue_draw();
        }
    });

    let drag_end_is_dragging = is_dragging.clone();
    let drag_end_hover = is_hovering_handle.clone();
    let drag_end_area = drawing_area.clone();
    let drag_end_pct = local_pct.clone();
    let drag_end_core = core.clone();
    let drag_end_id = id;
    let drag_end_bar_type = bar_type.clone();
    let drag_end_started = started_on_handle.clone();
    let drag_end_did_drag = did_drag.clone();

    let drag_end_status = status.clone();
    let drag_end_export_btn = export_btn.clone();
    let drag_end_scrolled_window = scrolled_window.clone();

    drag.connect_drag_end(move |_gesture, _offset_x, _offset_y| {
        let was_on_handle = *drag_end_started.borrow();
        let was_drag = *drag_end_did_drag.borrow();

        *drag_end_is_dragging.borrow_mut() = false;
        *drag_end_hover.borrow_mut() = false;
        drag_end_area.queue_draw();

        let event = if !was_drag {
            Event::CyclePattern {
                id: drag_end_id,
                bar_type: drag_end_bar_type.clone(),
            }
        } else if was_on_handle {
            let final_pct = *drag_end_pct.borrow();
            Event::UpdateConditionFill {
                id: drag_end_id,
                bar_type: drag_end_bar_type.clone(),
                new_percentage: final_pct,
            }
        } else {
            return;
        };

        // FIX: Clone variables *inside* the Fn closure
        let core_inner = drag_end_core.clone();
        let status_inner = drag_end_status.clone();
        let export_btn_inner = drag_end_export_btn.clone();
        let scrolled_inner = drag_end_scrolled_window.clone();

        glib::idle_add_local_once(move || {
            let effects = core_inner.process_event(event);
            process_effects(
                core_inner,
                effects,
                status_inner,
                export_btn_inner,
                scrolled_inner,
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
    on_commit: impl Fn(String) -> Event + 'static,
) -> gtk4::Widget {
    let on_commit = Rc::new(on_commit);

    let stack = gtk4::Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::None);

    stack.set_margin_top(4);
    stack.set_margin_bottom(4);
    stack.set_valign(gtk4::Align::Center);

    let label = gtk4::Label::new(Some(initial_text));
    label.set_wrap(true);
    label.set_max_width_chars(30);
    label.set_width_chars(5);

    label.set_halign(gtk4::Align::Fill);
    label.set_valign(gtk4::Align::Center);
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

    label.add_controller(click_gesture);

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

                glib::idle_add_local_once(move || {
                    let effects = core_inner.process_event(event);
                    process_effects(
                        core_inner,
                        effects,
                        status_inner,
                        export_btn_inner,
                        scrolled_inner,
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

    // --- Commit on focus out (clicking outside) ---
    let focus_controller = gtk4::EventControllerFocus::new();
    let commit_clone2 = commit_and_switch;
    focus_controller.connect_leave(move |_controller| {
        commit_clone2();
    });
    entry.add_controller(focus_controller);

    stack.upcast()
}
