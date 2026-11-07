use std::collections::HashMap;
use std::sync::OnceLock;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Dict, Value};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;

struct CachedFontData {
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

static FONT_DATA: OnceLock<CachedFontData> = OnceLock::new();

fn cached_fonts() -> &'static CachedFontData {
    FONT_DATA.get_or_init(|| {
        let font_regular = Font::new(
            Bytes::from_static(include_bytes!("../assets/LinLibertine_R.ttf")),
            0,
        )
        .expect("Invalid font");
        let font_bold = Font::new(
            Bytes::from_static(include_bytes!("../assets/LinLibertine_RB.ttf")),
            0,
        )
        .expect("Invalid font");
        let font_italic = Font::new(
            Bytes::from_static(include_bytes!("../assets/LinLibertine_RI.ttf")),
            0,
        )
        .expect("Invalid font");

        let mut book = FontBook::new();
        book.push(font_regular.info().clone());
        book.push(font_bold.info().clone());
        book.push(font_italic.info().clone());

        CachedFontData {
            book: LazyHash::new(book),
            fonts: vec![font_regular, font_bold, font_italic],
        }
    })
}

pub struct ConservationWorld {
    library: LazyHash<typst::Library>,
    files: HashMap<FileId, Source>,
    assets: HashMap<&'static str, Bytes>,
    main_id: FileId,
}

impl ConservationWorld {
    pub fn new(template_source: String, json_view_model: String) -> Self {
        let mut files = HashMap::new();
        let main_path = VirtualPath::new("main.typ");
        let main_id = FileId::new(None, main_path);
        files.insert(main_id, Source::new(main_id, template_source));

        let mut inputs = Dict::new();
        inputs.insert("json_data".into(), Value::Str(json_view_model.into()));

        let mut assets = HashMap::new();
        for &(name, bytes) in &crate::patterns::TILES {
            assets.insert(name, Bytes::from_static(bytes));
        }

        Self {
            library: LazyHash::new(typst::Library::builder().with_inputs(inputs).build()),
            files,
            assets,
            main_id,
        }
    }
}

impl typst::World for ConservationWorld {
    fn library(&self) -> &LazyHash<typst::Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &cached_fonts().book
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.assets
            .get(id.vpath().as_rootless_path().to_str().unwrap_or(""))
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        cached_fonts().fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        use chrono::Datelike;
        let now = chrono::Utc::now();
        Datetime::from_ymd(now.year(), now.month() as u8, now.day() as u8)
    }
}
