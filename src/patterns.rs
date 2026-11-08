//! SVG tile artwork backing the [`BarPattern`] variants.
//!
//! Single source of truth for the tile bytes, so the PDF world
//! ([`crate::view_model::ConservationWorld`]) and the GTK shell embed
//! exactly the same artwork (one `include_bytes!` per file, no duplicated
//! assets). Source file names are Polish and stay that way, matching
//! `patterns/*.svg` on disk; only the code-level variants are English.

use crate::storage::BarPattern;

/// Tile art, indexed by the tile index used by the renderers (see
/// [`tile_index`]): `(file name as served by the typst world, bytes)`.
pub const TILES: [(&str, &[u8]); 29] = [
    // 0 = BarPattern::Bolus
    ("bolus.svg", include_bytes!("../patterns/bolus.svg")),
    // 1 = BarPattern::Brick
    ("cegla.svg", include_bytes!("../patterns/cegla.svg")),
    // 2 = BarPattern::Ceramics
    ("ceramika.svg", include_bytes!("../patterns/ceramika.svg")),
    // 3 = BarPattern::Wood
    ("drewno.svg", include_bytes!("../patterns/drewno.svg")),
    // 4 = BarPattern::Textile
    (
        "Tkanina, płótno, pasy tapicerskie.svg",
        include_bytes!("../patterns/Tkanina, płótno, pasy tapicerskie.svg"),
    ),
    // 5 = BarPattern::MetalFoil
    (
        "folie metalowe, złocenia, srebrzenia.svg",
        include_bytes!("../patterns/folie metalowe, złocenia, srebrzenia.svg"),
    ),
    // 6 = BarPattern::Embroidery
    ("haft.svg", include_bytes!("../patterns/haft.svg")),
    // 7 = BarPattern::Stone
    ("kamień.svg", include_bytes!("../patterns/kamień.svg")),
    // 8 = BarPattern::Glue
    ("klej.svg", include_bytes!("../patterns/klej.svg")),
    // 9 = BarPattern::Bone
    (
        "kość kamienie szlachetne.svg",
        include_bytes!("../patterns/kość kamienie szlachetne.svg"),
    ),
    // 10 = BarPattern::SprayVarnish
    ("laserunek.svg", include_bytes!("../patterns/laserunek.svg")),
    // 11 = BarPattern::Painting
    (
        "malowidło, przemalowania, retusz.svg",
        include_bytes!("../patterns/malowidło, przemalowania, retusz.svg"),
    ),
    // 12 = BarPattern::Metal
    (
        "metal, blacha, gwoździe.svg",
        include_bytes!("../patterns/metal, blacha, gwoździe.svg"),
    ),
    // 13 = BarPattern::Paper
    ("papier.svg", include_bytes!("../patterns/papier.svg")),
    // 14 = BarPattern::Patina
    ("patyna.svg", include_bytes!("../patterns/patyna.svg")),
    // 15 = BarPattern::Parchment
    ("pergamin.svg", include_bytes!("../patterns/pergamin.svg")),
    // 16 = BarPattern::Whitewash
    ("pobiala.svg", include_bytes!("../patterns/pobiala.svg")),
    // 17 = BarPattern::Leather
    ("skora.svg", include_bytes!("../patterns/skora.svg")),
    // 18 = BarPattern::Stucco
    ("stiuk.svg", include_bytes!("../patterns/stiuk.svg")),
    // 19 = BarPattern::Glaze
    ("szkliwo.svg", include_bytes!("../patterns/szkliwo.svg")),
    // 20 = BarPattern::Plasterwork
    (
        "sztukateria, odlewy i aplikacje gipsowe.svg",
        include_bytes!("../patterns/sztukateria, odlewy i aplikacje gipsowe.svg"),
    ),
    // 21 = BarPattern::Cardboard
    ("tektura.svg", include_bytes!("../patterns/tektura.svg")),
    // 22 = BarPattern::Plastic
    (
        "tworzywo sztuczne.svg",
        include_bytes!("../patterns/tworzywo sztuczne.svg"),
    ),
    // 23 = BarPattern::MixedThread
    (
        "watek mieszany.svg",
        include_bytes!("../patterns/watek mieszany.svg"),
    ),
    // 24 = BarPattern::Varnish
    ("werniks.svg", include_bytes!("../patterns/werniks.svg")),
    // 25 = BarPattern::Wax
    (
        "wosk, masy dublazowe.svg",
        include_bytes!("../patterns/wosk, masy dublazowe.svg"),
    ),
    // 26 = BarPattern::Nonwoven
    ("włóknina.svg", include_bytes!("../patterns/włóknina.svg")),
    // 27 = BarPattern::Mortar
    (
        "zaprawa tynk.svg",
        include_bytes!("../patterns/zaprawa tynk.svg"),
    ),
    // 28 = BarPattern::GoldPowder
    (
        "zloto w proszku.svg",
        include_bytes!("../patterns/zloto w proszku.svg"),
    ),
];

/// Tile index for a pattern. Every [`BarPattern`] variant is backed by a
/// tile in [`TILES`], so this is a total mapping.
#[must_use]
pub fn tile_index(pattern: &BarPattern) -> usize {
    match pattern {
        BarPattern::Bolus => 0,
        BarPattern::Brick => 1,
        BarPattern::Ceramics => 2,
        BarPattern::Wood => 3,
        BarPattern::Textile => 4,
        BarPattern::MetalFoil => 5,
        BarPattern::Embroidery => 6,
        BarPattern::Stone => 7,
        BarPattern::Glue => 8,
        BarPattern::Bone => 9,
        BarPattern::SprayVarnish => 10,
        BarPattern::Painting => 11,
        BarPattern::Metal => 12,
        BarPattern::Paper => 13,
        BarPattern::Patina => 14,
        BarPattern::Parchment => 15,
        BarPattern::Whitewash => 16,
        BarPattern::Leather => 17,
        BarPattern::Stucco => 18,
        BarPattern::Glaze => 19,
        BarPattern::Plasterwork => 20,
        BarPattern::Cardboard => 21,
        BarPattern::Plastic => 22,
        BarPattern::MixedThread => 23,
        BarPattern::Varnish => 24,
        BarPattern::Wax => 25,
        BarPattern::Nonwoven => 26,
        BarPattern::Mortar => 27,
        BarPattern::GoldPowder => 28,
    }
}
