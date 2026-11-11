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
pub fn pct_to_x(pct: f32, w: f64) -> f64 {
    w * (f64::from(pct) / f64::from(Slot::PCT_MAX))
}

/// Resolves where a Create-tool click at `at_pct` inserts a new slot into
/// `slots` (sorted by start edge), or `None` when there is no free position.
///
/// A click in open space inserts exactly at `at_pct`. A click within the
/// `MIN_GAP_PCT` margin beside a slot's edge snaps to that margin — but only
/// if the snapped position also keeps the margin to the slot on the other
/// side of the gap; in a gap narrower than `2 * MIN_GAP_PCT` (or in the
/// middle of a slot) there is no free position and the click is ignored.
#[must_use]
pub fn snap_add_position(slots: &[Slot], at_pct: f32) -> Option<f32> {
    let idx = match slots
        .iter()
        .position(|s| at_pct >= s.start_pct - MIN_GAP_PCT && at_pct <= s.end_pct + MIN_GAP_PCT)
    {
        Some(idx) => idx,
        // Open space: insert exactly where clicked.
        None => return Some(at_pct),
    };
    let s = slots[idx];

    if at_pct > s.end_pct {
        // Just right of `s`: snap to its end + margin, provided the slot
        // after it leaves room for the snapped position.
        let candidate = s.end_pct + MIN_GAP_PCT;
        let room = slots
            .get(idx + 1)
            .map_or(candidate <= Slot::PCT_MAX, |next| {
                candidate <= next.start_pct - MIN_GAP_PCT
            });
        room.then_some(candidate)
    } else if at_pct < s.start_pct {
        // Just left of `s`: snap to its start - margin, provided the slot
        // before it leaves room for the snapped position.
        let candidate = s.start_pct - MIN_GAP_PCT;
        let room = idx
            .checked_sub(1)
            .map_or(candidate >= Slot::PCT_MIN, |prev| {
                candidate >= slots[prev].end_pct + MIN_GAP_PCT
            });
        room.then_some(candidate)
    } else {
        // Strictly inside a slot: no free position nearby.
        None
    }
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

impl Default for ConservationDocument {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            subtitle: String::new(),
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            entries: vec![],
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            document: ConservationDocument::default(),
            next_id: 1,
            last_error: None,
            is_exporting: false,
            language: Language::English,
            current_file_path: None,
            history: History::default(),
        }
    }
}

#[cfg(test)]
impl ConservationDocument {
    /// A representative two-row document for tests (the shipped default is
    /// empty); the two rows share `chrono_group = 0`, i.e. a merged
    /// "Iron Age" group, so group split/merge behaviour can be exercised.
    pub(crate) fn sample() -> Self {
        Self {
            project_name: "Sample Project".to_string(),
            subtitle: String::new(),
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
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ConservationDocument {
    pub project_name: String,
    #[serde(default)]
    pub subtitle: String,
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
    use super::{BarPattern, ConservationDocument, Slot, snap_add_position};

    fn mk(v: &[(f32, f32)]) -> Vec<Slot> {
        v.iter()
            .map(|&(start_pct, end_pct)| Slot { start_pct, end_pct })
            .collect()
    }

    /// A click in open space inserts exactly where it lands.
    #[test]
    fn open_space_inserts_where_clicked() {
        let slots = mk(&[(0.0, 40.0), (60.0, 100.0)]);
        assert_eq!(snap_add_position(&slots, 50.0), Some(50.0));
    }

    /// A click inside a slot's margin zone snaps to that margin, on either
    /// side, as long as the snapped position keeps the margin to the slot
    /// on the other side of the gap.
    #[test]
    fn margin_clicks_snap_to_the_nearest_free_position() {
        let slots = mk(&[(0.0, 40.0), (60.0, 100.0)]);
        // Just after A (A ends at 40): snaps to 42, clear of B at 60.
        assert_eq!(snap_add_position(&slots, 41.0), Some(42.0));
        // Just before B (B starts at 60): snaps to 58, clear of A at 40.
        assert_eq!(snap_add_position(&slots, 59.0), Some(58.0));

        // A tight gap (A ends at 40, B starts at 42) has no free position:
        // A's margin (42) collides with B's margin (40).
        let tight = mk(&[(0.0, 40.0), (42.0, 100.0)]);
        assert_eq!(snap_add_position(&tight, 41.0), None);
    }

    /// Clicks with no free position nearby are ignored: inside a slot, or
    /// past a bar boundary that cannot honour the margin.
    #[test]
    fn impossible_clicks_are_ignored() {
        let slots = mk(&[(0.0, 40.0), (60.0, 100.0)]);
        // In the middle of a slot.
        assert_eq!(snap_add_position(&slots, 20.0), None);
        assert_eq!(snap_add_position(&slots, 80.0), None);

        // The last slot ends at 99; its margin would need 101%.
        let near_end = mk(&[(0.0, 99.0)]);
        assert_eq!(snap_add_position(&near_end, 99.5), None);
        // ... but ending at 98 leaves exactly room for a 100% slot.
        assert_eq!(snap_add_position(&mk(&[(0.0, 98.0)]), 99.0), Some(100.0));

        // The first slot starts at 1; its margin would need -1%.
        let near_start = mk(&[(1.0, 40.0)]);
        assert_eq!(snap_add_position(&near_start, 0.5), None);
        // ... but starting at 2 leaves exactly room for a 0% slot.
        assert_eq!(snap_add_position(&mk(&[(2.0, 40.0)]), 1.0), Some(0.0));
    }

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

    /// Documents saved before the subtitle existed must still open; the
    /// missing field defaults to empty (no subtitle line in the PDF).
    #[test]
    fn documents_without_a_subtitle_still_parse() {
        let old = r#"{"project_name":"Old","date":"2024-01-01","entries":[]}"#;
        let doc: ConservationDocument = serde_json::from_str(old).unwrap();
        assert_eq!(doc.subtitle, "");
        assert_eq!(doc.project_name, "Old");
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
