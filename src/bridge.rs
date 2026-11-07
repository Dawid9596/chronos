use crate::context::CanvasParams;
use crate::i18n::Translations;
use crate::storage::StatusMessage;
use crate::storage::{
    AppState, BarPattern, ConditionBar, ConservationDocument, ConservationEntry, Slot,
};
use crate::view::AppViewModel;
use crux_core::Command;
use crux_core::capability::{CapabilityContext, Operation};
use crux_core::{App, macros::Effect, render::Render};
use crux_macros::Capability;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExportPdfRequest {
    pub file_name: String,
    pub view_model: crate::view::DocumentViewModel,
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
        view_model: crate::view::DocumentViewModel,
        on_complete: impl Fn(ExportPdfResponse) -> Ev + Send + Sync + 'static,
    ) {
        let request = ExportPdfRequest {
            file_name,
            view_model,
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeDirection {
    Down,
    Up,
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
    PATTERN_ORDER[next]
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
    RequestPdfExport([bool; 7], CanvasParams),
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
    MergeGroupCell {
        member_ids: Vec<u32>,
        field: EditableField,
        direction: MergeDirection,
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
            Event::RequestPdfExport(visible_columns, canvas_params) => {
                model.is_exporting = true;
                model.last_error = None;
                // Trigger an immediate UI render so the export button disables and shows "Exporting..."
                caps.render.render();

                let t = Translations::get(model.language);
                let view_model = model
                    .document
                    .to_view_model(t, visible_columns, canvas_params);
                let safe_stem: String = model
                    .document
                    .project_name
                    .chars()
                    .map(|c| {
                        if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect();
                let file_name = format!("{}.pdf", safe_stem.trim());

                caps.export_pdf
                    .export(file_name, view_model, Event::PdfExportFinished);
            }
            Event::PdfExportFinished(response) => {
                model.is_exporting = false;
                match response {
                    ExportPdfResponse::Success(path) => {
                        model.last_error = Some(StatusMessage::Info(path))
                    }
                    ExportPdfResponse::Error(err) => {
                        model.last_error = Some(StatusMessage::Error(err))
                    }
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
                                // Keep existing chrono_group so explicit splits are preserved
                                entry.chronological_phase = new_value.clone();
                            }
                            EditableField::DatingPeriod => {
                                // Keep existing dating_group so explicit splits are preserved
                                entry.dating_period = new_value.clone();
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
            Event::MergeGroupCell {
                member_ids,
                field,
                direction,
            } => {
                let len = model.document.entries.len();
                let first_pos = model
                    .document
                    .entries
                    .iter()
                    .position(|e| member_ids.contains(&e.id));
                let last_pos = model
                    .document
                    .entries
                    .iter()
                    .rposition(|e| member_ids.contains(&e.id));

                let neighbor_idx = match (direction, first_pos, last_pos) {
                    (MergeDirection::Up, Some(first), _) if first > 0 => Some(first - 1),
                    (MergeDirection::Down, _, Some(last)) if last + 1 < len => Some(last + 1),
                    _ => None,
                };

                if let (Some(first), Some(last), Some(n_idx)) = (first_pos, last_pos, neighbor_idx)
                {
                    // Allocate a fresh group ID so the merged span cannot accidentally
                    // coalesce with an adjacent group that happens to share the same text.
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

                    match field {
                        EditableField::ChronologicalPhase => {
                            let target_val =
                                model.document.entries[n_idx].chronological_phase.clone();
                            let target_group = model.document.entries[n_idx].chrono_group;

                            // Find only the contiguous slice of the neighbor group
                            let (mut range_start, mut range_end) = (n_idx, n_idx);
                            if n_idx < first {
                                while range_start > 0 {
                                    let prev = &model.document.entries[range_start - 1];
                                    if prev.chronological_phase == target_val
                                        && prev.chrono_group == target_group
                                    {
                                        range_start -= 1;
                                    } else {
                                        break;
                                    }
                                }
                                range_end = last;
                            } else {
                                while range_end + 1 < len {
                                    let next = &model.document.entries[range_end + 1];
                                    if next.chronological_phase == target_val
                                        && next.chrono_group == target_group
                                    {
                                        range_end += 1;
                                    } else {
                                        break;
                                    }
                                }
                                range_start = first;
                            }

                            let clicked_val =
                                model.document.entries[first].chronological_phase.clone();
                            let final_val = if !clicked_val.is_empty() {
                                clicked_val
                            } else {
                                target_val
                            };

                            for entry in &mut model.document.entries[range_start..=range_end] {
                                entry.chronological_phase = final_val.clone();
                                entry.chrono_group = new_group_id;
                            }
                        }
                        EditableField::DatingPeriod => {
                            let target_val = model.document.entries[n_idx].dating_period.clone();
                            let target_group = model.document.entries[n_idx].dating_group;

                            let (mut range_start, mut range_end) = (n_idx, n_idx);
                            if n_idx < first {
                                while range_start > 0 {
                                    let prev = &model.document.entries[range_start - 1];
                                    if prev.dating_period == target_val
                                        && prev.dating_group == target_group
                                    {
                                        range_start -= 1;
                                    } else {
                                        break;
                                    }
                                }
                                range_end = last;
                            } else {
                                while range_end + 1 < len {
                                    let next = &model.document.entries[range_end + 1];
                                    if next.dating_period == target_val
                                        && next.dating_group == target_group
                                    {
                                        range_end += 1;
                                    } else {
                                        break;
                                    }
                                }
                                range_start = first;
                            }

                            let clicked_val = model.document.entries[first].dating_period.clone();
                            let final_val = if !clicked_val.is_empty() {
                                clicked_val
                            } else {
                                target_val
                            };

                            for entry in &mut model.document.entries[range_start..=range_end] {
                                entry.dating_period = final_val.clone();
                                entry.dating_group = new_group_id;
                            }
                        }
                        _ => {}
                    }
                    caps.render.render();
                }
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
                            .sort_by(|a, b| a.start_pct.total_cmp(&b.start_pct));
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
                                model.last_error = Some(StatusMessage::Info(format!(
                                    "{}{}",
                                    t.status_loaded_prefix, path
                                )));
                            }
                            Err(e) => {
                                model.last_error = Some(StatusMessage::Error(format!(
                                    "{}{}",
                                    t.error_parse_prefix, e
                                )));
                            }
                        }
                    }
                    FilePickerResponse::Saved { .. } => {}
                    FilePickerResponse::Cancelled => {}
                    FilePickerResponse::Error(e) => {
                        model.last_error = Some(StatusMessage::Error(format!(
                            "{}{}",
                            t.error_open_prefix, e
                        )));
                    }
                }
                caps.render.render();
            }
            Event::FileSaveResponse(response) => {
                let t = crate::i18n::Translations::get(model.language);
                match response {
                    FilePickerResponse::Saved { path } => {
                        model.current_file_path = Some(path.clone());
                        model.last_error = Some(StatusMessage::Info(format!(
                            "{}{}",
                            t.status_saved_prefix, path
                        )));
                    }
                    FilePickerResponse::Opened { .. } => {}
                    FilePickerResponse::Cancelled => {}
                    FilePickerResponse::Error(e) => {
                        model.last_error = Some(StatusMessage::Error(format!(
                            "{}{}",
                            t.error_save_prefix, e
                        )));
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
            document: model
                .document
                .to_view_model(t.clone(), [true; 7], CanvasParams::default()),
            last_error: model.last_error.clone(),
            is_exporting: model.is_exporting,
            translations: t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{Language, Translations};
    use crux_core::Core;

    #[test]
    fn split_and_edit_group_preserves_boundary() {
        let core = Core::<ConservationApp>::default();
        // Default state has rows 1 and 2 merged under "Iron Age" (chrono_group = 0)
        let vm = core.view();
        assert_eq!(
            vm.document.rows[0]
                .chronological_phase
                .as_ref()
                .unwrap()
                .span,
            2
        );

        // 1. Split row 2 from row 1
        let _ = core.process_event(Event::SplitGroupCell {
            tail_member_ids: vec![2],
            field: EditableField::ChronologicalPhase,
        });
        let vm = core.view();
        assert_eq!(
            vm.document.rows[0]
                .chronological_phase
                .as_ref()
                .unwrap()
                .span,
            1
        );
        assert_eq!(
            vm.document.rows[1]
                .chronological_phase
                .as_ref()
                .unwrap()
                .span,
            1
        );

        // 2. Rename row 2 to "Bronze Age", then back to "Iron Age"
        let _ = core.process_event(Event::UpdateGroupText {
            member_ids: vec![2],
            field: EditableField::ChronologicalPhase,
            new_value: "Bronze Age".into(),
        });
        let _ = core.process_event(Event::UpdateGroupText {
            member_ids: vec![2],
            field: EditableField::ChronologicalPhase,
            new_value: "Iron Age".into(),
        });

        // Rows 1 and 2 must remain separate (span = 1 each) despite having identical text
        let vm = core.view();
        assert_eq!(
            vm.document.rows[0]
                .chronological_phase
                .as_ref()
                .unwrap()
                .span,
            1
        );
        assert_eq!(
            vm.document.rows[1]
                .chronological_phase
                .as_ref()
                .unwrap()
                .span,
            1
        );
    }

    #[test]
    fn merge_only_affects_contiguous_neighbor() {
        let core = Core::<ConservationApp>::default();
        // Insert a 3rd row at the bottom and set its phase to "Iron Age" (chrono_group = 0)
        let _ = core.process_event(Event::InsertLayerAfter(Some(2)));
        let _ = core.process_event(Event::UpdateGroupText {
            member_ids: vec![3],
            field: EditableField::ChronologicalPhase,
            new_value: "Iron Age".into(),
        });
        // Split row 2 and set it to "Middle Ages" so row 1 ("Iron Age") and row 3 ("Iron Age") are separated
        let _ = core.process_event(Event::SplitGroupCell {
            tail_member_ids: vec![2],
            field: EditableField::ChronologicalPhase,
        });
        let _ = core.process_event(Event::UpdateGroupText {
            member_ids: vec![2],
            field: EditableField::ChronologicalPhase,
            new_value: "Middle Ages".into(),
        });

        // Merge row 2 ("Middle Ages") UP into row 1 ("Iron Age")
        let _ = core.process_event(Event::MergeGroupCell {
            member_ids: vec![2],
            field: EditableField::ChronologicalPhase,
            direction: MergeDirection::Up,
        });

        let _t = Translations::get(Language::English);
        let vm = core.view().document;
        // Rows 1 & 2 should now be merged as "Middle Ages" (span = 2)
        assert_eq!(
            vm.rows[0].chronological_phase.as_ref().unwrap().value,
            "Middle Ages"
        );
        assert_eq!(vm.rows[0].chronological_phase.as_ref().unwrap().span, 2);
        // Row 3 (non-adjacent) must still be "Iron Age" (span = 1), untouched by the merge!
        assert_eq!(
            vm.rows[2].chronological_phase.as_ref().unwrap().value,
            "Iron Age"
        );
        assert_eq!(vm.rows[2].chronological_phase.as_ref().unwrap().span, 1);
    }
}
