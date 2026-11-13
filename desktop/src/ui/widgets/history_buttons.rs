use crate::shortcuts::{Action, ShortcutRegistry};
use chronos::bridge::Event;
use chronos::i18n::Translations;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Image, Label, Orientation};
use std::rc::Rc;

pub struct HistoryButtons {
    root: GtkBox,
    undo: Button,
    redo: Button,
    undo_label: Label,
    redo_label: Label,
    menu_undo: Button,
    menu_redo: Button,
}

fn make_button(
    shortcuts: &ShortcutRegistry,
    t: &Translations,
    action: Action,
    label: &str,
    icon_name: &str,
    send: &Rc<dyn Fn(Event)>,
    event: Event,
) -> (Button, Label) {
    let btn_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let icon = Image::from_icon_name(icon_name);
    let text_label = Label::new(Some(label));
    btn_box.append(&icon);
    btn_box.append(&text_label);

    let button = Button::builder()
        .child(&btn_box)
        .focus_on_click(false)
        .build();

    let send = send.clone();
    button.connect_clicked(move |_| send(event.clone()));

    shortcuts.bind_named_tooltip(action, &button, t);
    (button, text_label)
}

impl HistoryButtons {
    pub fn new(
        shortcuts: &ShortcutRegistry,
        t: &Translations,
        send: Rc<dyn Fn(Event)>,
        menu_undo: Button,
        menu_redo: Button,
    ) -> Self {
        let (undo, undo_label) = make_button(
            shortcuts,
            t,
            Action::Undo,
            &t.menu_undo,
            "edit-undo-symbolic",
            &send,
            Event::Undo,
        );
        let (redo, redo_label) = make_button(
            shortcuts,
            t,
            Action::Redo,
            &t.menu_redo,
            "edit-redo-symbolic",
            &send,
            Event::Redo,
        );

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
            undo_label,
            redo_label,
            menu_undo,
            menu_redo,
        };
        // Initially nothing to undo.
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
        self.undo_label.set_text(&t.menu_undo);
        self.redo_label.set_text(&t.menu_redo);
    }

    pub fn set_labels_visible(&self, visible: bool) {
        self.undo_label.set_visible(visible);
        self.redo_label.set_visible(visible);
    }
}