use crate::{
    context::{
        CanvasParams, COLUMN_COUNT,
    },
    i18n::Translations,
    storage::{ConditionBar, ConservationDocument, StatusMessage},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DocumentViewModel {
    pub project_name: String,
    pub rows: Vec<RowViewModel>,
    pub translations: Translations,
    pub visible_columns: [bool; COLUMN_COUNT],
    pub canvas_params: CanvasParams,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RowViewModel {
    pub id: u32,
    pub row_number: usize,
    pub graphical_representation: ConditionBar,
    pub woven_trim: ConditionBar,
    pub layer_characteristics: String,
    pub layer_thickness: String,
    pub chronological_phase: Option<GroupSpan>,
    pub dating_period: Option<GroupSpan>,
    pub is_last_in_group: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GroupSpan {
    pub value: String,
    pub span: usize,
    pub member_ids: Vec<u32>,
}

impl ConservationDocument {
    pub fn to_view_model(
        &self,
        translations: Translations,
        visible_columns: [bool; COLUMN_COUNT],
        canvas_params: CanvasParams,
    ) -> DocumentViewModel {
        let len = self.entries.len();
        let mut rows = Vec::with_capacity(len);
        if len == 0 {
            return DocumentViewModel {
                project_name: self.project_name.clone(),
                rows,
                translations,
                visible_columns,
                canvas_params,
            };
        }

        let mut chrono_spans = vec![1; len];
        let mut dating_spans = vec![1; len];

        for i in (1..len).rev() {
            let current = &self.entries[i];
            let previous = &self.entries[i - 1];
            if current.chronological_phase == previous.chronological_phase
                && current.chrono_group == previous.chrono_group
            {
                chrono_spans[i - 1] = chrono_spans[i] + 1;
                chrono_spans[i] = 0;
            }
            if current.dating_period == previous.dating_period
                && current.dating_group == previous.dating_group
            {
                dating_spans[i - 1] = dating_spans[i] + 1;
                dating_spans[i] = 0;
            }
        }

        for i in 0..len {
            let entry = &self.entries[i];

            let chronological_phase = if chrono_spans[i] > 0 {
                let span = chrono_spans[i];
                Some(GroupSpan {
                    value: entry.chronological_phase.clone(),
                    span,
                    member_ids: self.entries[i..i + span].iter().map(|e| e.id).collect(),
                })
            } else {
                None
            };

            let dating_period = if dating_spans[i] > 0 {
                let span = dating_spans[i];
                Some(GroupSpan {
                    value: entry.dating_period.clone(),
                    span,
                    member_ids: self.entries[i..i + span].iter().map(|e| e.id).collect(),
                })
            } else {
                None
            };

            let is_last_in_group = if i + 1 < len {
                let next = &self.entries[i + 1];
                (entry.chronological_phase != next.chronological_phase
                    || entry.chrono_group != next.chrono_group)
                    && (entry.dating_period != next.dating_period
                        || entry.dating_group != next.dating_group)
            } else {
                true
            };

            rows.push(RowViewModel {
                id: entry.id,
                row_number: i + 1,
                graphical_representation: entry.graphical_representation.clone(),
                woven_trim: entry.woven_trim.clone(),
                layer_characteristics: entry.layer_characteristics.clone(),
                layer_thickness: entry.layer_thickness.clone(),
                chronological_phase,
                dating_period,
                is_last_in_group,
            });
        }

        DocumentViewModel {
            project_name: self.project_name.clone(),
            rows,
            translations,
            visible_columns,
            canvas_params,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppViewModel {
    pub document: DocumentViewModel,
    pub last_error: Option<StatusMessage>,
    pub is_exporting: bool,
    pub translations: Translations,
    pub can_undo: bool,
    pub can_redo: bool,
}
