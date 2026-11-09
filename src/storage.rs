use serde::{Deserialize, Serialize};

use crate::history::History;
use crate::i18n::Language;

/// Minimum separation, in percent of the bar width, between the edges of two
/// neighbouring slots. Enforced by the core when inserting or resizing slots
/// and mirrored by the shell's live-drag preview; change in one place only.
pub const MIN_GAP_PCT: f32 = 2.0;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    /// Left edge, in percent of the bar's drawn width.
    pub start_pct: f32,
    /// Right edge, in percent of the bar's drawn width.
    pub end_pct: f32,
}

impl Slot {
    pub const PCT_MIN: f32 = 0.0;
    pub const PCT_MAX: f32 = 100.0;

    /// Absolute x position, across a bar of width `w`, of the slot's start
    /// edge. Inverse of [`Slot::pct_at`].
    #[must_use]
    pub fn start_x(self, w: f64) -> f64 {
        pct_to_x(self.start_pct, w)
    }

    /// Same as [`Slot::start_x`] for the slot's end edge.
    #[must_use]
    pub fn end_x(self, w: f64) -> f64 {
        pct_to_x(self.end_pct, w)
    }

    /// Inverse of [`Slot::start_x`]/[`Slot::end_x`]: the percentage position
    /// across a bar of width `w` for an absolute `x` (e.g. a pointer
    /// coordinate). Values beyond the bar clamp to the domain edges.
    #[must_use]
    pub fn pct_at(x: f64, w: f64) -> f32 {
        x_to_pct(x, w)
    }
}

/// Converts a slot-domain percentage (see [`Slot::PCT_MIN`]) into an absolute
/// x coordinate across a bar of width `w`.
#[must_use]
fn pct_to_x(pct: f32, w: f64) -> f64 {
    w * (f64::from(pct) / f64::from(Slot::PCT_MAX))
}

/// Inverse of [`pct_to_x`], clamped into the slot domain so a pointer
/// position beyond the bar maps to the bar edge.
#[must_use]
fn x_to_pct(x: f64, w: f64) -> f32 {
    (x / w * f64::from(Slot::PCT_MAX)).clamp(f64::from(Slot::PCT_MIN), f64::from(Slot::PCT_MAX))
        as f32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ConditionBar {
    pub pattern: BarPattern,
    pub slots: Vec<Slot>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BarPattern {
    Bolus,
    Brick,
    Ceramics,
    Wood,
    Textile,
    MetalFoil,
    Embroidery,
    Stone,
    Glue,
    Bone,
    SprayVarnish,
    Painting,
    Metal,
    Paper,
    Patina,
    Parchment,
    Whitewash,
    Leather,
    Stucco,
    Glaze,
    Plasterwork,
    Cardboard,
    Plastic,
    MixedThread,
    Varnish,
    Wax,
    Nonwoven,
    Mortar,
    GoldPowder,
}

impl BarPattern {
    /// All patterns in declaration order — THE canonical enumeration.
    pub const ALL: [BarPattern; 29] = [
        Self::Bolus,
        Self::Brick,
        Self::Ceramics,
        Self::Wood,
        Self::Textile,
        Self::MetalFoil,
        Self::Embroidery,
        Self::Stone,
        Self::Glue,
        Self::Bone,
        Self::SprayVarnish,
        Self::Painting,
        Self::Metal,
        Self::Paper,
        Self::Patina,
        Self::Parchment,
        Self::Whitewash,
        Self::Leather,
        Self::Stucco,
        Self::Glaze,
        Self::Plasterwork,
        Self::Cardboard,
        Self::Plastic,
        Self::MixedThread,
        Self::Varnish,
        Self::Wax,
        Self::Nonwoven,
        Self::Mortar,
        Self::GoldPowder,
    ];

    #[must_use]
    pub fn display_name(self, lang: Language) -> &'static str {
        match lang {
            Language::English => match self {
                Self::Bolus => "Bolus",
                Self::Brick => "Brick",
                Self::Ceramics => "Ceramics",
                Self::Wood => "Wood",
                Self::Textile => "Textile",
                Self::MetalFoil => "Metal foil",
                Self::Embroidery => "Embroidery",
                Self::Stone => "Stone",
                Self::Glue => "Glue",
                Self::Bone => "Bone",
                Self::SprayVarnish => "Spray varnish",
                Self::Painting => "Painting",
                Self::Metal => "Metal",
                Self::Paper => "Paper",
                Self::Patina => "Patina",
                Self::Parchment => "Parchment",
                Self::Whitewash => "Whitewash",
                Self::Leather => "Leather",
                Self::Stucco => "Stucco",
                Self::Glaze => "Glaze",
                Self::Plasterwork => "Plasterwork",
                Self::Cardboard => "Cardboard",
                Self::Plastic => "Plastic",
                Self::MixedThread => "Mixed thread",
                Self::Varnish => "Varnish",
                Self::Wax => "Wax",
                Self::Nonwoven => "Nonwoven",
                Self::Mortar => "Mortar",
                Self::GoldPowder => "Gold powder",
            },
            Language::Polish => match self {
                Self::Bolus => "Bolus",
                Self::Brick => "Cegła",
                Self::Ceramics => "Ceramika",
                Self::Wood => "Drewno",
                Self::Textile => "Tkanina",
                Self::MetalFoil => "Folie metalowe",
                Self::Embroidery => "Haft",
                Self::Stone => "Kamień",
                Self::Glue => "Klej",
                Self::Bone => "Kość",
                Self::SprayVarnish => "Laserunek",
                Self::Painting => "Malowidło",
                Self::Metal => "Metal",
                Self::Paper => "Papier",
                Self::Patina => "Patyna",
                Self::Parchment => "Pergamin",
                Self::Whitewash => "Pobiała",
                Self::Leather => "Skóra",
                Self::Stucco => "Stiuk",
                Self::Glaze => "Szkliwo",
                Self::Plasterwork => "Sztukateria",
                Self::Cardboard => "Tektura",
                Self::Plastic => "Tworzywo sztuczne",
                Self::MixedThread => "Wątek mieszany",
                Self::Varnish => "Werniks",
                Self::Wax => "Wosk",
                Self::Nonwoven => "Włóknina",
                Self::Mortar => "Zaprawa",
                Self::GoldPowder => "Złoto w proszku",
            },
        }
    }
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
                            pattern: BarPattern::Bolus,
                            slots: vec![Slot {
                                start_pct: 0.0,
                                end_pct: 75.0,
                            }],
                        },
                        woven_trim: ConditionBar {
                            pattern: BarPattern::Bolus,
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
                            pattern: BarPattern::Bolus,
                            slots: vec![Slot {
                                start_pct: 0.0,
                                end_pct: 30.0,
                            }],
                        },
                        woven_trim: ConditionBar {
                            pattern: BarPattern::Bolus,
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

#[cfg(test)]
mod tests {
    use super::{BarPattern, Slot};

    /// `BarPattern::ALL` stands for the whole enum: entry `i` must be the
    /// `i`-th declared variant. Catches a re-ordered or duplicated list;
    /// a *missing* variant cannot occur without `patterns::tile` failing to
    /// compile (it has one exhaustive arm per schema pattern).
    #[test]
    fn all_lists_every_variant_in_declaration_order() {
        for (i, pattern) in BarPattern::ALL.iter().enumerate() {
            assert_eq!(*pattern as usize, i);
        }
    }

    /// Every variant has a tooltip name in every interface language.
    #[test]
    fn display_name_covers_every_variant_and_language() {
        for pattern in BarPattern::ALL {
            for lang in [
                crate::i18n::Language::English,
                crate::i18n::Language::Polish,
            ] {
                assert!(!pattern.display_name(lang).is_empty(), "{pattern:?}");
            }
        }
    }

    #[test]
    fn pct_x_conversion_round_trips() {
        let slot = Slot {
            start_pct: 25.0,
            end_pct: 75.0,
        };
        assert_eq!(slot.start_x(200.0), 50.0);
        assert_eq!(slot.end_x(200.0), 150.0);
        assert_eq!(Slot::pct_at(50.0, 200.0), 25.0);
        assert_eq!(Slot::pct_at(150.0, 200.0), 75.0);

        // Pointer positions beyond the bar clamp to the domain edges.
        assert_eq!(Slot::pct_at(-5.0, 200.0), Slot::PCT_MIN);
        assert_eq!(Slot::pct_at(999.0, 200.0), Slot::PCT_MAX);
    }
}
