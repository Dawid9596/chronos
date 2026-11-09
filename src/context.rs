use serde::{Deserialize, Serialize};

/// Number of document table columns ("No.", Graphical, Woven, Characteristics,
/// Thickness, Chronological Phase, Dating Period). Every visible-colour flag
/// array and column-ratio array must have exactly this length.
pub const COLUMN_COUNT: usize = 7;

/// Allowed `(min, max)` for [`CanvasParams::page_margin_mm`].
pub const PAGE_MARGIN_MM_RANGE: (f64, f64) = (5.0, 50.0);

/// Allowed `(min, max)` in mm for [`CanvasParams::cell_padding_mm`] and
/// [`CanvasParams::bar_padding_mm`].
pub const PADDING_MM_RANGE: (f64, f64) = (0.0, 20.0);

/// Allowed `(min, max)` for [`CanvasParams::zero_width_line_pt`].
pub const ZERO_WIDTH_LINE_PT_RANGE: (f64, f64) = (0.1, 5.0);

/// Allowed `(min, max)` for each element of [`CanvasParams::column_ratios`].
pub const COLUMN_RATIO_RANGE: (u32, u32) = (1, 100);

/// Paper sizes recognized project- and settings-wide; also fills the dialog
/// dropdown. `us-letter` and `us-legal` are intentionally lowercase — the
/// Typst template uses the same tokens.
pub const PAPER_SIZES: [&str; 5] = ["A4", "A3", "A5", "us-letter", "us-legal"];

/// Sanitized value for an unrecognized paper size.
pub const DEFAULT_PAPER_SIZE: &str = "A4";

/// Clamps `value` into `range`. Non-finite values (JSON `NaN`/`Infinity`)
/// collapse to the range minimum instead of poisoning layout calculations.
pub fn clamp_finite_range(value: f64, range: (f64, f64)) -> f64 {
    if value.is_finite() {
        value.clamp(range.0, range.1)
    } else {
        range.0
    }
}

/// Maps a raw paper size from a file onto the [`PAPER_SIZES`] whitelist: a
/// known size (case-insensitive) keeps its own spelling, anything else falls
/// back to [`DEFAULT_PAPER_SIZE`]. The shared sanitizer used by project files
/// and the settings file alike.
pub fn sanitize_paper_size(size: &str) -> String {
    let trimmed = size.trim();
    let known = PAPER_SIZES.iter().any(|s| s.eq_ignore_ascii_case(trimmed));
    if known {
        trimmed.to_string()
    } else {
        DEFAULT_PAPER_SIZE.to_string()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CanvasParams {
    pub paper_size: String,
    pub landscape: bool,
    pub page_margin_mm: f64,
    pub cell_padding_mm: f64,
    pub bar_padding_mm: f64,
    pub column_ratios: [u32; COLUMN_COUNT],
    pub zero_width_line_pt: f64,
}

impl Default for CanvasParams {
    fn default() -> Self {
        Self {
            paper_size: "A4".to_string(),
            landscape: false,
            page_margin_mm: 20.0,
            cell_padding_mm: 2.0,
            bar_padding_mm: 2.0,
            column_ratios: [5, 35, 10, 20, 10, 10, 10],
            zero_width_line_pt: 1.5,
        }
    }
}
