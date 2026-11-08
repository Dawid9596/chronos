use serde::{Deserialize, Serialize};

use crate::history::History;
use crate::i18n::Language;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Slot {
    pub start_pct: f32,
    pub end_pct: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ConditionBar {
    pub pattern: BarPattern,
    pub slots: Vec<Slot>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BarPattern {
    VerticalHatch,
    DenseGrid,
    Bolus,
    Brick,
    Ceramics,
    Wood,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum StatusMessage {
    Info(String),
    Error(String),
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub document: ConservationDocument,
    pub next_id: u32,
    pub last_error: Option<StatusMessage>,
    pub is_exporting: bool,
    pub language: Language,
    pub current_file_path: Option<String>,
    pub history: History<ConservationDocument>,
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
                            pattern: BarPattern::VerticalHatch,
                            slots: vec![Slot {
                                start_pct: 0.0,
                                end_pct: 75.0,
                            }],
                        },
                        woven_trim: ConditionBar {
                            pattern: BarPattern::VerticalHatch,
                            slots: vec![Slot {
                                start_pct: 0.0,
                                end_pct: 75.0,
                            }],
                        },
                        layer_characteristics: "Compact clay".to_string(),
                        layer_thickness: "12 cm".to_string(),
                        chronological_phase: "Iron Age".to_string(),
                        dating_period: "500 BC".to_string(),
                        chrono_group: 0,
                        dating_group: 0,
                    },
                    ConservationEntry {
                        id: 2,
                        graphical_representation: ConditionBar {
                            pattern: BarPattern::VerticalHatch,
                            slots: vec![Slot {
                                start_pct: 0.0,
                                end_pct: 30.0,
                            }],
                        },
                        woven_trim: ConditionBar {
                            pattern: BarPattern::VerticalHatch,
                            slots: vec![Slot {
                                start_pct: 0.0,
                                end_pct: 10.0,
                            }],
                        },
                        layer_characteristics: "Ash layer".to_string(),
                        layer_thickness: "5 cm".to_string(),
                        chronological_phase: "Iron Age".to_string(),
                        dating_period: "400 BC".to_string(),
                        chrono_group: 0,
                        dating_group: 0,
                    },
                ],
            },
            next_id: 3,
            last_error: None,
            is_exporting: false,
            language: Language::English,
            current_file_path: None,
            history: History::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ConservationDocument {
    pub project_name: String,
    pub date: String,
    pub entries: Vec<ConservationEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ConservationEntry {
    pub id: u32,
    pub graphical_representation: ConditionBar,
    pub woven_trim: ConditionBar,
    pub layer_characteristics: String,
    pub layer_thickness: String,
    pub chronological_phase: String,
    pub dating_period: String,
    #[serde(default)]
    pub chrono_group: u32,
    #[serde(default)]
    pub dating_group: u32,
}
