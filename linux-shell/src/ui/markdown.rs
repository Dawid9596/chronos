use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const ABOUT_MD: &str = concat!(
    r#"# Conservation Editor (Chronos)

**Version:** "#,
    env!("CARGO_PKG_VERSION"),
    r#"

---

## Overview {#overview}

Conservation Editor is a stratigraphic layer and condition documentation tool.

- **Editor:** Manage documents in standard format including layer characteristics, thickness, chronological phases, and dating periods.
- **WYSIWYG:** Visually specify graphical representations.
- **PDF Generation**: Export reports to PDF. 

Need assistance? Open the [User Guide](doc:help) or jump directly to [Keyboard Shortcuts](doc:help#shortcuts).
"#
);

pub const HELP_MD: &str = r#"# Chronos Help & Documentation

Welcome to the Conservation Editor documentation. Use the internal links below to jump to a topic:

- [Working with Tools](#tools)
- [Managing Layers & Groups](#layers)
- [Saving & Exporting](#files)
- [Keyboard Shortcuts](#shortcuts)
- [About Chronos](doc:about)

---

## Working with Tools {#tools}

The top toolbar provides interactive tools for editing condition bars and cells:

- **Edit:** Drag slot edges to resize coverage, or click a bar to cycle through patterns. Right-click cycles backward.
- **Create:** Click inside a condition bar to insert a new slot at the cursor position.
- **Delete:** Click an existing slot inside a condition bar to remove it.
- **Paint:** Select one of the 29 patterns (Bolus, Brick, Ceramics, Wood, Textile, Metal Foil, Embroidery, Stone, Glue, Bone, Spray Varnish, Painting, Metal, Paper, Patina, Parchment, Whitewash, Leather, Stucco, Glaze, Plasterwork, Cardboard, Plastic, Mixed Thread, Varnish, Wax, Nonwoven, Mortar, Gold Powder) and click any bar to apply it.
- **Split:** Click the highlighted horizontal divider inside a merged cell to split the group at that boundary.
- **Duplicate:** Click a row number or condition bar to duplicate the entire stratigraphic layer.
- **Merge:** Choose **Merge Down** or **Merge Up** and click a Chronological Phase or Dating Period cell to merge it with its neighbor.

---

## Managing Layers & Groups {#layers}

- **Insert Layer:** Click the thin gap above, below, or between groups to insert a blank layer.
- **Reorder Layer:** Double-click and hold a row number in the **No.** column, then drag vertically to reorder.
- **Edit Text:** Double-click any text cell or the project title to edit inline. Press `Enter` to commit or `Escape` to cancel.

---

## Saving & Exporting {#files}

Projects are saved in `.json` format and can be exported directly to `.pdf` using the **File** menu or toolbar button. You can customize column visibility and bar dimensions under **Edit -> Output Canvas Parameters**.

---

## Keyboard Shortcuts {#shortcuts}
Defaults are
```
Ctrl + O          Open Project (.json)
Ctrl + S          Save Project
Ctrl + Shift + S  Save Project As...
Ctrl + E          Export to PDF
E                 Edit Tool
C                 Create Tool
D                 Delete Tool
P                 Paint Tool
S                 Split Tool
R                 Duplicate Tool
M                 Merge Tool
Double-Click      Inline Edit / Reorder Handle
Escape            Cancel Inline Edit
```
Return to [Top of Help](#top) or read [About Conservation Editor](doc:about).
"#;

type NavFn = Rc<RefCell<Option<Box<dyn Fn(&str)>>>>;

thread_local! {
    static ACTIVE_MD_WIN: RefCell<Option<(glib::WeakRef<gtk4::Window>, NavFn)>> = const { RefCell::new(None) };
}

/// Opens a window capable of rendering predetermined basic markdown with internal links.
pub fn show_markdown_window(
    parent: &gtk4::ApplicationWindow,
    initial_title: &str,
    initial_doc_id: &str,
) {
    // Reuse existing documentation window if one is already open
    let reused = ACTIVE_MD_WIN.with(|slot| {
        if let Some((weak_win, nav_fn)) = slot.borrow().as_ref()
            && let Some(win) = weak_win.upgrade() {
                if let Some(ref f) = *nav_fn.borrow() {
                    f(&format!("doc:{initial_doc_id}"));
                }
                win.set_title(Some(initial_title));
                win.present();
                return true;
            }
        false
    });
    if reused {
        return;
    }

    let mut pages: HashMap<String, (String, &'static str)> = HashMap::new();
    pages.insert("about".to_string(), ("About".to_string(), ABOUT_MD));
    pages.insert("help".to_string(), ("Help".to_string(), HELP_MD));
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
            && let Some((title, md_source)) = pages_clone.get(doc_id) {
                if let Some(w) = win_weak.upgrade() {
                    w.set_title(Some(title));
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
                    && let Some(content_box) = viewport.first_child() {
                        let mut child = content_box.first_child();
                        while let Some(w) = child {
                            if w.widget_name().as_str() == format!("anchor-{anchor_owned}") {
                                let point = gtk4::graphene::Point::new(0.0, 0.0);
                            if let Some(transformed) = w.compute_point(&content_box, &point) {
                                scrolled_page.vadjustment().set_value(transformed.y().max(0.0) as f64);
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
        && let Some(end) = text[start..].find('}') {
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

/// Converts inline markdown (`**bold**`, `*italic*`, `` `code` ``, `[label](href)`) into Pango markup
/// and connects `activate-link` for internal navigation.
fn build_rich_label(
    markdown_inline: &str,
    nav_fn: &NavFn,
) -> gtk4::Label {
    let markup = inline_markdown_to_pango(markdown_inline);
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

fn inline_markdown_to_pango(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Link: [text](url)
        if chars[i] == '['
            && let Some(close_bracket) = chars[i + 1..].iter().position(|&c| c == ']') {
                let close_idx = i + 1 + close_bracket;
                if close_idx + 1 < chars.len() && chars[close_idx + 1] == '('
                    && let Some(close_paren) = chars[close_idx + 2..].iter().position(|&c| c == ')')
                    {
                        let end_paren = close_idx + 2 + close_paren;
                        let link_text: String = chars[i + 1..close_idx].iter().collect();
                        let link_url: String = chars[close_idx + 2..end_paren].iter().collect();
                        out.push_str(&format!(
                            "<a href=\"{}\">{}</a>",
                            glib::markup_escape_text(&link_url),
                            glib::markup_escape_text(&link_text)
                        ));
                        i = end_paren + 1;
                        continue;
                    }
            }

        // Bold: **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*'
            && let Some(end) = find_double_delim(&chars, i + 2, '*') {
                let inner: String = chars[i + 2..end].iter().collect();
                out.push_str(&format!("<b>{}</b>", glib::markup_escape_text(&inner)));
                i = end + 2;
                continue;
            }

        // Inline code: `code`
        if chars[i] == '`'
            && let Some(rel_end) = chars[i + 1..].iter().position(|&c| c == '`') {
                let end = i + 1 + rel_end;
                let inner: String = chars[i + 1..end].iter().collect();
                out.push_str(&format!("<tt>{}</tt>", glib::markup_escape_text(&inner)));
                i = end + 1;
                continue;
            }

        // Italic: *text*
        if chars[i] == '*'
            && let Some(rel_end) = chars[i + 1..].iter().position(|&c| c == '*') {
                let end = i + 1 + rel_end;
                let inner: String = chars[i + 1..end].iter().collect();
                out.push_str(&format!("<i>{}</i>", glib::markup_escape_text(&inner)));
                i = end + 1;
                continue;
            }

        // Escape single character for Pango
        let ch_str = chars[i].to_string();
        out.push_str(&glib::markup_escape_text(&ch_str));
        i += 1;
    }

    out
}

fn find_double_delim(chars: &[char], start: usize, delim: char) -> Option<usize> {
    let mut idx = start;
    while idx + 1 < chars.len() {
        if chars[idx] == delim && chars[idx + 1] == delim {
            return Some(idx);
        }
        idx += 1;
    }
    None
}
