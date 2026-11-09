use crate::pdf_attachment::attach_project_json;
use crate::view::DocumentViewModel;
use crate::view_model::ConservationWorld;

pub fn typst_compile_to_bytes(
    view_model: &DocumentViewModel,
    project_json: Option<&str>,
) -> Result<Vec<u8>, String> {
    let json_data = serde_json::to_string(view_model)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;

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
    use crate::context::{CanvasParams, COLUMN_COUNT};
    use crate::i18n::{Language, Translations};
    use crate::pdf_attachment::{PdfProjectError, extract_project_json};
    use crate::project::{ExportSettings, ProjectFile};
    use crate::storage::AppState;

    fn sample_view_model() -> DocumentViewModel {
        AppState::default().document.to_view_model(
            Translations::get(Language::English),
            [true; COLUMN_COUNT],
            CanvasParams::default(),
        )
    }

    /// The default document must compile end to end, including the embedded
    /// artwork attribution on the last page (guards `template.typ` changes).
    #[test]
    fn default_document_compiles_to_pdf_with_attribution() {
        let state = AppState::default();
        let view_model = state.document.to_view_model(
            Translations::get(state.language),
            [true; COLUMN_COUNT],
            CanvasParams::default(),
        );

        let bytes =
            typst_compile_to_bytes(&view_model, None).expect("default document compiles");
        assert!(bytes.starts_with(b"%PDF"), "output is a PDF");

        // Same for the Polish attribution text.
        let polish = state.document.to_view_model(
            Translations::get(Language::Polish),
            [true; COLUMN_COUNT],
            CanvasParams::default(),
        );
        typst_compile_to_bytes(&polish, None).expect("Polish document compiles");
    }

    #[test]
    fn exported_pdf_carries_the_project_and_reopens_it() {
        let document = AppState::default().document;
        let mut settings = ExportSettings::default();
        settings.visible_columns[4] = false;
        settings.canvas.page_margin_mm = 27.0;
        let project = ProjectFile::new(document, settings);

        let pdf = typst_compile_to_bytes(&sample_view_model(), Some(&project.to_json())).unwrap();
        assert!(pdf.starts_with(b"%PDF"));

        let json = extract_project_json(&pdf).unwrap();
        assert_eq!(ProjectFile::parse(&json).unwrap(), project);
    }

    #[test]
    fn attaching_does_not_damage_the_report() {
        let plain = typst_compile_to_bytes(&sample_view_model(), None).unwrap();
        let with = typst_compile_to_bytes(&sample_view_model(), Some("{}")).unwrap();
        let pages = |bytes: &[u8]| lopdf::Document::load_mem(bytes).unwrap().get_pages().len();
        assert_eq!(pages(&plain), 1);
        assert_eq!(pages(&with), pages(&plain));
    }

    #[test]
    fn pdf_without_embedding_has_no_project_data() {
        let pdf = typst_compile_to_bytes(&sample_view_model(), None).unwrap();
        assert_eq!(
            extract_project_json(&pdf),
            Err(PdfProjectError::NoProjectData)
        );
    }
}
