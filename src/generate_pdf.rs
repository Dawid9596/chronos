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
