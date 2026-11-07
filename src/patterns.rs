//! SVG tile artwork backing the image-based [`BarPattern`] variants.
//!
//! Single source of truth for the tile bytes, so the PDF world
//! ([`crate::view_model::ConservationWorld`]) and the GTK shell embed
//! exactly the same artwork (one `include_bytes!` per file, no duplicated
//! assets). Source file names are Polish and stay that way, matching
//! `patterns/*.svg` on disk; only the code-level variants are English.

use crate::storage::BarPattern;

/// Tile art, indexed by the tile index used by the renderers (see
/// [`tile_index`]): `(file name as served by the typst world, bytes)`.
pub const TILES: [(&str, &[u8]); 4] = [
    // 0 = BarPattern::Bolus
    ("bolus.svg", include_bytes!("../patterns/bolus.svg")),
    // 1 = BarPattern::Brick
    ("cegla.svg", include_bytes!("../patterns/cegla.svg")),
    // 2 = BarPattern::Ceramics
    ("ceramika.svg", include_bytes!("../patterns/ceramika.svg")),
    // 3 = BarPattern::Wood
    ("drewno.svg", include_bytes!("../patterns/drewno.svg")),
];

/// Tile index for patterns backed by SVG artwork. Returns `None` for
/// patterns drawn natively with Cairo/typst primitives.
#[must_use]
pub fn tile_index(pattern: &BarPattern) -> Option<usize> {
    match pattern {
        BarPattern::Bolus => Some(0),
        BarPattern::Brick => Some(1),
        BarPattern::Ceramics => Some(2),
        BarPattern::Wood => Some(3),
        BarPattern::VerticalHatch | BarPattern::DenseGrid => None,
    }
}
