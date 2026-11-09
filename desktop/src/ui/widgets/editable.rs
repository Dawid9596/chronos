use crate::context::AppContext;
use chronos::bridge::Event;
use gtk4::glib;
use gtk4::prelude::*;

/// Handle to a live double-click-to-edit cell.
///
/// The table keeps one of these per text cell so that a state change which
/// only alters text can be pushed into the existing widget instead of
/// rebuilding the whole grid.
#[derive(Clone)]
pub struct EditableCell {
    stack: gtk4::Stack,
    label: gtk4::Label,
    entry: gtk4::Entry,
}

impl EditableCell {
    /// The widget to attach to the grid.
    pub fn widget(&self) -> gtk4::Widget {
        self.stack.clone().upcast()
    }

    /// Updates the displayed text from the model without disturbing the user.
    ///
    /// The label always follows the model. The entry is only touched while the
    /// cell is *not* being edited, so a re-render triggered by something else
    /// (e.g. the status-message timer) can never clobber what is being typed.
    pub fn set_text(&self, text: &str) {
        if self.label.text().as_str() != text {
            self.label.set_text(text);
        }
        let editing = self.stack.visible_child_name().as_deref() == Some("entry");
        if !editing && self.entry.text().as_str() != text {
            self.entry.set_text(text);
        }
    }
}

pub fn create_editable_cell(
    initial_text: &str,
    ctx: &AppContext,
    on_commit: impl Fn(String) -> Event + 'static,
) -> EditableCell {
    let on_commit = std::rc::Rc::new(on_commit);

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
    label.set_margin_start(2);

    let entry = gtk4::Entry::new();
    entry.set_text(initial_text);
    entry.set_has_frame(false);
    entry.set_valign(gtk4::Align::Center);
    entry.set_width_chars(5);

    stack.add_named(&label, Some("label"));
    stack.add_named(&entry, Some("entry"));
    stack.set_visible_child_name("label");

    let click_gesture = gtk4::GestureClick::new();
    click_gesture.set_button(1);

    let stack_click = stack.clone();
    let entry_click = entry.clone();
    click_gesture.connect_pressed(glib::clone!(move |_gesture, n_press, _x, _y| {
        if n_press == 2 {
            stack_click.set_visible_child_name("entry");
            entry_click.grab_focus();
            entry_click.select_region(0, -1);
        }
    }));
    stack.add_controller(click_gesture);

    let commit_and_switch = {
        let stack = stack.clone();
        let entry = entry.clone();
        let label = label.clone();
        let ctx = ctx.clone();
        let on_commit = on_commit.clone();

        move || {
            if stack.visible_child_name().as_deref() == Some("entry") {
                if let Some(root) = entry.root() {
                    root.set_focus(None::<&gtk4::Widget>);
                }
                let new_text = entry.text().to_string();
                stack.set_visible_child_name("label");
                label.set_text(&new_text);
                let event = on_commit(new_text);
                let ctx_idle = ctx.clone();
                let event_idle = event.clone();
                glib::idle_add_local_once(move || {
                    let effects = ctx_idle.core.process_event(event_idle);
                    crate::effects::process_effects(&ctx_idle, effects);
                });
            }
        }
    };

    let commit_and_switch_connect = commit_and_switch.clone();
    entry.connect_activate(move |entry| {
        if let Some(root) = entry.root() {
            root.set_focus(None::<&gtk4::Widget>);
        }
        commit_and_switch_connect();
    });

    let key_controller = gtk4::EventControllerKey::new();
    let entry_connect = entry.clone();
    let stack_connect = stack.clone();
    let label_connect = label.clone();
    key_controller.connect_key_pressed(move |_controller, key, _keycode, _state| {
        if key == gtk4::gdk::Key::Escape {
            if let Some(root) = entry_connect.root() {
                root.set_focus(None::<&gtk4::Widget>);
            }
            stack_connect.set_visible_child_name("label");
            entry_connect.set_text(label_connect.text().as_str());
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    entry.add_controller(key_controller);

    let focus_controller = gtk4::EventControllerFocus::new();
    focus_controller.connect_leave(move |_controller| {
        commit_and_switch();
    });
    entry.add_controller(focus_controller);

    EditableCell {
        stack,
        label,
        entry,
    }
}
