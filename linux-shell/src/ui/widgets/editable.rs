use crate::context::AppContext;
use chronos::bridge::Event;
use gtk4::glib;
use gtk4::prelude::*;

pub fn create_editable_widget(
    initial_text: &str,
    ctx: &AppContext,
    on_commit: impl Fn(String) -> Event + 'static,
) -> gtk4::Widget {
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
                let new_text = entry.text().to_string();
                stack.set_visible_child_name("label");
                label.set_text(&new_text);
                let event = on_commit(new_text);

                let cdx_idle = ctx.clone();
                let event_idle = event.clone();
                glib::idle_add_local_once(move || {
                    let effects = cdx_idle.core.process_event(event_idle);
                    crate::effects::process_effects(&cdx_idle, effects);
                });
            }
        }
    };

    let commit_and_switch_connect = commit_and_switch.clone();
    entry.connect_activate(move |_| {
        commit_and_switch_connect();
    });

    let key_controller = gtk4::EventControllerKey::new();
    let entry_connect = entry.clone();
    let stack_connect = stack.clone();
    key_controller.connect_key_pressed(move |_controller, key, _keycode, _state| {
        if key == gtk4::gdk::Key::Escape {
            stack.set_visible_child_name("label");
            entry_connect.set_text(label.text().as_str());
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

    stack_connect.upcast()
}
