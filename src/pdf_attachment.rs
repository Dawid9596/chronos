//! Handling data embedding and extration from PDF
//!
//! * [`attach_project_json`] adds the JSON to a PDF
//! * [`extract_project_json`] reads it back

use crate::i18n::Translations;
use crate::project::EMBEDDED_FILE_NAME;
use lopdf::{Dictionary, Document, Object, Stream, dictionary};
use std::borrow::Cow;
use std::fmt;
use std::path::Path;

const ATTACHMENT_DESCRIPTION: &str = "Chronos project data";

pub fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PdfProjectError {
    Unreadable(String),
    NoProjectData,
}

impl PdfProjectError {
    /// User-facing message in the given language. The `Unreadable` detail
    /// (`why`) stays untranslated, like the library errors it wraps.
    pub fn describe(&self, t: &Translations) -> Cow<'static, str> {
        match self {
            Self::Unreadable(why) => Cow::Owned(format!("{}{why}", t.error_pdf_unreadable_prefix)),
            Self::NoProjectData => Cow::Borrowed(t.error_no_project_data),
        }
    }
}

fn unreadable(e: impl fmt::Display) -> PdfProjectError {
    PdfProjectError::Unreadable(e.to_string())
}

/// Expects a PDF which has no attachments of its own (like from Typsy); an
/// existing attachment tree would be replaced.
pub fn attach_project_json(pdf_bytes: &[u8], json: &str) -> Result<Vec<u8>, PdfProjectError> {
    let mut doc = Document::load_mem(pdf_bytes).map_err(unreadable)?;

    let mut file = Stream::new(
        dictionary! {
            "Type" => "EmbeddedFile",
            "Subtype" => "application/json",
            "Params" => dictionary! { "Size" => json.len() as i64 },
        },
        json.as_bytes().to_vec(),
    );
    // Compression is an optimisation only; fall back to the raw bytes.
    let _ = file.compress();
    let file_id = doc.add_object(file);

    let spec_id = doc.add_object(dictionary! {
        "Type" => "Filespec",
        "F" => Object::string_literal(EMBEDDED_FILE_NAME),
        "UF" => Object::string_literal(EMBEDDED_FILE_NAME),
        "Desc" => Object::string_literal(ATTACHMENT_DESCRIPTION),
        "EF" => dictionary! {
            "F" => Object::Reference(file_id),
            "UF" => Object::Reference(file_id),
        },
    });

    // A one-entry name tree: /EmbeddedFiles << /Names [ (name) spec ] >>
    let tree = dictionary! {
        "Names" => vec![Object::string_literal(EMBEDDED_FILE_NAME), Object::Reference(spec_id)],
    };

    let catalog_id = doc
        .trailer
        .get(b"Root")
        .and_then(Object::as_reference)
        .map_err(unreadable)?;
    // The catalog's /Names (named destinations, ...) may be direct or indirect.
    let names = doc
        .get_dictionary(catalog_id)
        .map_err(unreadable)?
        .get(b"Names")
        .ok()
        .cloned();
    match names {
        Some(Object::Reference(names_id)) => {
            doc.get_dictionary_mut(names_id)
                .map_err(unreadable)?
                .set("EmbeddedFiles", tree);
        }
        Some(Object::Dictionary(mut names)) => {
            names.set("EmbeddedFiles", tree);
            doc.get_dictionary_mut(catalog_id)
                .map_err(unreadable)?
                .set("Names", names);
        }
        _ => {
            doc.get_dictionary_mut(catalog_id)
                .map_err(unreadable)?
                .set("Names", dictionary! { "EmbeddedFiles" => tree });
        }
    }

    let mut out = Vec::with_capacity(pdf_bytes.len() + json.len() + 512);
    doc.save_to(&mut out).map_err(unreadable)?;
    Ok(out)
}

pub fn extract_project_json(pdf_bytes: &[u8]) -> Result<String, PdfProjectError> {
    let doc = Document::load_mem(pdf_bytes).map_err(unreadable)?;

    for object in doc.objects.values() {
        let Object::Dictionary(spec) = object else {
            continue;
        };
        if !is_project_filespec(spec) {
            continue;
        }
        if let Some(bytes) = embedded_bytes(&doc, spec) {
            return String::from_utf8(bytes)
                .map_err(|e| PdfProjectError::Unreadable(format!("attachment is not text: {e}")));
        }
    }
    Err(PdfProjectError::NoProjectData)
}

fn is_project_filespec(spec: &Dictionary) -> bool {
    let is_filespec = matches!(
        spec.get(b"Type").and_then(Object::as_name),
        Ok(name) if name == b"Filespec"
    );
    // `UF` is the Unicode name, `F` the legacy one; writers set either or both.
    is_filespec
        && [&b"UF"[..], &b"F"[..]].iter().any(|key| {
            matches!(
                spec.get(key).and_then(Object::as_str),
                Ok(name) if name == EMBEDDED_FILE_NAME.as_bytes()
            )
        })
}

fn embedded_bytes(doc: &Document, spec: &Dictionary) -> Option<Vec<u8>> {
    let files = resolve(doc, spec.get(b"EF").ok()?)?.as_dict().ok()?;
    let Object::Stream(stream) = resolve(doc, files.get(b"F").ok()?)? else {
        return None;
    };

    Some(
        stream
            .decompressed_content()
            .unwrap_or_else(|_| stream.content.clone()),
    )
}

fn resolve<'a>(doc: &'a Document, object: &'a Object) -> Option<&'a Object> {
    doc.dereference(object).ok().map(|(_, resolved)| resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest valid PDF: one empty page.
    fn blank_pdf() -> Vec<u8> {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => vec![0.into(), 0.into(), 200.into(), 200.into()],
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![Object::Reference(page_id)],
                "Count" => 1,
            }),
        );
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => Object::Reference(pages_id),
        });
        doc.trailer.set("Root", Object::Reference(catalog_id));
        let mut out = Vec::new();
        doc.save_to(&mut out).unwrap();
        out
    }

    #[test]
    fn rejects_non_pdf_bytes() {
        assert!(matches!(
            extract_project_json(b"definitely not a pdf"),
            Err(PdfProjectError::Unreadable(_))
        ));
        assert!(matches!(
            attach_project_json(b"definitely not a pdf", "{}"),
            Err(PdfProjectError::Unreadable(_))
        ));
    }

    #[test]
    fn pdf_without_attachment_reports_no_project_data() {
        assert_eq!(
            extract_project_json(&blank_pdf()),
            Err(PdfProjectError::NoProjectData)
        );
    }

    #[test]
    fn attach_then_extract_round_trips_text_exactly() {
        let json = r#"{"project_name":"Łódź ąęł 😀 ✓","note":"line\nbreak \"quoted\""}"#;
        let with = attach_project_json(&blank_pdf(), json).unwrap();
        assert_eq!(extract_project_json(&with).unwrap(), json);
    }

    #[test]
    fn attaching_keeps_an_existing_names_dictionary() {
        // Typst emits /Names for anchors and the like; it must survive.
        let mut doc = Document::load_mem(&blank_pdf()).unwrap();
        let root = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
        doc.get_dictionary_mut(root)
            .unwrap()
            .set("Names", dictionary! { "Dests" => dictionary! {} });
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).unwrap();

        let with = attach_project_json(&bytes, "{}").unwrap();
        let doc = Document::load_mem(&with).unwrap();
        let root = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
        let names = doc
            .get_dictionary(root)
            .unwrap()
            .get(b"Names")
            .unwrap()
            .as_dict()
            .unwrap();
        assert!(names.has(b"Dests"));
        assert!(names.has(b"EmbeddedFiles"));
        assert_eq!(extract_project_json(&with).unwrap(), "{}");
    }
}
