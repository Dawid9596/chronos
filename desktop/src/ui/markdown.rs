use chronos::i18n::Translations;
use gtk4::glib;
use gtk4::prelude::*;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

fn author_line() -> String {
    match APP_AUTHORS.split_once(" <") {
        Some((name, email)) if email.ends_with('>') => {
            format!("{name} — {}", &email[..email.len() - 1])
        }
        _ => APP_AUTHORS.to_string(),
    }
}

const ABOUT_PAGE: &str = "about";
const HELP_PAGE: &str = "help";

type NavFn = Rc<RefCell<Option<Box<dyn Fn(&str)>>>>;

thread_local! {
    static ACTIVE_MD_WIN: RefCell<Option<(glib::WeakRef<gtk4::Window>, NavFn)>> = const { RefCell::new(None) };
}

/// Opens a window capable of rendering predetermined basic markdown with internal links.
pub fn show_markdown_window(
    parent: &gtk4::ApplicationWindow,
    t: &Translations,
    initial_doc_id: &str,
) {
    let page_of = |doc_id: &str| -> (&str, &str) {
        // (page-id, window title)
        if doc_id == ABOUT_PAGE {
            (ABOUT_PAGE, t.doc_about_title)
        } else {
            (HELP_PAGE, t.doc_help_title)
        }
    };

    // Reuse existing documentation window if one is already open
    let reused = ACTIVE_MD_WIN.with(|slot| {
        if let Some((weak_win, nav_fn)) = slot.borrow().as_ref()
            && let Some(win) = weak_win.upgrade()
        {
            if let Some(ref f) = *nav_fn.borrow() {
                f(&format!("doc:{initial_doc_id}"));
            }
            win.set_title(Some(page_of(initial_doc_id).1));
            win.present();
            return true;
        }
        false
    });
    if reused {
        return;
    }

    let (_, initial_title) = page_of(initial_doc_id);
    let mut pages: HashMap<&'static str, (&'static str, Cow<'static, str>)> = HashMap::new();
    pages.insert(
        ABOUT_PAGE,
        (
            t.doc_about_title,
            Cow::Owned(
                t.about_markdown
                    .replace("{version}", APP_VERSION)
                    .replace("{authors}", &author_line()),
            ),
        ),
    );
    pages.insert(
        HELP_PAGE,
        (t.doc_help_title, Cow::Borrowed(t.help_markdown)),
    );
    let pages = Rc::new(pages);

    let win = gtk4::Window::builder()
        .transient_for(parent)
        .modal(false)
        .title(initial_title)
        .default_width(680)
        .default_height(540)
        .build();

    let scrolled = gtk4::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

    win.set_child(Some(&scrolled));

    let nav_fn: NavFn = Rc::new(RefCell::new(None));

    let win_weak = win.downgrade();
    let scrolled_clone = scrolled.clone();
    let pages_clone = pages.clone();
    let nav_fn_clone = nav_fn.clone();

    let navigate = move |target: &str| {
        let (doc_part, anchor_part) = if let Some(rest) = target.strip_prefix("doc:") {
            if let Some((doc, anchor)) = rest.split_once('#') {
                (Some(doc), Some(anchor))
            } else {
                (Some(rest), None)
            }
        } else if let Some(anchor) = target.strip_prefix('#') {
            (None, Some(anchor))
        } else {
            (Some(target), None)
        };

        if let Some(doc_id) = doc_part
            && let Some((title, md_source)) = pages_clone.get(doc_id)
        {
            if let Some(w) = win_weak.upgrade() {
                w.set_title(Some(*title));
            }
            let content_box = render_markdown(md_source, nav_fn_clone.clone());
            scrolled_clone.set_child(Some(&content_box));
            scrolled_clone.vadjustment().set_value(0.0);
        }

        if let Some(anchor_name) = anchor_part {
            let scrolled_page = scrolled_clone.clone();
            let anchor_owned = anchor_name.to_string();
            // Look up anchor in the current content box via widget name
            glib::idle_add_local_once(move || {
                if anchor_owned == "top" {
                    scrolled_page.vadjustment().set_value(0.0);
                    return;
                }
                if let Some(viewport) = scrolled_page.child()
                    && let Some(content_box) = viewport.first_child()
                {
                    let mut child = content_box.first_child();
                    while let Some(w) = child {
                        if w.widget_name().as_str() == format!("anchor-{anchor_owned}") {
                            let point = gtk4::graphene::Point::new(0.0, 0.0);
                            if let Some(transformed) = w.compute_point(&content_box, &point) {
                                scrolled_page
                                    .vadjustment()
                                    .set_value(transformed.y().max(0.0) as f64);
                            }
                            break;
                        }
                        child = w.next_sibling();
                    }
                }
            });
        }
    };

    *nav_fn.borrow_mut() = Some(Box::new(navigate));

    if let Some(ref f) = *nav_fn.borrow() {
        f(&format!("doc:{initial_doc_id}"));
    }

    ACTIVE_MD_WIN.with(|slot| {
        *slot.borrow_mut() = Some((win.downgrade(), nav_fn));
    });

    win.present();
}

fn render_markdown(markdown: &str, nav_fn: NavFn) -> gtk4::Box {
    let vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_top(20)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();
    vbox.set_widget_name("anchor-top");

    let mut in_code_block = false;
    let mut code_lines: Vec<String> = Vec::new();

    for raw_line in markdown.lines() {
        let line = raw_line.trim_end();

        if line.trim_start().starts_with("```") {
            if in_code_block {
                let code_label = gtk4::Label::builder()
                    .label(code_lines.join("\n"))
                    .halign(gtk4::Align::Fill)
                    .xalign(0.0)
                    .selectable(true)
                    .build();
                code_label.add_css_class("md-code");
                vbox.append(&code_label);
                code_lines.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            code_lines.push(raw_line.to_string());
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        if line.trim() == "---" {
            let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
            sep.set_margin_top(6);
            sep.set_margin_bottom(6);
            vbox.append(&sep);
            continue;
        }

        // Heading detection with optional `{#anchor}`
        if let Some((level, rest)) = parse_heading(line) {
            let (title_text, explicit_anchor) = extract_anchor(rest);
            let slug = explicit_anchor.unwrap_or_else(|| slugify(title_text));

            let lbl = build_rich_label(title_text, &nav_fn);
            lbl.add_css_class(match level {
                1 => "md-h1",
                2 => "md-h2",
                _ => "md-h3",
            });
            lbl.set_widget_name(&format!("anchor-{slug}"));
            vbox.append(&lbl);
            continue;
        }

        // Bullet item
        if let Some(item_text) = line.trim_start().strip_prefix("- ") {
            let row = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(8)
                .margin_start(8)
                .build();
            let bullet = gtk4::Label::builder()
                .label("•")
                .valign(gtk4::Align::Start)
                .build();
            let item_lbl = build_rich_label(item_text, &nav_fn);
            row.append(&bullet);
            row.append(&item_lbl);
            vbox.append(&row);
            continue;
        }

        // Standard paragraph
        let p_lbl = build_rich_label(line, &nav_fn);
        vbox.append(&p_lbl);
    }

    vbox
}

fn parse_heading(line: &str) -> Option<(u8, &str)> {
    if let Some(rest) = line.strip_prefix("### ") {
        Some((3, rest))
    } else if let Some(rest) = line.strip_prefix("## ") {
        Some((2, rest))
    } else if let Some(rest) = line.strip_prefix("# ") {
        Some((1, rest))
    } else {
        None
    }
}

fn extract_anchor(text: &str) -> (&str, Option<String>) {
    if let Some(start) = text.find("{#")
        && let Some(end) = text[start..].find('}')
    {
        let anchor = text[start + 2..start + end].trim().to_string();
        let clean = text[..start].trim();
        return (clean, Some(anchor));
    }
    (text.trim(), None)
}

fn slugify(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Converts a paragraph of inline markdown into a rich `GtkLabel` and
/// connects `activate-link` for internal navigation.
///
/// The inline conversion itself (including `**bold**`, `*italic*`, code,
/// links and escaping) lives in `chronos::richtext`.
fn build_rich_label(markdown_inline: &str, nav_fn: &NavFn) -> gtk4::Label {
    let markup = chronos::richtext::to_pango(markdown_inline);
    let label = gtk4::Label::builder()
        .label(&markup)
        .use_markup(true)
        .wrap(true)
        .xalign(0.0)
        .halign(gtk4::Align::Fill)
        .hexpand(true)
        .build();

    let nav_clone = nav_fn.clone();
    label.connect_activate_link(move |_label, uri| {
        if let Some(ref f) = *nav_clone.borrow() {
            f(uri);
        }
        glib::Propagation::Stop
    });

    label
}
