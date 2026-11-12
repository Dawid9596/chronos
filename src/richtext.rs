//! Hanles enriched text for Pango, Typst and plaintext.
//!
//! - [`to_pango`] produces Pango markup for GTK labels (on-screen display).
//! - [`to_typst`] produces Typst markup for the exported PDF, escaping every
//!   Typst-special character it does not emit itself, so user text can never
//!   inject Typst code into the document.
//! - [`typst_escape`] escapes a string *without* interpreting markup, for
//!   fields that are deliberately plain (e.g. the project name).
//!
//! Syntax: `**bold**`, `*italic*`, `^superscript^`, `~subscript~`,
//! `` `code` ``, `[text](href)` links, and backslash escapes (`\*`, `\\`,
//! …) for literal characters. Any other backslash is literal. Unterminated
//! or unpaired markers render literally. Styles nest (`**bold *italic***`).

/// A parsed inline-markup span.
#[derive(Clone, Debug, PartialEq)]
pub enum Span {
    /// Literal text, already unescaped.
    Text(String),
    Bold(Vec<Span>),
    Italic(Vec<Span>),
    Sup(Vec<Span>),
    Sub(Vec<Span>),
    /// Inline code; its content is literal by construction (a backtick
    /// cannot appear between the delimiters).
    Code(String),
    Link {
        text: String,
        href: String,
    },
}

/// Parses inline markup into spans. See the module docs for the syntax.
#[must_use]
pub fn parse(input: &str) -> Vec<Span> {
    let chars: Vec<char> = input.chars().collect();
    let mut spans = Vec::new();
    let mut text = String::new();
    let mut i = 0;

    let flush = |spans: &mut Vec<Span>, text: &mut String| {
        if !text.is_empty() {
            spans.push(Span::Text(std::mem::take(text)));
        }
    };

    while i < chars.len() {
        let c = chars[i];

        // Backslash escape: the next character is always literal.
        if c == '\\' && i + 1 < chars.len() {
            text.push(chars[i + 1]);
            i += 2;
            continue;
        }

        // Link: [text](href)
        if c == '['
            && let Some((link_text, href, next)) = parse_link(&chars, i)
        {
            flush(&mut spans, &mut text);
            spans.push(Span::Link {
                text: link_text,
                href,
            });
            i = next;
            continue;
        }

        // Bold: **text** (checked before the single-* italic).
        if c == '*' && chars.get(i + 1) == Some(&'*') {
            if let Some(end) = find_delim(&chars, i + 2, "**") {
                let inner: String = chars[i + 2..end].iter().collect();
                flush(&mut spans, &mut text);
                spans.push(Span::Bold(parse(&inner)));
                i = end + 2;
                continue;
            }
            // Unterminated `**`: emit the first `*` literally and let the
            // next iteration re-examine the second one.
            text.push('*');
            i += 1;
            continue;
        }

        // Inline code: `text` (checked before other single-char delimiters
        // so code content is never interpreted as markup).
        if c == '`'
            && let Some(end) = find_delim(&chars, i + 1, "`")
        {
            let inner: String = chars[i + 1..end].iter().collect();
            flush(&mut spans, &mut text);
            spans.push(Span::Code(inner));
            i = end + 1;
            continue;
        }

        // Italic: *text*
        if c == '*'
            && let Some(end) = find_delim(&chars, i + 1, "*")
        {
            let inner: String = chars[i + 1..end].iter().collect();
            flush(&mut spans, &mut text);
            spans.push(Span::Italic(parse(&inner)));
            i = end + 1;
            continue;
        }

        // Superscript: ^text^
        if c == '^'
            && let Some(end) = find_delim(&chars, i + 1, "^")
        {
            let inner: String = chars[i + 1..end].iter().collect();
            flush(&mut spans, &mut text);
            spans.push(Span::Sup(parse(&inner)));
            i = end + 1;
            continue;
        }

        // Subscript: ~text~
        if c == '~'
            && let Some(end) = find_delim(&chars, i + 1, "~")
        {
            let inner: String = chars[i + 1..end].iter().collect();
            flush(&mut spans, &mut text);
            spans.push(Span::Sub(parse(&inner)));
            i = end + 1;
            continue;
        }

        text.push(c);
        i += 1;
    }
    flush(&mut spans, &mut text);
    spans
}

/// Finds the start index of the next unescaped occurrence of `delim`.
fn find_delim(chars: &[char], from: usize, delim: &str) -> Option<usize> {
    let delim: Vec<char> = delim.chars().collect();
    let mut i = from;
    while i + delim.len() <= chars.len() {
        match chars[i] {
            // Skip over escape pairs; an escaped delimiter never closes.
            '\\' => i += 2,
            c if c == delim[0] && chars[i..i + delim.len()] == delim[..] => return Some(i),
            _ => i += 1,
        }
    }
    None
}

/// Parses `[text](href)` at `start`, returning the parts and the index past
/// the closing parenthesis.
fn parse_link(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    let close_bracket = chars[start + 1..].iter().position(|&c| c == ']')? + start + 1;
    if chars.get(close_bracket + 1) != Some(&'(') {
        return None;
    }
    let close_paren =
        chars[close_bracket + 2..].iter().position(|&c| c == ')')? + close_bracket + 2;
    let text: String = chars[start + 1..close_bracket].iter().collect();
    let href: String = chars[close_bracket + 2..close_paren].iter().collect();
    Some((text, href, close_paren + 1))
}

fn pango_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The characters that carry markup meaning in Typst markup mode; every
/// other character is emitted verbatim.
const TYPST_SPECIALS: [char; 11] = ['\\', '#', '*', '_', '`', '$', '[', ']', '<', '>', '@'];

/// Escapes Typst markup characters so the text renders verbatim.
#[must_use]
pub fn typst_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if TYPST_SPECIALS.contains(&c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn render_typst(spans: &[Span], out: &mut String) {
    for span in spans {
        match span {
            Span::Text(text) => out.push_str(&typst_escape(text)),
            Span::Bold(children) => {
                out.push('*');
                render_typst(children, out);
                out.push('*');
            }
            Span::Italic(children) => {
                out.push('_');
                render_typst(children, out);
                out.push('_');
            }
            Span::Sup(children) => {
                out.push_str("#super[");
                render_typst(children, out);
                out.push(']');
            }
            Span::Sub(children) => {
                out.push_str("#sub[");
                render_typst(children, out);
                out.push(']');
            }
            Span::Code(text) => {
                // The content cannot contain a backtick (the parser stops at
                // the first one), so wrapping is always well-formed.
                out.push('`');
                out.push_str(text);
                out.push('`');
            }
            // Links have no place in the exported table; render their text.
            Span::Link { text, .. } => out.push_str(&typst_escape(text)),
        }
    }
}

fn render_pango(spans: &[Span], out: &mut String) {
    for span in spans {
        match span {
            Span::Text(text) => out.push_str(&pango_escape(text)),
            Span::Bold(children) => {
                out.push_str("<b>");
                render_pango(children, out);
                out.push_str("</b>");
            }
            Span::Italic(children) => {
                out.push_str("<i>");
                render_pango(children, out);
                out.push_str("</i>");
            }
            Span::Sup(children) => {
                out.push_str("<sup>");
                render_pango(children, out);
                out.push_str("</sup>");
            }
            Span::Sub(children) => {
                out.push_str("<sub>");
                render_pango(children, out);
                out.push_str("</sub>");
            }
            Span::Code(text) => {
                out.push_str("<tt>");
                out.push_str(&pango_escape(text));
                out.push_str("</tt>");
            }
            Span::Link { text, href } => {
                out.push_str("<a href=\"");
                out.push_str(&pango_escape(href));
                out.push_str("\">");
                out.push_str(&pango_escape(text));
                out.push_str("</a>");
            }
        }
    }
}

/// Renders inline markup as Pango markup, for `GtkLabel`s with
/// `use_markup(true)`.
#[must_use]
pub fn to_pango(input: &str) -> String {
    let spans = parse(input);
    let mut out = String::with_capacity(input.len());
    render_pango(&spans, &mut out);
    out
}

/// Renders inline markup as Typst markup, for interpolation into the PDF
/// template. All Typst-special characters in the *user's* text are escaped.
#[must_use]
pub fn to_typst(input: &str) -> String {
    let spans = parse(input);
    let mut out = String::with_capacity(input.len());
    render_typst(&spans, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(input: &str) -> Vec<Span> {
        parse(input)
    }

    #[test]
    fn plain_text_round_trips() {
        assert_eq!(p("Compact clay"), vec![Span::Text("Compact clay".into())]);
        assert_eq!(to_pango("Compact clay"), "Compact clay");
        assert_eq!(to_typst("Compact clay"), "Compact clay");
    }

    #[test]
    fn styles_render_to_both_backends() {
        assert_eq!(to_pango("**bold**"), "<b>bold</b>");
        assert_eq!(to_pango("*italic*"), "<i>italic</i>");
        assert_eq!(to_pango("^14^C"), "<sup>14</sup>C");
        assert_eq!(to_pango("m~3~"), "m<sub>3</sub>");
        assert_eq!(to_pango("`code`"), "<tt>code</tt>");
        assert_eq!(to_typst("**bold**"), "*bold*");
        assert_eq!(to_typst("*italic*"), "_italic_");
        assert_eq!(to_typst("^14^C"), "#super[14]C");
        assert_eq!(to_typst("m~3~"), "m#sub[3]");
    }

    #[test]
    fn styles_nest() {
        assert_eq!(
            to_pango("**bold *both* end**"),
            "<b>bold <i>both</i> end</b>"
        );
        assert_eq!(to_typst("**bold ^1^**"), "*bold #super[1]*");
    }

    #[test]
    fn unterminated_markers_are_literal() {
        assert_eq!(to_pango("a * b"), "a * b");
        assert_eq!(to_pango("a **b"), "a **b");
        assert_eq!(
            to_pango("^cold^? no — ^cold"),
            "<sup>cold</sup>? no — ^cold"
        );
        assert_eq!(to_typst("a * b"), "a \\* b");
    }

    #[test]
    fn backslash_escapes_are_literal() {
        assert_eq!(to_pango(r"\*not italic\*"), "*not italic*");
        assert_eq!(to_pango(r"a\\b"), r"a\b");
        assert_eq!(to_pango(r"trailing\"), r"trailing\");
        // An escaped delimiter does not close its style, nor open another.
        assert_eq!(to_pango(r"*a \* b*"), "<i>a * b</i>");
        assert_eq!(to_pango(r"3 \* 4 \* 5"), "3 * 4 * 5");
        // Backslash escaping works in both backends.
        assert_eq!(to_typst(r"\*x\*"), r"\*x\*");
    }

    #[test]
    fn pango_output_is_escaped() {
        assert_eq!(to_pango("a & b < c > d"), "a &amp; b &lt; c &gt; d");
        // Even inside styles.
        assert_eq!(to_pango("**<&>**"), "<b>&lt;&amp;&gt;</b>");
    }

    #[test]
    fn typst_output_is_escaped() {
        assert_eq!(
            to_typst("#a $b _c *d [e] <f> @g"),
            "\\#a \\$b \\_c \\*d \\[e\\] \\<f\\> \\@g"
        );
        // An unpaired backtick stays literal.
        assert_eq!(to_typst("a ` b"), "a \\` b");
        // Backslash pairs round-trip: `\\` is a literal backslash, which
        // must be escaped again on output.
        assert_eq!(to_typst(r"a\\b"), r"a\\b");
    }

    #[test]
    fn typst_escape_does_not_parse_markup() {
        // The project name is plain: markup characters stay visible.
        assert_eq!(typst_escape("3 * 4 * 5"), "3 \\* 4 \\* 5");
        assert_eq!(typst_escape("*bold*"), "\\*bold\\*");
    }

    #[test]
    fn links_render_and_are_escaped() {
        assert_eq!(
            to_pango("[help](doc:help#top)"),
            "<a href=\"doc:help#top\">help</a>"
        );
        assert_eq!(
            to_pango("[a&b](x?y&z)"),
            "<a href=\"x?y&amp;z\">a&amp;b</a>"
        );
        // Links degrade to their text in the PDF.
        assert_eq!(to_typst("[help](doc:help)"), "help");
        // A bracket without a valid link part is literal.
        assert_eq!(to_pango("[just text"), "[just text");
        assert_eq!(to_pango("[no url] (nothing)"), "[no url] (nothing)");
    }

    #[test]
    fn empty_and_tiny_inputs() {
        assert_eq!(to_pango(""), "");
        assert_eq!(to_pango("****"), "<b></b>");
        assert_eq!(to_pango("***"), "***");
    }
}
