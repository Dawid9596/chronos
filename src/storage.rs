use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConservationDocument {
    pub project_name: String,
    pub date: String,
    pub entries: Vec<ConservationEntry>,
}

/// Represents a single un-merged row in the document
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConservationEntry {
    pub id: u32,
    pub item_id: String,
    pub condition: ConditionBar,
    pub treatment_notes: String,
    pub material_category: String,
    pub epoch_or_location: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConditionBar {
    /// 0.0 to 100.0
    pub fill_percentage: f32,
    pub pattern: BarPattern,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BarPattern {
    Solid,
    Wavy,
    CrossedOut,
    DiagonalHatch,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub document: ConservationDocument,
    pub next_id: u32,
    pub last_error: Option<String>,
    pub is_exporting: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            document: ConservationDocument {
                project_name: "Sample Conservation Project".to_string(),
                date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                entries: vec![
                    ConservationEntry {
                        id: 1,
                        item_id: "ART-001".to_string(),
                        condition: ConditionBar {
                            fill_percentage: 75.0,
                            pattern: BarPattern::DiagonalHatch,
                        },
                        treatment_notes: "Surface cleaning required".to_string(),
                        material_category: "Ceramic".to_string(),
                        epoch_or_location: "Site A, Layer 2".to_string(),
                    },
                    ConservationEntry {
                        id: 2,
                        item_id: "ART-002".to_string(),
                        condition: ConditionBar {
                            fill_percentage: 30.0,
                            pattern: BarPattern::CrossedOut,
                        },
                        treatment_notes: "Consolidation needed".to_string(),
                        material_category: "Ceramic".to_string(),
                        epoch_or_location: "Site A, Layer 2".to_string(),
                    },
                ],
            },
            next_id: 3, // Start counting after the default entries
            last_error: None,
            is_exporting: false,
        }
    }
}
