//! Persistent user settings.
//!
//! | Platform | Location                                        |
//! |----------|-------------------------------------------------|
//! | Linux    | `$XDG_CONFIG_HOME/chronos/settings.json`        |
//! |          | (default `~/.config/chronos/settings.json`)     |
//! | Windows  | `%APPDATA%\Chronos\settings.json`               |

use crate::context::{CanvasParams, ColumnVisibility};
use chronos::context::{
    clamp_finite_range, sanitize_paper_size, COLUMN_RATIO_RANGE, PAGE_MARGIN_MM_RANGE,
    PADDING_MM_RANGE, ZERO_WIDTH_LINE_PT_RANGE,
};
use chronos::i18n::Language;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)] // missing fields fall back to defaults, so old files keep working
pub struct Settings {
    /// `"en"` or `"pl"`. `None` means core's default.
    pub language: Option<String>,
    /// Contains only non-default shortcuts, keyed by
    /// `Action::id()`, valued by a GTK accelerator name (e.g. `<Control>o`).
    pub shortcuts: BTreeMap<String, String>,
    pub columns: ColumnVisibility,
    pub canvas: CanvasParams,
    /// Attach the project to exported PDFs so they can be opened for editing.
    /// `None` means the default, which is on.
    pub embed_project_data: Option<bool>,
}

pub fn settings_path() -> Option<PathBuf> {
    let app_dir = if cfg!(windows) { "Chronos" } else { "chronos" };
    dirs::config_dir().map(|dir| dir.join(app_dir).join("settings.json"))
}

impl Settings {
    pub fn language_code(lang: Language) -> &'static str {
        match lang {
            Language::English => "en",
            Language::Polish => "pl",
        }
    }

    pub fn saved_language(&self) -> Option<Language> {
        match self.language.as_deref()? {
            "en" => Some(Language::English),
            "pl" => Some(Language::Polish),
            _ => None,
        }
    }

    /// Loads settings, doesn't fail: a missing file gives defaults, and an
    /// unreadable/corrupt file is moved aside, defaults apply.
    pub fn load() -> Self {
        let Some(path) = settings_path() else {
            return Self::default();
        };
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Self::default(),
            Err(e) => {
                eprintln!("settings: cannot read {}: {e}", path.display());
                return Self::default();
            }
        };
        match serde_json::from_str::<Settings>(&text) {
            Ok(mut settings) => {
                settings.sanitize();
                settings
            }
            Err(e) => {
                let backup = path.with_extension("json.bad");
                eprintln!(
                    "settings: {} is invalid ({e}); keeping it as {} and using defaults",
                    path.display(),
                    backup.display()
                );
                let _ = std::fs::rename(&path, &backup);
                Self::default()
            }
        }
    }

    /// Writes atomically
    pub fn save(&self) -> io::Result<()> {
        let path = settings_path()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no user config directory"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, &path)
    }

    fn sanitize(&mut self) {
        self.canvas.paper_size = sanitize_paper_size(&self.canvas.paper_size);
        self.canvas.page_margin_mm = clamp_finite_range(self.canvas.page_margin_mm, PAGE_MARGIN_MM_RANGE);
        self.canvas.cell_padding_mm = clamp_finite_range(self.canvas.cell_padding_mm, PADDING_MM_RANGE);
        self.canvas.bar_padding_mm = clamp_finite_range(self.canvas.bar_padding_mm, PADDING_MM_RANGE);
        self.canvas.zero_width_line_pt =
            clamp_finite_range(self.canvas.zero_width_line_pt, ZERO_WIDTH_LINE_PT_RANGE);
        for r in self.canvas.column_ratios.iter_mut() {
            *r = (*r).clamp(COLUMN_RATIO_RANGE.0, COLUMN_RATIO_RANGE.1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let mut s = Settings {
            language: Some("pl".into()),
            ..Settings::default()
        };
        s.shortcuts.insert("save".into(), "<Control>j".into());
        s.columns.flags[3] = false;
        s.canvas.page_margin_mm = 33.0;

        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.language.as_deref(), Some("pl"));
        assert_eq!(
            back.shortcuts.get("save").map(String::as_str),
            Some("<Control>j")
        );
        assert!(!back.columns.flags[3]);
        assert_eq!(back.canvas.page_margin_mm, 33.0);
    }

    #[test]
    fn partial_and_unknown_fields_are_tolerated() {
        let s: Settings =
            serde_json::from_str(r#"{ "language": "en", "some_future_field": 1 }"#).unwrap();
        assert_eq!(s.language.as_deref(), Some("en"));
        assert!(s.shortcuts.is_empty());
        assert!(s.columns.flags.iter().all(|&f| f));
        assert_eq!(
            s.canvas.page_margin_mm,
            CanvasParams::default().page_margin_mm
        );
    }

    #[test]
    fn embedding_is_on_unless_switched_off() {
        let s: Settings = serde_json::from_str("{}").unwrap();
        assert!(s.embed_project_data.unwrap_or(true));
        let s: Settings = serde_json::from_str(r#"{ "embed_project_data": false }"#).unwrap();
        assert!(!s.embed_project_data.unwrap_or(true));
    }

    #[test]
    fn out_of_range_values_are_clamped() {
        let mut s: Settings = serde_json::from_str(
            r#"{ "canvas": { "page_margin_mm": 9999, "cell_padding_mm": -5, "zero_width_line_pt": 99, "column_ratios": [0, 200, 5, 5, 5, 5, 5] } }"#,
        )
        .unwrap();
        s.sanitize();
        assert_eq!(s.canvas.page_margin_mm, 50.0);
        assert_eq!(s.canvas.cell_padding_mm, 0.0);
        assert_eq!(s.canvas.zero_width_line_pt, 5.0);
        assert_eq!(s.canvas.column_ratios[0], 1);
    }
}
