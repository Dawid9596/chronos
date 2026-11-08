use crate::shortcuts::{Action, ShortcutRegistry};
use chronos::bridge::Event;
use chronos::i18n::Translations;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Orientation};
use std::rc::Rc;

pub struct HistoryButtons {
    root: GtkBox,
    undo: Button,
    redo: Button,
    menu_undo: Button,
    menu_redo: Button,
}

fn make_button(
    shortcuts: &ShortcutRegistry,
    t: &Translations,
    action: Action,
    label: &str,
    send: &Rc<dyn Fn(Event)>,
    event: Event,
) -> Button {
    let button = Button::with_label(label);
    // Must not steal focus from items when clicked
    button.set_focus_on_click(false);

    let send = send.clone();
    button.connect_clicked(move |_| send(event.clone()));

    shortcuts.bind_tooltip(action, &button, t);
    button
}

impl HistoryButtons {
    pub fn new(
        shortcuts: &ShortcutRegistry,
        t: &Translations,
        send: Rc<dyn Fn(Event)>,
        menu_undo: Button,
        menu_redo: Button,
    ) -> Self {
        let undo = make_button(shortcuts, t, Action::Undo, &t.menu_undo, &send, Event::Undo);
        let redo = make_button(shortcuts, t, Action::Redo, &t.menu_redo, &send, Event::Redo);

        let root = GtkBox::new(Orientation::Horizontal, 6);
        // Lives in the CenterBox end slot: hug the right edge, and stay level
        // with the tool row when a variant picker expands below it.
        root.set_halign(Align::End);
        root.set_valign(Align::Start);
        root.append(&undo);
        root.append(&redo);

        let this = Self {
            root,
            undo,
            redo,
            menu_undo,
            menu_redo,
        };
        // NInitially nothing to undo.
        this.sync(false, false);
        this
    }

    pub fn widget(&self) -> &GtkBox {
        &self.root
    }

    pub fn sync(&self, can_undo: bool, can_redo: bool) {
        self.undo.set_sensitive(can_undo);
        self.menu_undo.set_sensitive(can_undo);
        self.redo.set_sensitive(can_redo);
        self.menu_redo.set_sensitive(can_redo);
    }

    pub fn update_translations(&self, t: &Translations) {
        self.undo.set_label(&t.menu_undo);
        self.redo.set_label(&t.menu_redo);
    }
}
