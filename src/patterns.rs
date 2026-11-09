//! SVG tile artwork backing the [`BarPattern`] variants.
//!
//! Single source of truth for the tile bytes, so the PDF world
//! ([`crate::view_model::ConservationWorld`]) and the GTK shell embed
//! exactly the same artwork (one `include_bytes!` per file, no duplicated
//! assets). Source file names are Polish and stay that way, matching
//! `patterns/*.svg` on disk; only the code-level variants are English.
//!
//! There is deliberately **no positional coupling** with [`BarPattern::ALL`]:
//! every variant gets its own arm in [`tile`] holding its name and its bytes,
//! so the match is compiler-checked to cover the whole enum, and a missing
//! pattern breaks the build instead of misrendering.

use crate::storage::BarPattern;

/// Tile art for `pattern`: `(file name as served by the typst world, bytes)`.
/// One arm per variant — adding a [`BarPattern`] variant without artwork
/// fails to compile here.
#[must_use]
pub fn tile(pattern: BarPattern) -> (&'static str, &'static [u8]) {
    match pattern {
        BarPattern::Bolus => ("bolus.svg", include_bytes!("../patterns/bolus.svg")),
        BarPattern::Brick => ("cegla.svg", include_bytes!("../patterns/cegla.svg")),
        BarPattern::Ceramics => ("ceramika.svg", include_bytes!("../patterns/ceramika.svg")),
        BarPattern::Wood => ("drewno.svg", include_bytes!("../patterns/drewno.svg")),
        BarPattern::Textile => (
            "Tkanina, płótno, pasy tapicerskie.svg",
            include_bytes!("../patterns/Tkanina, płótno, pasy tapicerskie.svg"),
        ),
        BarPattern::MetalFoil => (
            "folie metalowe, złocenia, srebrzenia.svg",
            include_bytes!("../patterns/folie metalowe, złocenia, srebrzenia.svg"),
        ),
        BarPattern::Embroidery => ("haft.svg", include_bytes!("../patterns/haft.svg")),
        BarPattern::Stone => ("kamień.svg", include_bytes!("../patterns/kamień.svg")),
        BarPattern::Glue => ("klej.svg", include_bytes!("../patterns/klej.svg")),
        BarPattern::Bone => (
            "kość kamienie szlachetne.svg",
            include_bytes!("../patterns/kość kamienie szlachetne.svg"),
        ),
        BarPattern::SprayVarnish => ("laserunek.svg", include_bytes!("../patterns/laserunek.svg")),
        BarPattern::Painting => (
            "malowidło, przemalowania, retusz.svg",
            include_bytes!("../patterns/malowidło, przemalowania, retusz.svg"),
        ),
        BarPattern::Metal => (
            "metal, blacha, gwoździe.svg",
            include_bytes!("../patterns/metal, blacha, gwoździe.svg"),
        ),
        BarPattern::Paper => ("papier.svg", include_bytes!("../patterns/papier.svg")),
        BarPattern::Patina => ("patyna.svg", include_bytes!("../patterns/patyna.svg")),
        BarPattern::Parchment => ("pergamin.svg", include_bytes!("../patterns/pergamin.svg")),
        BarPattern::Whitewash => ("pobiala.svg", include_bytes!("../patterns/pobiala.svg")),
        BarPattern::Leather => ("skora.svg", include_bytes!("../patterns/skora.svg")),
        BarPattern::Stucco => ("stiuk.svg", include_bytes!("../patterns/stiuk.svg")),
        BarPattern::Glaze => ("szkliwo.svg", include_bytes!("../patterns/szkliwo.svg")),
        BarPattern::Plasterwork => (
            "sztukateria, odlewy i aplikacje gipsowe.svg",
            include_bytes!("../patterns/sztukateria, odlewy i aplikacje gipsowe.svg"),
        ),
        BarPattern::Cardboard => ("tektura.svg", include_bytes!("../patterns/tektura.svg")),
        BarPattern::Plastic => (
            "tworzywo sztuczne.svg",
            include_bytes!("../patterns/tworzywo sztuczne.svg"),
        ),
        BarPattern::MixedThread => (
            "watek mieszany.svg",
            include_bytes!("../patterns/watek mieszany.svg"),
        ),
        BarPattern::Varnish => ("werniks.svg", include_bytes!("../patterns/werniks.svg")),
        BarPattern::Wax => (
            "wosk, masy dublazowe.svg",
            include_bytes!("../patterns/wosk, masy dublazowe.svg"),
        ),
        BarPattern::Nonwoven => ("włóknina.svg", include_bytes!("../patterns/włóknina.svg")),
        BarPattern::Mortar => (
            "zaprawa tynk.svg",
            include_bytes!("../patterns/zaprawa tynk.svg"),
        ),
        BarPattern::GoldPowder => (
            "zloto w proszku.svg",
            include_bytes!("../patterns/zloto w proszku.svg"),
        ),
    }
}

pub fn all_tiles() -> impl Iterator<Item = (&'static str, &'static [u8])> {
    BarPattern::ALL.iter().copied().map(tile)
}
