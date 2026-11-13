//! Opening project data from disk
//!
//! Data is stored in a Save file (JSON) or in a PDF this program
//! has embedded data in.

use chronos::i18n::Translations;
use chronos::pdf_attachment::{PdfProjectError, extract_project_json, is_pdf_path};
use gtk4::gio;
use std::borrow::Cow;
use std::path::Path;

#[derive(Debug)]
pub enum ReadProjectError {
    Io(String),
    Pdf(PdfProjectError),
}

impl ReadProjectError {
    pub fn describe(&self, t: &Translations) -> Cow<'_, str> {
        match self {
            Self::Io(e) => Cow::Borrowed(e),
            Self::Pdf(e) => e.describe(t),
        }
    }
}

pub fn read_project_text(path: &Path) -> Result<String, ReadProjectError> {
    if is_pdf_path(path) {
        let bytes = std::fs::read(path).map_err(|e| ReadProjectError::Io(e.to_string()))?;
        extract_project_json(&bytes).map_err(ReadProjectError::Pdf)
    } else {
        std::fs::read_to_string(path).map_err(|e| ReadProjectError::Io(e.to_string()))
    }
}

pub fn project_file_filters(name: &str) -> gio::ListStore {
    let filters = gio::ListStore::new::<gtk4::FileFilter>();
    let filter = gtk4::FileFilter::new();
    for ext in ["json", "pdf"] {
        filter.add_pattern(&format!("*.{ext}"));
        filter.add_suffix(ext);
    }
    filter.set_name(Some(name));
    filters.append(&filter);
    filters
}
