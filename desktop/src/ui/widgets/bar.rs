use crate::context::AppContext;
use crate::tool::{BarInteractionState, DragTarget};
use chronos::bridge::{BarType, Event};
use chronos::storage::{BarPattern, ConditionBar, Slot};
use gtk4::{gdk, glib, prelude::*};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// --- SVG-file patterns ---

const SVG_TILES_CACHE_MAX: usize = 64;

thread_local! {
    /// Cache of ready-to-tile SVG surfaces keyed by (tile name, height).
    static SVG_TILES: RefCell<HashMap<(&'static str, i32), Option<gtk4::cairo::ImageSurface>>> =
        RefCell::new(HashMap::new());
}

static SVG_TREES: std::sync::OnceLock<HashMap<&'static str, Option<resvg::usvg::Tree>>> =
    std::sync::OnceLock::new();

/// Parses all tile SVGs once, lazily, keyed by their file names (as given by
/// `patterns::tile`, which is also the name the PDF world serves them under).
/// Parse failures become None entries so the matching tile is skipped (never
/// a panic inside a draw handler).
fn tree_for(name: &'static str) -> Option<&'static resvg::usvg::Tree> {
    let trees = SVG_TREES.get_or_init(|| {
        chronos::patterns::all_tiles()
            .map(|(name, data)| {
                let options = resvg::usvg::Options {
                    fontdb: std::sync::Arc::new(resvg::usvg::fontdb::Database::new()),
                    ..resvg::usvg::Options::default()
                };
                (name, resvg::usvg::Tree::from_data(data, &options).ok())
            })
            .collect()
    });
    trees.get(name).and_then(|tree| tree.as_ref())
}

/// Rasterizes tile `idx` at exactly the bar's height into a surface that can
/// be tiled 1:1 without any pattern matrix.
///
/// Deriving the scales from the unrounded h and rounding
/// width/height separately can leave the content up to half a pixel short
/// of the pixmap edge, which repeats as a 1px transparent seam between
/// tiles.
fn svg_surface_for(name: &'static str, h: f64) -> Option<gtk4::cairo::ImageSurface> {
    let tree = tree_for(name)?;
    let size = tree.size();
    let tile_w = size.width() as f64;
    let tile_h = size.height() as f64;

    let height = h.round().max(1.0) as u32;
    let width = ((tile_w / tile_h) * f64::from(height)).round().max(1.0) as u32;
    let scale_x = (f64::from(width) / tile_w) as f32;
    let scale_y = (f64::from(height) / tile_h) as f32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
    resvg::render(
        tree,
        resvg::tiny_skia::Transform::from_scale(scale_x, scale_y),
        &mut pixmap.as_mut(),
    );

    let mut argb = Vec::with_capacity(pixmap.data().len());
    for chunk in pixmap.data().as_chunks::<4>().0 {
        argb.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
    }

    gtk4::cairo::ImageSurface::create_for_data(
        argb,
        gtk4::cairo::Format::ARgb32,
        width as i32,
        height as i32,
        (width as i32) * 4,
    )
    .ok()
}

fn fill_svg_tile(cr: &gtk4::cairo::Context, name: &'static str, x0: f64, seg_w: f64, h: f64) {
    SVG_TILES.with(|cache| {
        let key = (name, h.round().max(1.0) as i32);
        let mut cache = cache.borrow_mut();
        if !cache.contains_key(&key) && cache.len() >= SVG_TILES_CACHE_MAX {
            cache.clear();
        }
        let surface = match cache
            .entry(key)
            .or_insert_with(|| svg_surface_for(name, h))
            .as_ref()
        {
            Some(surface) => surface,
            None => return,
        };

        let pattern = gtk4::cairo::SurfacePattern::create(surface);
        pattern.set_extend(gtk4::cairo::Extend::Repeat);

        cr.rectangle(x0, 0.0, seg_w, h);
        cr.set_source(&pattern).expect("set svg tile source");
        cr.fill().expect("fill svg tile");
    });
}

pub fn draw_pattern_rect(
    cr: &gtk4::cairo::Context,
    pattern: &BarPattern,
    x0: f64,
    seg_w: f64,
    h: f64,
) {
    cr.rectangle(x0, 0.0, seg_w, h);
    cr.clip();
    let (name, _) = chronos::patterns::tile(*pattern);
    fill_svg_tile(cr, name, x0, seg_w, h);
    cr.reset_clip();
}

/// Builds a small non-interactive preview swatch of `pattern` for the toolbar.
pub fn build_pattern_swatch(pattern: BarPattern, width: i32, height: i32) -> gtk4::DrawingArea {
    let area = gtk4::DrawingArea::builder()
        .width_request(width)
        .height_request(height)
        .build();
    area.set_draw_func(move |_area, cr, w, h| {
        let w = w as f64;
        let h = h as f64;
        if w == 0.0 || h == 0.0 {
            return;
        }
        cr.set_source_rgb(0.95, 0.95, 0.95);
        cr.rectangle(0.0, 0.0, w, h);
        cr.fill().expect("fill swatch bg");
        draw_pattern_rect(cr, &pattern, 0.0, w, h);
        cr.set_source_rgb(0.6, 0.6, 0.6);
        cr.set_line_width(1.0);
        cr.rectangle(0.0, 0.0, w, h);
        cr.stroke().expect("stroke swatch border");
    });
    area
}

#[derive(Clone)]
pub struct ConditionBarHandle {
    area: gtk4::DrawingArea,
    slots: Rc<RefCell<Vec<Slot>>>,
    pattern: Rc<RefCell<BarPattern>>,
    interaction: Rc<RefCell<BarInteractionState>>,
}

impl ConditionBarHandle {
    pub fn widget(&self) -> gtk4::DrawingArea {
        self.area.clone()
    }

    /// Brings the bar in line with the model and repaints only if something
    /// that is actually drawn has changed.
    ///
    /// Ignored while an edge/slot drag is in progress: the drag mutates the
    /// local slots live, and an unrelated re-render must not snap the bar
    /// back under the user's pointer. The drag-end event produces its own
    /// render, which syncs the final value.
    pub fn sync(&self, condition: &ConditionBar) {
        if self.interaction.borrow().drag_target != DragTarget::None {
            return;
        }

        let mut dirty = false;
        {
            let mut slots = self.slots.borrow_mut();
            let same = slots.len() == condition.slots.len()
                && slots
                    .iter()
                    .zip(&condition.slots)
                    .all(|(a, b)| a.start_pct == b.start_pct && a.end_pct == b.end_pct);
            if !same {
                *slots = condition.slots.clone();
                dirty = true;
            }
        }
        {
            let mut pattern = self.pattern.borrow_mut();
            if *pattern != condition.pattern {
                *pattern = condition.pattern;
                dirty = true;
            }
        }
        if dirty {
            self.area.queue_draw();
        }
    }
}

pub fn build_interactive_condition_bar(
    id: u32,
    bar_type: BarType,
    condition: ConditionBar,
    width_request: i32,
    ctx: &AppContext,
) -> ConditionBarHandle {
    let drawing_area = gtk4::DrawingArea::builder()
        .hexpand(true)
        .width_request(width_request)
        .height_request(24)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    let draw_slots = Rc::new(RefCell::new(condition.slots.clone()));
    let draw_pattern = Rc::new(RefCell::new(condition.pattern));
    let interaction = Rc::new(RefCell::new(BarInteractionState::default()));

    // --- Draw Function ---
    let draw_slots_draw = draw_slots.clone();
    let draw_pattern_draw = draw_pattern.clone();
    let interaction_draw = interaction.clone();
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
        let state = interaction_draw.borrow();
        let current_tool = ctx_draw.tool.get();

        for (idx, slot) in slots.iter().enumerate() {
            let x0 = slot.start_x(w);
            let x1 = slot.end_x(w);
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
                draw_pattern_rect(cr, &pattern, x0, seg_w, h);
                cr.reset_clip();
            }

            current_tool.draw_slot_overlay(cr, &state, idx, x0, x1, h);
        }

        current_tool.draw_bar_overlay(cr, &state, w, h);
    });

    // --- Motion Controller ---
    let drawing_area_motion = drawing_area.clone();
    let draw_slots_motion = draw_slots.clone();
    let interaction_motion = interaction.clone();
    let ctx_motion = ctx.clone();

    let motion = gtk4::EventControllerMotion::new();

    motion.connect_motion(move |controller, x, y| {
        let w = controller.widget().unwrap().width() as f64;
        let slots = draw_slots_motion.borrow();
        let mut state = interaction_motion.borrow_mut();
        let current_tool = ctx_motion.tool.get();

        if current_tool.on_motion(&mut state, &slots, w, x, y) {
            drawing_area_motion.queue_draw();
        }
    });

    let interaction_leave = interaction.clone();
    let drawing_area_leave = drawing_area.clone();
    let ctx_leave = ctx.clone();
    motion.connect_leave(move |_controller| {
        let current_tool = ctx_leave.tool.get();
        current_tool.on_leave(&mut interaction_leave.borrow_mut());
        drawing_area_leave.queue_draw();
    });
    drawing_area.add_controller(motion);

    // --- Drag Controller ---
    let interaction_begin = interaction.clone();
    let draw_slots_begin = draw_slots.clone();
    let ctx_begin = ctx.clone();
    let drag = gtk4::GestureDrag::new();

    drag.connect_drag_begin(move |gesture, start_x, _start_y| {
        let w = gesture.widget().unwrap().width() as f64;
        let slots = draw_slots_begin.borrow();
        let mut state = interaction_begin.borrow_mut();
        ctx_begin
            .tool
            .get()
            .on_drag_begin(&mut state, &slots, w, start_x);
    });

    let interaction_update = interaction.clone();
    let drawing_area_update = drawing_area.clone();
    let draw_slots_update = draw_slots.clone();
    let ctx_update = ctx.clone();

    drag.connect_drag_update(move |gesture, offset_x, _offset_y| {
        let w = gesture.widget().unwrap().width() as f64;
        let start_x = gesture.start_point().map(|(x, _)| x);
        let mut state = interaction_update.borrow_mut();
        let mut slots = draw_slots_update.borrow_mut();
        if ctx_update
            .tool
            .get()
            .on_drag_update(&mut state, &mut slots, w, start_x, offset_x)
        {
            drawing_area_update.queue_draw();
        }
    });

    let interaction_end = interaction.clone();
    let ctx_end = ctx.clone();
    let drawing_area_end = drawing_area.clone();
    let bar_type_right = bar_type.clone();

    drag.connect_drag_end(move |gesture, offset_x, _offset_y| {
        let w = gesture.widget().unwrap().width() as f64;
        let start_x = gesture.start_point().map(|(x, _)| x).unwrap_or(0.0);
        let mut state = interaction_end.borrow_mut();
        let event =
            ctx_end
                .tool
                .get()
                .on_drag_end(&mut state, id, bar_type.clone(), w, start_x, offset_x);
        drawing_area_end.queue_draw();
        if let Some(ev) = event {
            dispatch_event(&ctx_end, ev);
        }
    });
    drawing_area.add_controller(drag);

    // --- Right-click: secondary action ---
    let ctx_right = ctx.clone();
    let right_click = gtk4::GestureClick::builder()
        .button(gdk::BUTTON_SECONDARY)
        .build();
    right_click.connect_released(move |_gesture, _n_press, _x, _y| {
        if let Some(ev) = ctx_right
            .tool
            .get()
            .on_secondary_click(id, bar_type_right.clone())
        {
            dispatch_event(&ctx_right, ev);
        }
    });
    drawing_area.add_controller(right_click);

    ConditionBarHandle {
        area: drawing_area,
        slots: draw_slots,
        pattern: draw_pattern,
        interaction,
    }
}

/// Defers `event` to the next idle tick so the core isn't re-entered from
/// inside a GTK signal handler, then runs the resulting effects.
fn dispatch_event(ctx: &AppContext, event: Event) {
    let ctx = ctx.clone();
    glib::idle_add_local_once(move || {
        let effects = ctx.core.process_event(event);
        crate::effects::process_effects(&ctx, effects);
    });
}
