use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CanvasParams {
    pub paper_size: String,
    pub landscape: bool,
    pub page_margin_mm: f64,
    pub cell_padding_mm: f64,
    pub bar_padding_mm: f64,
    pub column_ratios: [u32; 7],
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
