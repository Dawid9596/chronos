use serde::{Deserialize, Serialize};

use crate::storage::{ConditionBar, ConservationDocument};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentViewModel {
    pub project_name: String,
    pub rows: Vec<RowViewModel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RowViewModel {
    pub id: u32,
    pub item_id: String,
    pub condition: ConditionBar,
    pub treatment_notes: String,

    pub material_group: Option<GroupSpan>,
    pub location_group: Option<GroupSpan>,
    pub is_last_in_group: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupSpan {
    pub value: String,
    pub span: usize, // e.g., 2 means it merges this row and the next
    pub member_ids: Vec<u32>,
}

impl ConservationDocument {
    pub fn to_view_model(&self) -> DocumentViewModel {
        let len = self.entries.len();
        let mut rows = Vec::with_capacity(len);
        if len == 0 {
            return DocumentViewModel {
                project_name: self.project_name.clone(),
                rows,
            };
        }

        let mut mat_spans = vec![1; len];
        let mut loc_spans = vec![1; len];

        for i in (1..len).rev() {
            let current = &self.entries[i];
            let previous = &self.entries[i - 1];
            if current.material_category == previous.material_category {
                mat_spans[i - 1] = mat_spans[i] + 1;
                mat_spans[i] = 0;
            }
            if current.epoch_or_location == previous.epoch_or_location {
                loc_spans[i - 1] = loc_spans[i] + 1;
                loc_spans[i] = 0;
            }
        }

        for i in 0..len {
            let entry = &self.entries[i];

            let material_group = if mat_spans[i] > 0 {
                let span = mat_spans[i];
                Some(GroupSpan {
                    value: entry.material_category.clone(),
                    span,
                    member_ids: self.entries[i..i + span].iter().map(|e| e.id).collect(),
                })
            } else {
                None
            };

            let location_group = if loc_spans[i] > 0 {
                let span = loc_spans[i];
                Some(GroupSpan {
                    value: entry.epoch_or_location.clone(),
                    span,
                    member_ids: self.entries[i..i + span].iter().map(|e| e.id).collect(),
                })
            } else {
                None
            };

            let is_last_in_group = if i + 1 < len {
                let next = &self.entries[i + 1];
                entry.material_category != next.material_category
                    || entry.epoch_or_location != next.epoch_or_location
            } else {
                true
            };

            rows.push(RowViewModel {
                id: entry.id,
                item_id: entry.item_id.clone(),
                condition: entry.condition.clone(),
                treatment_notes: entry.treatment_notes.clone(),
                material_group,
                location_group,
                is_last_in_group,
            });
        }

        DocumentViewModel {
            project_name: self.project_name.clone(),
            rows,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppViewModel {
    pub document: DocumentViewModel,
    pub last_error: Option<String>,
    pub is_exporting: bool,
}
