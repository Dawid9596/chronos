use crate::i18n::Translations;
use crate::storage::{
    AppState, BarPattern, ConditionBar, ConservationDocument, ConservationEntry, Slot,
};
use crate::view::AppViewModel;
use crux_core::Command;
use crux_core::capability::{CapabilityContext, Operation};
use crux_core::{App, macros::Effect, render::Render};
use crux_macros::Capability;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExportPdfRequest {
    pub file_name: String,
    pub pdf_bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ExportPdfResponse {
    Success(String),
    Error(String),
}

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
                let response = context.request_from_shell(request).await;
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotEdge {
    Start,
    End,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleDirection {
    Forward,
    Backward,
}

pub const PATTERN_ORDER: [BarPattern; 6] = [
    BarPattern::VerticalHatch,
    BarPattern::DenseGrid,
    BarPattern::Bolus,
    BarPattern::Brick,
    BarPattern::Ceramics,
    BarPattern::Wood,
];

/// The pattern one step from `current` in `direction`, wrapping at both ends.
fn cycled_pattern(current: &BarPattern, direction: CycleDirection) -> BarPattern {
    let len = PATTERN_ORDER.len();
    let idx = PATTERN_ORDER
        .iter()
        .position(|p| std::mem::discriminant(p) == std::mem::discriminant(current))
        .unwrap_or(0);
    let next = match direction {
        CycleDirection::Forward => (idx + 1) % len,
        CycleDirection::Backward => (idx + len - 1) % len,
    };
    PATTERN_ORDER[next].clone()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FilePickerRequest {
    OpenFile {
        title: String,
        filter_name: String,
    },
    SaveFile {
        path: Option<String>,
        content: String,
        title: String,
        filter_name: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FilePickerResponse {
    Opened { path: String, content: String },
    Saved { path: String },
    Cancelled,
    Error(String),
}

impl Operation for FilePickerRequest {
    type Output = FilePickerResponse;
}

#[derive(Capability)]
pub struct FilePicker<Ev> {
    context: CapabilityContext<FilePickerRequest, Ev>,
}

impl<Ev> FilePicker<Ev>
where
    Ev: 'static,
{
    pub fn new(context: CapabilityContext<FilePickerRequest, Ev>) -> Self {
        Self { context }
    }

    pub fn pick(
        &self,
        request: FilePickerRequest,
        on_complete: impl Fn(FilePickerResponse) -> Ev + Send + Sync + 'static,
    ) {
        self.context.spawn({
            let context = self.context.clone();
            async move {
                let response = context.request_from_shell(request).await;
                context.update_app(on_complete(response));
            }
        });
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    Init,
    RequestPdfExport([bool; 7]),
    PdfExportFinished(ExportPdfResponse),
    DeleteLayer(u32),
    DuplicateLayer(u32),
    InsertLayerAfter(Option<u32>),
    MoveLayer {
        id: u32,
        target_index: usize,
    },
    UpdateProjectName(String),
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
    CyclePattern {
        id: u32,
        bar_type: BarType,
        direction: CycleDirection,
    },
    SetPattern {
        id: u32,
        bar_type: BarType,
        pattern: BarPattern,
    },
    SplitGroupCell {
        tail_member_ids: Vec<u32>,
        field: EditableField,
    },
    AddSlot {
        id: u32,
        bar_type: BarType,
        at_pct: f32,
    },
    DeleteSlot {
        id: u32,
        bar_type: BarType,
        slot_index: usize,
    },
    UpdateSlotEdge {
        id: u32,
        bar_type: BarType,
        slot_index: usize,
        edge: SlotEdge,
        new_pct: f32,
    },
    ToggleLanguage,
    RequestOpen,
    RequestSave,
    RequestSaveAs,
    FileOpenResponse(FilePickerResponse),
    FileSaveResponse(FilePickerResponse),
    ClearStatus,
}

#[derive(Default)]
pub struct ConservationApp;

#[derive(Effect)]
pub struct Capabilities {
    pub render: Render<Event>,
    pub export_pdf: ExportPdf<Event>,
    pub file_picker: FilePicker<Event>,
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
                caps.render.render();
            }
            Event::RequestPdfExport(visible_columns) => {
                model.is_exporting = true;
                model.last_error = None;
                let t = Translations::get(model.language);
                let view_model = model.document.to_view_model(t, visible_columns);

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
                    ExportPdfResponse::Success(path) => model.last_error = Some(path),
                    ExportPdfResponse::Error(err) => model.last_error = Some(err),
                }
                caps.render.render();
            }
            Event::DeleteLayer(id) => {
                model.document.entries.retain(|e| e.id != id);
                caps.render.render();
            }
            Event::DuplicateLayer(id) => {
                if let Some(idx) = model.document.entries.iter().position(|e| e.id == id) {
                    let new_id = model.next_id;
                    model.next_id += 1;

                    let mut duplicated = model.document.entries[idx].clone();
                    duplicated.id = new_id;

                    model.document.entries.insert(idx + 1, duplicated);
                    caps.render.render();
                }
            }
            Event::InsertLayerAfter(after_id) => {
                let new_id = model.next_id;
                model.next_id += 1;
                let new_entry = ConservationEntry {
                    id: new_id,
                    graphical_representation: ConditionBar {
                        pattern: BarPattern::VerticalHatch,
                        slots: vec![],
                    },
                    woven_trim: ConditionBar {
                        pattern: BarPattern::VerticalHatch,
                        slots: vec![],
                    },
                    layer_characteristics: String::new(),
                    layer_thickness: String::new(),
                    chronological_phase: String::new(),
                    dating_period: String::new(),
                    chrono_group: 0,
                    dating_group: 0,
                };
                match after_id {
                    Some(id) => {
                        if let Some(idx) = model.document.entries.iter().position(|e| e.id == id) {
                            model.document.entries.insert(idx + 1, new_entry);
                        } else {
                            model.document.entries.push(new_entry);
                        }
                    }
                    None => {
                        model.document.entries.insert(0, new_entry);
                    }
                }
                caps.render.render();
            }
            Event::MoveLayer { id, target_index } => {
                if let Some(pos) = model.document.entries.iter().position(|e| e.id == id) {
                    let entry = model.document.entries.remove(pos);
                    let clamped_index = target_index.min(model.document.entries.len());
                    model.document.entries.insert(clamped_index, entry);
                    caps.render.render();
                }
            }
            Event::UpdateProjectName(new_name) => {
                let trimmed = new_name.trim();
                if !trimmed.is_empty() {
                    model.document.project_name = trimmed.to_string();
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
                        _ => {}
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
                                if entry.chronological_phase != new_value {
                                    entry.chronological_phase = new_value.clone();
                                    entry.chrono_group = 0;
                                }
                            }
                            EditableField::DatingPeriod => {
                                if entry.dating_period != new_value {
                                    entry.dating_period = new_value.clone();
                                    entry.dating_group = 0;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                caps.render.render();
            }
            Event::SplitGroupCell {
                tail_member_ids,
                field,
            } => {
                let new_group_id = model
                    .document
                    .entries
                    .iter()
                    .map(|e| match field {
                        EditableField::ChronologicalPhase => e.chrono_group,
                        EditableField::DatingPeriod => e.dating_group,
                        _ => 0,
                    })
                    .max()
                    .unwrap_or(0)
                    + 1;

                for entry in model.document.entries.iter_mut() {
                    if tail_member_ids.contains(&entry.id) {
                        match field {
                            EditableField::ChronologicalPhase => {
                                entry.chrono_group = new_group_id;
                            }
                            EditableField::DatingPeriod => {
                                entry.dating_group = new_group_id;
                            }
                            _ => {}
                        }
                    }
                }
                caps.render.render();
            }
            Event::CyclePattern {
                id,
                bar_type,
                direction,
            } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    let pattern = match bar_type {
                        BarType::GraphicalRepresentation => {
                            &mut entry.graphical_representation.pattern
                        }
                        BarType::WovenTrim => &mut entry.woven_trim.pattern,
                    };
                    *pattern = cycled_pattern(pattern, direction);
                }
                caps.render.render();
            }
            Event::SetPattern {
                id,
                bar_type,
                pattern,
            } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    let target = match bar_type {
                        BarType::GraphicalRepresentation => {
                            &mut entry.graphical_representation.pattern
                        }
                        BarType::WovenTrim => &mut entry.woven_trim.pattern,
                    };
                    *target = pattern;
                }
                caps.render.render();
            }
            Event::AddSlot {
                id,
                bar_type,
                at_pct,
            } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    let bar = match bar_type {
                        BarType::GraphicalRepresentation => &mut entry.graphical_representation,
                        BarType::WovenTrim => &mut entry.woven_trim,
                    };
                    const MIN_GAP_PCT: f32 = 2.0;
                    let clashes = bar.slots.iter().any(|s| {
                        at_pct >= s.start_pct - MIN_GAP_PCT && at_pct <= s.end_pct + MIN_GAP_PCT
                    });
                    if !clashes {
                        bar.slots.push(Slot {
                            start_pct: at_pct,
                            end_pct: at_pct,
                        });
                        bar.slots
                            .sort_by(|a, b| a.start_pct.partial_cmp(&b.start_pct).unwrap());
                    }
                }
                caps.render.render();
            }
            Event::DeleteSlot {
                id,
                bar_type,
                slot_index,
            } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    let bar = match bar_type {
                        BarType::GraphicalRepresentation => &mut entry.graphical_representation,
                        BarType::WovenTrim => &mut entry.woven_trim,
                    };
                    if slot_index < bar.slots.len() {
                        bar.slots.remove(slot_index);
                    }
                }
                caps.render.render();
            }
            Event::UpdateSlotEdge {
                id,
                bar_type,
                slot_index,
                edge,
                new_pct,
            } => {
                if let Some(entry) = model.document.entries.iter_mut().find(|e| e.id == id) {
                    let bar = match bar_type {
                        BarType::GraphicalRepresentation => &mut entry.graphical_representation,
                        BarType::WovenTrim => &mut entry.woven_trim,
                    };
                    let prev_end = if slot_index > 0 {
                        bar.slots[slot_index - 1].end_pct
                    } else {
                        0.0
                    };
                    let next_start = bar
                        .slots
                        .get(slot_index + 1)
                        .map(|s| s.start_pct)
                        .unwrap_or(100.0);
                    const MIN_GAP_PCT: f32 = 2.0;

                    if let Some(slot) = bar.slots.get_mut(slot_index) {
                        match edge {
                            SlotEdge::Start => {
                                slot.start_pct =
                                    new_pct.clamp(prev_end + MIN_GAP_PCT, slot.end_pct);
                            }
                            SlotEdge::End => {
                                slot.end_pct =
                                    new_pct.clamp(slot.start_pct, next_start - MIN_GAP_PCT);
                            }
                        }
                    }
                }
                caps.render.render();
            }
            Event::ToggleLanguage => {
                model.language = match model.language {
                    crate::i18n::Language::English => crate::i18n::Language::Polish,
                    crate::i18n::Language::Polish => crate::i18n::Language::English,
                };
                caps.render.render();
            }
            Event::RequestOpen => {
                let t = Translations::get(model.language);
                caps.file_picker.pick(
                    FilePickerRequest::OpenFile {
                        title: t.dialog_open_title,
                        filter_name: t.filter_json_name,
                    },
                    Event::FileOpenResponse,
                );
            }
            Event::RequestSave => {
                let content = serde_json::to_string(&model.document).unwrap_or_default();
                let t = Translations::get(model.language);
                caps.file_picker.pick(
                    FilePickerRequest::SaveFile {
                        path: model.current_file_path.clone(),
                        content,
                        title: t.dialog_save_title,
                        filter_name: t.filter_json_name,
                    },
                    Event::FileSaveResponse,
                );
            }
            Event::RequestSaveAs => {
                let content = serde_json::to_string(&model.document).unwrap_or_default();
                let t = Translations::get(model.language);
                caps.file_picker.pick(
                    FilePickerRequest::SaveFile {
                        path: None, // Force "Save As" by clearing path
                        content,
                        title: t.dialog_save_title,
                        filter_name: t.filter_json_name,
                    },
                    Event::FileSaveResponse,
                );
            }
            Event::FileOpenResponse(response) => {
                let t = Translations::get(model.language);
                match response {
                    FilePickerResponse::Opened { path, content } => {
                        match serde_json::from_str::<ConservationDocument>(&content) {
                            Ok(doc) => {
                                model.next_id =
                                    doc.entries.iter().map(|e| e.id).max().unwrap_or(0) + 1;
                                model.document = doc;
                                model.current_file_path = Some(path.clone());
                                model.last_error =
                                    Some(format!("{}{}", t.status_loaded_prefix, path));
                            }
                            Err(e) => {
                                model.last_error = Some(format!("{}{}", t.error_parse_prefix, e));
                            }
                        }
                    }
                    FilePickerResponse::Saved { .. } => {}
                    FilePickerResponse::Cancelled => {}
                    FilePickerResponse::Error(e) => {
                        model.last_error = Some(format!("{}{}", t.error_open_prefix, e));
                    }
                }
                caps.render.render();
            }
            Event::FileSaveResponse(response) => {
                let t = crate::i18n::Translations::get(model.language);
                match response {
                    FilePickerResponse::Saved { path } => {
                        model.current_file_path = Some(path.clone());
                        model.last_error = Some(format!("{}{}", t.status_saved_prefix, path));
                    }
                    FilePickerResponse::Opened { .. } => {}
                    FilePickerResponse::Cancelled => {}
                    FilePickerResponse::Error(e) => {
                        model.last_error = Some(format!("{}{}", t.error_save_prefix, e));
                    }
                }
                caps.render.render();
            }
            Event::ClearStatus => {
                model.last_error = None;
                caps.render.render();
            }
        }
        Command::done()
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let t = Translations::get(model.language);
        AppViewModel {
            document: model.document.to_view_model(t.clone(), [true; 7]),
            last_error: model.last_error.clone(),
            is_exporting: model.is_exporting,
            translations: t,
        }
    }
}
