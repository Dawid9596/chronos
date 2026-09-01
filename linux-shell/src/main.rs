use chronos::bridge::{ConservationApp, Effect, Event, ExportPdfResponse};
use chronos::storage::{BarPattern, ConditionBar};
use chronos::view::DocumentViewModel;
use crux_core::Core;
use gtk4::{Align, DrawingArea, GestureDrag, Grid, Label, glib, prelude::*};
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, Orientation, ScrolledWindow};
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

const APP_ID: &str = "org.conservation.Editor";

fn main() {
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
            .default_width(900)
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
                if view.is_exporting {
                    export_btn.set_label("Exporting...");
                } else {
                    export_btn.set_label("Export PDF");
                }

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
        .row_spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .hexpand(true)
        .vexpand(true)
        .build();

    let headers = [
        "Item ID",
        "Condition",
        "Treatment Notes",
        "Material",
        "Location/Epoch",
    ];
    for (col, title) in headers.iter().enumerate() {
        let label = Label::builder()
            .use_markup(true)
            .label(&format!("<b>{}</b>", title))
            .build();
        grid.attach(&label, col as i32, 0, 1, 1);
    }

    for (i, row) in view_model.rows.iter().enumerate() {
        let grid_row = (i + 1) as i32;

        grid.attach(&Label::new(Some(&row.item_id)), 0, grid_row, 1, 1);

        let interactive_bar = build_interactive_condition_bar(
            row.item_id.clone(),
            row.condition.clone(),
            core.clone(),
            status.clone(),
            export_btn.clone(),
            scrolled_window.clone(),
        );
        grid.attach(&interactive_bar, 1, grid_row, 1, 1);

        let notes_label = Label::builder()
            .label(&row.treatment_notes)
            .wrap(true)
            .max_width_chars(30)
            .build();
        grid.attach(&notes_label, 2, grid_row, 1, 1);

        if let Some(mat) = &row.material_group {
            let label = Label::builder()
                .label(&mat.value)
                .valign(Align::Center)
                .build();
            grid.attach(&label, 3, grid_row, 1, mat.span as i32);
        }
        if let Some(loc) = &row.location_group {
            let label = Label::builder()
                .label(&loc.value)
                .valign(Align::Center)
                .build();
            grid.attach(&label, 4, grid_row, 1, loc.span as i32);
        }
    }
    grid
}

pub fn build_condition_drawing_area(condition: ConditionBar) -> DrawingArea {
    let drawing_area = DrawingArea::builder()
        .hexpand(true)
        .width_request(150)
        .height_request(24)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;

        if w == 0.0 || h == 0.0 {
            return;
        }

        cr.set_source_rgb(0.9, 0.9, 0.9);
        cr.rectangle(0.0, 0.0, w, h);
        cr.fill().expect("Failed to fill track");
        cr.set_source_rgb(0.7, 0.7, 0.7);
        cr.rectangle(0.0, 0.0, w, h);
        cr.stroke().expect("Failed to stroke track");

        let fill_w = w * (condition.fill_percentage as f64 / 100.0);

        if fill_w > 0.0 {
            cr.rectangle(0.0, 0.0, fill_w, h);
            cr.clip();

            cr.set_source_rgb(0.3, 0.3, 0.3); // Dark gray for the pattern
            cr.set_line_width(1.5);

            match condition.pattern {
                BarPattern::Solid => {
                    cr.rectangle(0.0, 0.0, fill_w, h);
                    cr.fill().expect("Failed to fill solid bar");
                }
                BarPattern::DiagonalHatch => {
                    let step = 8.0;
                    let mut x = -h;
                    while x < fill_w {
                        cr.move_to(x, h);
                        cr.line_to(x + h, 0.0);
                        x += step;
                    }
                    cr.stroke().expect("Failed to stroke hatch");
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
                    cr.stroke().expect("Failed to stroke crosses");
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
                    cr.stroke().expect("Failed to stroke wavy path");
                }
            }
            cr.reset_clip();
        }
    });

    drawing_area
}

pub fn build_interactive_condition_bar(
    item_id: String,
    condition: ConditionBar,
    core: Rc<Core<ConservationApp>>,
    status: gtk4::Label,
    export_btn: gtk4::Button,
    scrolled_window: gtk4::ScrolledWindow,
) -> DrawingArea {
    let drawing_area = DrawingArea::builder()
        .hexpand(true)
        .height_request(24)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    let local_pct = Rc::new(RefCell::new(condition.fill_percentage));
    let is_hovering_handle = Rc::new(RefCell::new(false));
    let is_dragging = Rc::new(RefCell::new(false));

    // --- CAIRO DRAWING ---
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
    let drag_end_item_id = item_id.clone();
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
                item_id: drag_end_item_id.clone(),
            }
        } else if was_on_handle {
            let final_pct = *drag_end_pct.borrow();
            Event::UpdateConditionFill {
                item_id: drag_end_item_id.clone(),
                new_percentage: final_pct,
            }
        } else {
            return;
        };

        let effects = drag_end_core.process_event(event);
        process_effects(
            drag_end_core.clone(),
            effects,
            drag_end_status.clone(),
            drag_end_export_btn.clone(),
            drag_end_scrolled_window.clone(),
        );
    });

    drawing_area.add_controller(drag);
    drawing_area
}
