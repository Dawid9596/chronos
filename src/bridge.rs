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
        item_id: String,
        new_percentage: f32,
    },
    CyclePattern {
        item_id: String,
    },
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
            Event::UpdateConditionFill {
                item_id,
                new_percentage,
            } => {
                if let Some(entry) = model
                    .document
                    .entries
                    .iter_mut()
                    .find(|e| e.item_id == item_id)
                {
                    entry.condition.fill_percentage = new_percentage;
                }
                caps.render.render();
            }
            Event::CyclePattern { item_id } => {
                if let Some(entry) = model
                    .document
                    .entries
                    .iter_mut()
                    .find(|e| e.item_id == item_id)
                {
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
