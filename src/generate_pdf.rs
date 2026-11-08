use crate::view::DocumentViewModel;
use crate::view_model::ConservationWorld;

pub fn typst_compile_to_bytes(view_model: &DocumentViewModel) -> Result<Vec<u8>, String> {
    let json_data = serde_json::to_string(view_model)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;

    let template_source = include_str!("../assets/template.typ").to_string();
    let world = ConservationWorld::new(template_source, json_data);

    let document = typst::compile(&world)
        .output
        .map_err(|err| format!("Typst compilation failed: {:?}", err))?;

    let pdf_bytes = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .map_err(|err| format!("PDF generation failed: {:?}", err))?;

    Ok(pdf_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CanvasParams;
    use crate::i18n::Translations;
    use crate::storage::AppState;

    /// The default document must compile end to end, including the embedded
    /// artwork attribution on the last page (guards `template.typ` changes).
    #[test]
    fn default_document_compiles_to_pdf_with_attribution() {
        let state = AppState::default();
        let view_model = state.document.to_view_model(
            Translations::get(state.language),
            [true; 7],
            CanvasParams::default(),
        );

        let bytes = typst_compile_to_bytes(&view_model).expect("default document compiles");
        assert!(bytes.starts_with(b"%PDF"), "output is a PDF");
    }
}
