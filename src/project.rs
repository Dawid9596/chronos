//! The project file format
//!
//! ```json
//! {
//!   "version": 1,
//!   "document": { "project_name": "...", "date": "...", "entries": [...] },
//!   "export_settings": { "visible_columns": [...], "canvas": {...} }
//! }
//! ```
//!
//! * `version` is [`PROJECT_FORMAT_VERSION`]. Files written before the
//!   envelope existed are a bare document; they are still read, and report
//!   version `0`.
//! * Opening a file only ever takes the `document`. `export_settings` are
//!   there so the user can *choose* to import them (canvas dialog); they must
//!   never silently replace the current ones.
//! * Anything from a file is untrusted: settings are clamped to the ranges the
//!   UI allows, and unreadable settings are dropped instead of failing the
//!   whole file.

use crate::context::CanvasParams;
use crate::storage::ConservationDocument;
use serde::{Deserialize, Serialize};

/// Bump when the format changes in a way older readers cannot handle.
pub const PROJECT_FORMAT_VERSION: u32 = 1;

/// Name of the attachment inside an exported PDF. The Typst template embeds
/// the file under this name; which must be in sync
pub const EMBEDDED_FILE_NAME: &str = "chronos-project.json";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportSettings {
    pub visible_columns: [bool; 7],
    pub canvas: CanvasParams,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            visible_columns: [true; 7],
            canvas: CanvasParams::default(),
        }
    }
}

impl ExportSettings {
    pub fn sanitize(&mut self) {
        let canvas = &mut self.canvas;
        let paper = canvas.paper_size.trim();
        canvas.paper_size = match paper.to_ascii_lowercase().as_str() {
            "a4" | "a3" | "a5" | "us-letter" | "us-legal" => paper.to_string(),
            _ => "A4".to_string(),
        };
        canvas.page_margin_mm = clamp_finite(canvas.page_margin_mm, 5.0, 50.0);
        canvas.cell_padding_mm = clamp_finite(canvas.cell_padding_mm, 0.0, 20.0);
        canvas.zero_width_line_pt = clamp_finite(canvas.zero_width_line_pt, 0.1, 5.0);
        for ratio in canvas.column_ratios.iter_mut() {
            *ratio = (*ratio).clamp(1, 100);
        }
    }
}

fn clamp_finite(value: f64, min: f64, max: f64) -> f64 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        min
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectFile {
    pub version: u32,
    pub document: ConservationDocument,
    #[serde(default)]
    pub export_settings: Option<ExportSettings>,
}

#[derive(Deserialize)]
struct RawProject {
    version: u32,
    document: ConservationDocument,
    #[serde(default)]
    export_settings: Option<serde_json::Value>,
}

impl ProjectFile {
    #[must_use]
    pub fn new(document: ConservationDocument, export_settings: ExportSettings) -> Self {
        Self {
            version: PROJECT_FORMAT_VERSION,
            document,
            export_settings: Some(export_settings),
        }
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Read a project from JSON: the current envelope, or a legacy bare
    /// document. Errors are short English technical messages meant to follow a
    /// translated prefix such as `error_parse_prefix`.
    pub fn parse(json: &str) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;

        let is_envelope = value.get("version").is_some() && value.get("document").is_some();
        if !is_envelope {
            let document =
                serde_json::from_value::<ConservationDocument>(value).map_err(|e| e.to_string())?;
            return Ok(Self {
                version: 0,
                document,
                export_settings: None,
            });
        }

        let raw = serde_json::from_value::<RawProject>(value).map_err(|e| e.to_string())?;
        if raw.version > PROJECT_FORMAT_VERSION {
            return Err(format!(
                "this file was written by a newer version of the program \
                 (file format {}, supported up to {})",
                raw.version, PROJECT_FORMAT_VERSION
            ));
        }
        let export_settings = raw
            .export_settings
            .and_then(|v| serde_json::from_value::<ExportSettings>(v).ok())
            .map(|mut settings| {
                settings.sanitize();
                settings
            });
        Ok(Self {
            version: raw.version,
            document: raw.document,
            export_settings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::AppState;

    fn sample_settings() -> ExportSettings {
        let mut s = ExportSettings::default();
        s.visible_columns[3] = false;
        s.canvas.page_margin_mm = 33.0;
        s.canvas.column_ratios = [4, 30, 10, 25, 10, 11, 10];
        s
    }

    #[test]
    fn round_trips() {
        let doc = AppState::default().document;
        let project = ProjectFile::new(doc, sample_settings());
        let back = ProjectFile::parse(&project.to_json()).unwrap();
        assert_eq!(back, project);
        assert_eq!(back.version, PROJECT_FORMAT_VERSION);
    }

    #[test]
    fn legacy_bare_document_still_opens() {
        let doc = AppState::default().document;
        let legacy = serde_json::to_string(&doc).unwrap();
        let project = ProjectFile::parse(&legacy).unwrap();
        assert_eq!(project.document, doc);
        assert_eq!(project.version, 0);
        assert!(project.export_settings.is_none());
    }

    #[test]
    fn newer_format_is_refused_with_a_clear_message() {
        let doc = AppState::default().document;
        let mut v = serde_json::to_value(ProjectFile::new(doc, ExportSettings::default())).unwrap();
        v["version"] = serde_json::json!(PROJECT_FORMAT_VERSION + 1);
        let err = ProjectFile::parse(&v.to_string()).unwrap_err();
        assert!(err.contains("newer version"), "{err}");
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let doc = AppState::default().document;
        let mut v = serde_json::to_value(ProjectFile::new(doc.clone(), ExportSettings::default()))
            .unwrap();
        v["something_from_the_future"] = serde_json::json!({ "a": 1 });
        v["export_settings"]["canvas"]["future_option"] = serde_json::json!(true);
        let project = ProjectFile::parse(&v.to_string()).unwrap();
        assert_eq!(project.document, doc);
        assert!(project.export_settings.is_some());
    }

    #[test]
    fn damaged_settings_do_not_block_the_document() {
        let doc = AppState::default().document;
        let mut v = serde_json::to_value(ProjectFile::new(doc.clone(), ExportSettings::default()))
            .unwrap();
        v["export_settings"] = serde_json::json!({ "visible_columns": "nope" });
        let project = ProjectFile::parse(&v.to_string()).unwrap();
        assert_eq!(project.document, doc);
        assert!(project.export_settings.is_none());
    }

    #[test]
    fn missing_settings_are_none() {
        let doc = AppState::default().document;
        let mut v = serde_json::to_value(ProjectFile::new(doc, ExportSettings::default())).unwrap();
        v.as_object_mut().unwrap().remove("export_settings");
        assert!(ProjectFile::parse(&v.to_string()).unwrap().export_settings.is_none());
    }

    #[test]
    fn imported_settings_are_clamped() {
        let doc = AppState::default().document;
        let mut v = serde_json::to_value(ProjectFile::new(doc, ExportSettings::default())).unwrap();
        v["export_settings"]["canvas"]["page_margin_mm"] = serde_json::json!(9999);
        v["export_settings"]["canvas"]["cell_padding_mm"] = serde_json::json!(-5);
        v["export_settings"]["canvas"]["zero_width_line_pt"] = serde_json::json!(99);
        v["export_settings"]["canvas"]["paper_size"] = serde_json::json!("tabloid");
        v["export_settings"]["canvas"]["column_ratios"] = serde_json::json!([0, 200, 5, 5, 5, 5, 5]);
        let s = ProjectFile::parse(&v.to_string()).unwrap().export_settings.unwrap();
        assert_eq!(s.canvas.page_margin_mm, 50.0);
        assert_eq!(s.canvas.cell_padding_mm, 0.0);
        assert_eq!(s.canvas.zero_width_line_pt, 5.0);
        assert_eq!(s.canvas.paper_size, "A4");
        assert_eq!(s.canvas.column_ratios[0], 1);
        assert_eq!(s.canvas.column_ratios[1], 100);
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(ProjectFile::parse("not json").is_err());
        assert!(ProjectFile::parse("{\"hello\": 1}").is_err());
    }
}
