use crate::storage::AppState;
use crate::view::AppViewModel;
use crux_core::Command;
use crux_core::capability::{CapabilityContext, Operation};
use crux_core::{App, macros::Effect, render::Render};
use crux_macros::Capability;
use serde::{Deserialize, Serialize};

/// The request sent from Core to Shell
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExportPdfRequest {
    pub file_name: String,
    pub pdf_bytes: Vec<u8>,
}

/// The response sent from Shell back to Core
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ExportPdfResponse {
    Success(String), // Returns the saved file path
    Error(String),
}

// The operation that binds our request and response types
impl Operation for ExportPdfRequest {
    type Output = ExportPdfResponse;
}

#[derive(Capability)]
pub struct ExportPdf<Ev> {
    context: CapabilityContext<ExportPdfRequest, Ev>,
}

impl<Ev> ExportPdf<Ev>
where
    Ev: 'static,
{
    pub fn new(context: CapabilityContext<ExportPdfRequest, Ev>) -> Self {
        Self { context }
    }

    /// The method called by the Core to trigger the export
    pub fn export(
        &self,
        file_name: String,
        pdf_bytes: Vec<u8>,
        on_complete: impl Fn(ExportPdfResponse) -> Ev + Send + Sync + 'static,
    ) {
        let request = ExportPdfRequest {
            file_name,
            pdf_bytes,
        };

        self.context.spawn({
            let context = self.context.clone();
            async move {
                // Yield the request to the Shell and await the response
                let response = context.request_from_shell(request).await;
                // Dispatch the Core event with the result
                context.update_app(on_complete(response));
            }
        });
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EditableField {
    LayerCharacteristics,
    LayerThickness,
    ChronologicalPhase,
    DatingPeriod,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BarType {
    GraphicalRepresentation,
    WovenTrim,
}

#[derive(Default)]
pub struct ConservationApp;

// The events the Core can process
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    Init,
    // User clicked "Export to PDF" in the UI
    RequestPdfExport,
    // The capability returned the result from the Shell
    PdfExportFinished(ExportPdfResponse),
    UpdateConditionFill {
        id: u32,
        bar_type: BarType,
        new_percentage: f32,
    },
    CyclePattern {
        id: u32,
        bar_type: BarType,
    },
    UpdateText {
        id: u32,
        field: EditableField,
        new_value: String,
    },
    UpdateGroupText {
        member_ids: Vec<u32>,
        field: EditableField,
        new_value: String,
    },
    DeleteLayer(u32),
    InsertLayerAfter(Option<u32>),
}

// The App's capabilities
#[derive(Effect)]
pub struct Capabilities {
    pub render: Render<Event>,
    pub export_pdf: ExportPdf<Event>,
}

impl App for ConservationApp {
    type Event = Event;
    type Model = AppState;
    type ViewModel = AppViewModel;
    type Capabilities = Capabilities;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
        caps: &Self::Capabilities,
    ) -> Command<Self::Effect, Self::Event> {
        match event {
            Event::Init => {
                caps.render.render(); // Just draw the UI, don't export
            }
            Event::RequestPdfExport => {
                model.is_exporting = true;
                model.last_error = None;

                let view_model = model.document.to_view_model();

                match crate::generate_pdf::typst_compile_to_bytes(&view_model) {
                    Ok(pdf_bytes) => {
                        let file_name = format!("{}.pdf", model.document.project_name);
                        caps.export_pdf
                            .export(file_name, pdf_bytes, Event::PdfExportFinished);
                    }
                    Err(err_msg) => {
                        model.is_exporting = false;
                        model.last_error = Some(err_msg);
                        caps.render.render();
                    }
                }
            }
            Event::PdfExportFinished(response) => {
                model.is_exporting = false;
                match response {
                    ExportPdfResponse::Success(_) => model.last_error = None,
                    ExportPdfResponse::Error(err) => model.last_error = Some(err),
                }
                caps.render.render();
            }
            Event::UpdateConditionFill {
                id,
                bar_type,
                new_percentage,
            } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    match bar_type {
                        BarType::GraphicalRepresentation => {
                            entry.graphical_representation.fill_percentage = new_percentage
                        }
                        BarType::WovenTrim => entry.woven_trim.fill_percentage = new_percentage,
                    }
                }
                caps.render.render();
            }
            Event::CyclePattern { id, bar_type } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    let pattern = match bar_type {
                        BarType::GraphicalRepresentation => {
                            &mut entry.graphical_representation.pattern
                        }
                        BarType::WovenTrim => &mut entry.woven_trim.pattern,
                    };
                    *pattern = match pattern {
                        crate::storage::BarPattern::Solid => {
                            crate::storage::BarPattern::DiagonalHatch
                        }
                        crate::storage::BarPattern::DiagonalHatch => {
                            crate::storage::BarPattern::CrossedOut
                        }
                        crate::storage::BarPattern::CrossedOut => crate::storage::BarPattern::Wavy,
                        crate::storage::BarPattern::Wavy => crate::storage::BarPattern::Solid,
                    };
                }
                caps.render.render();
            }
            Event::UpdateText {
                id,
                field,
                new_value,
            } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    match field {
                        EditableField::LayerCharacteristics => {
                            entry.layer_characteristics = new_value
                        }
                        EditableField::LayerThickness => entry.layer_thickness = new_value,
                        EditableField::ChronologicalPhase => entry.chronological_phase = new_value,
                        EditableField::DatingPeriod => entry.dating_period = new_value,
                    }
                }
                caps.render.render();
            }
            Event::UpdateGroupText {
                member_ids,
                field,
                new_value,
            } => {
                for entry in model.document.entries.iter_mut() {
                    if member_ids.contains(&entry.id) {
                        match field {
                            EditableField::ChronologicalPhase => {
                                entry.chronological_phase = new_value.clone()
                            }
                            EditableField::DatingPeriod => entry.dating_period = new_value.clone(),
                            _ => {}
                        }
                    }
                }
                caps.render.render();
            }
            Event::DeleteLayer(id) => {
                model.document.entries.retain(|e| e.id != id);
                caps.render.render();
            }
            Event::InsertLayerAfter(maybe_id) => {
                let insert_index = match maybe_id {
                    Some(id) => model
                        .document
                        .entries
                        .iter()
                        .position(|e| e.id == id)
                        .map(|i| i + 1)
                        .unwrap_or(0),
                    None => 0,
                };

                let new_entry = crate::storage::ConservationEntry {
                    id: model.next_id,
                    graphical_representation: crate::storage::ConditionBar {
                        fill_percentage: 50.0,
                        pattern: crate::storage::BarPattern::Solid,
                    },
                    woven_trim: crate::storage::ConditionBar {
                        fill_percentage: 50.0,
                        pattern: crate::storage::BarPattern::Solid,
                    },
                    layer_characteristics: "".to_string(),
                    layer_thickness: "".to_string(),
                    chronological_phase: "Unknown".to_string(),
                    dating_period: "Unknown".to_string(),
                };
                model.document.entries.insert(insert_index, new_entry);
                model.next_id += 1;
                caps.render.render();
            }
        }
        Command::done()
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        AppViewModel {
            document: model.document.to_view_model(),
            last_error: model.last_error.clone(),
            is_exporting: model.is_exporting,
        }
    }
}
