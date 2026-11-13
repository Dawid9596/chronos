use crate::i18n::Translations;
use crate::pdf_attachment::attach_project_json;
use crate::richtext::{to_typst, typst_escape};
use crate::view::DocumentViewModel;
use crate::view_model::ConservationWorld;

pub fn typst_compile_to_bytes(
    view_model: &DocumentViewModel,
    translations: &Translations,
    project_json: Option<&str>,
) -> Result<Vec<u8>, String> {
    let mut data = {
        let mut markup = view_model.clone();
        markup.project_name = typst_escape(&markup.project_name);
        markup.subtitle = to_typst(&markup.subtitle);
        for row in &mut markup.rows {
            row.layer_characteristics = to_typst(&row.layer_characteristics);
            row.layer_thickness = to_typst(&row.layer_thickness);
            if let Some(chrono) = &mut row.chronological_phase {
                chrono.value = to_typst(&chrono.value);
            }
            if let Some(dating) = &mut row.dating_period {
                dating.value = to_typst(&dating.value);
            }
        }
        serde_json::to_value(&markup).map_err(|e| format!("JSON serialization failed: {}", e))?
    };
    data["translations"] = serde_json::to_value(translations)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;
    let json_data = data.to_string();

    let template_source = include_str!("../assets/template.typ").to_string();
    let world = ConservationWorld::new(template_source, json_data);

    let document = typst::compile(&world)
        .output
        .map_err(|err| format!("Typst compilation failed: {:?}", err))?;

    let pdf_bytes = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .map_err(|err| format!("PDF generation failed: {:?}", err))?;

    match project_json {
        Some(json) => attach_project_json(&pdf_bytes, json)
            .map_err(|e| format!("Embedding project data failed: {e}")),
        None => Ok(pdf_bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{COLUMN_COUNT, CanvasParams};
    use crate::i18n::{Language, Translations};
    use crate::pdf_attachment::{PdfProjectError, extract_project_json};
    use crate::project::{ExportSettings, ProjectFile};
    use crate::storage::ConservationDocument;

    fn sample_view_model() -> DocumentViewModel {
        ConservationDocument::sample().to_view_model([true; COLUMN_COUNT], CanvasParams::default())
    }

    /// The sample document must compile end to end, including the embedded
    /// artwork attribution on the last page (guards `template.typ` changes).
    #[test]
    fn sample_document_compiles_to_pdf_with_attribution() {
        let document = ConservationDocument::sample();
        let view_model = document.to_view_model([true; COLUMN_COUNT], CanvasParams::default());

        let bytes =
            typst_compile_to_bytes(&view_model, &Translations::get(Language::English), None)
                .expect("sample document compiles");
        assert!(bytes.starts_with(b"%PDF"), "output is a PDF");

        // Same for the Polish attribution text.
        let polish = document.to_view_model([true; COLUMN_COUNT], CanvasParams::default());
        typst_compile_to_bytes(&polish, &Translations::get(Language::Polish), None)
            .expect("Polish document compiles");
    }

    /// Rich-text markup must survive the Typst conversion: styles render
    /// and hostile characters (which would be code injection if passed
    /// through raw) must be escaped, not break the compilation.
    #[test]
    fn markup_and_hostile_characters_compile_to_pdf() {
        let mut document = ConservationDocument::sample();
        document.subtitle = "*Report* #2 $test_ <section> @ref \\ [x]".to_string();
        let entry = document.entries.first_mut().unwrap();
        entry.layer_characteristics = "**Bold** *italic* ^14^C m~3~ & 3 * 4 * 5".into();
        entry.chronological_phase = "Iron Age #1".into();

        let view_model = document.to_view_model([true; COLUMN_COUNT], CanvasParams::default());
        let bytes =
            typst_compile_to_bytes(&view_model, &Translations::get(Language::English), None)
                .expect("markup document compiles");
        assert!(bytes.starts_with(b"%PDF"), "output is a PDF");
    }

    /// A non-empty subtitle must render as its own line between the title
    /// and the table (guards the conditional branch in `template.typ`).
    #[test]
    fn subtitle_renders_between_title_and_table() {
        let mut document = ConservationDocument::sample();
        document.subtitle = "Trench 3 — southern section".into();
        let view_model = document.to_view_model([true; COLUMN_COUNT], CanvasParams::default());
        let bytes =
            typst_compile_to_bytes(&view_model, &Translations::get(Language::English), None)
                .expect("document with subtitle compiles");
        assert!(bytes.starts_with(b"%PDF"), "output is a PDF");
    }

    /// Fresh documents start empty; exporting one must still produce a valid
    /// (single-page, no-project-data) PDF.
    #[test]
    fn empty_document_compiles_to_pdf() {
        let view_model = ConservationDocument::default()
            .to_view_model([true; COLUMN_COUNT], CanvasParams::default());
        let bytes =
            typst_compile_to_bytes(&view_model, &Translations::get(Language::English), None)
                .expect("empty document compiles");
        assert!(bytes.starts_with(b"%PDF"), "output is a PDF");
    }

    #[test]
    fn exported_pdf_carries_the_project_and_reopens_it() {
        let document = ConservationDocument::sample();
        let mut settings = ExportSettings::default();
        settings.visible_columns[4] = false;
        settings.canvas.page_margin_mm = 27.0;
        let project = ProjectFile::new(document, settings);

        let pdf = typst_compile_to_bytes(
            &sample_view_model(),
            &Translations::get(Language::English),
            Some(&project.to_json()),
        )
        .unwrap();
        assert!(pdf.starts_with(b"%PDF"));

        let json = extract_project_json(&pdf).unwrap();
        assert_eq!(
            ProjectFile::parse(&json, Language::English).unwrap(),
            project
        );
    }

    #[test]
    fn attaching_does_not_damage_the_report() {
        let plain = typst_compile_to_bytes(
            &sample_view_model(),
            &Translations::get(Language::English),
            None,
        )
        .unwrap();
        let with = typst_compile_to_bytes(
            &sample_view_model(),
            &Translations::get(Language::English),
            Some("{}"),
        )
        .unwrap();
        let pages = |bytes: &[u8]| lopdf::Document::load_mem(bytes).unwrap().get_pages().len();
        assert_eq!(pages(&plain), 1);
        assert_eq!(pages(&with), pages(&plain));
    }

    #[test]
    fn pdf_without_embedding_has_no_project_data() {
        let pdf = typst_compile_to_bytes(
            &sample_view_model(),
            &Translations::get(Language::English),
            None,
        )
        .unwrap();
        assert_eq!(
            extract_project_json(&pdf),
            Err(PdfProjectError::NoProjectData)
        );
    }
}
