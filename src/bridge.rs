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
    ItemId,
    TreatmentNotes,
    MaterialCategory,
    EpochOrLocation,
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
        new_percentage: f32,
    },
    CyclePattern {
        id: u32,
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
                    ExportPdfResponse::Success(_path) => {
                        model.last_error = None;
                    }
                    ExportPdfResponse::Error(err) => {
                        model.last_error = Some(err);
                    }
                }
                caps.render.render();
            }
            Event::UpdateConditionFill { id, new_percentage } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    entry.condition.fill_percentage = new_percentage;
                }
                caps.render.render();
            }
            Event::CyclePattern { id } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    entry.condition.pattern = match entry.condition.pattern {
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
                        EditableField::ItemId => entry.item_id = new_value,
                        EditableField::TreatmentNotes => entry.treatment_notes = new_value,
                        EditableField::MaterialCategory => entry.material_category = new_value,
                        EditableField::EpochOrLocation => entry.epoch_or_location = new_value,
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
                            EditableField::MaterialCategory => {
                                entry.material_category = new_value.clone()
                            }
                            EditableField::EpochOrLocation => {
                                entry.epoch_or_location = new_value.clone()
                            }
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
