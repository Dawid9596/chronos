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
    pub graphical_representation: ConditionBar,
    pub woven_trim: ConditionBar,
    pub layer_characteristics: String,
    pub layer_thickness: String,
    pub chronological_phase: String,
    pub dating_period: String,
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
                        graphical_representation: ConditionBar {
                            fill_percentage: 75.0,
                            pattern: BarPattern::DiagonalHatch,
                        },
                        woven_trim: ConditionBar {
                            fill_percentage: 40.0,
                            pattern: BarPattern::Wavy,
                        },
                        layer_characteristics: "Compact clay".to_string(),
                        layer_thickness: "12 cm".to_string(),
                        chronological_phase: "Iron Age".to_string(),
                        dating_period: "500 BC".to_string(),
                    },
                    ConservationEntry {
                        id: 2,
                        graphical_representation: ConditionBar {
                            fill_percentage: 30.0,
                            pattern: BarPattern::CrossedOut,
                        },
                        woven_trim: ConditionBar {
                            fill_percentage: 10.0,
                            pattern: BarPattern::Solid,
                        },
                        layer_characteristics: "Ash layer".to_string(),
                        layer_thickness: "5 cm".to_string(),
                        chronological_phase: "Iron Age".to_string(),
                        dating_period: "400 BC".to_string(),
                    },
                ],
            },
            next_id: 3, // Start counting after the default entries
            last_error: None,
            is_exporting: false,
        }
    }
}
