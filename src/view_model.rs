use std::collections::HashMap;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Dict, Value};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::Font;
use typst::utils::LazyHash;

pub struct ConservationWorld {
    library: LazyHash<typst::Library>,
    book: LazyHash<typst::text::FontBook>,
    fonts: Vec<typst::text::Font>,
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

        let font_data = include_bytes!("../assets/LinLibertine_R.ttf");
        let font = Font::new(font_data.as_slice().into(), 0).expect("Invalid font");
        let mut book = typst::text::FontBook::new();
        book.push(font.info().clone());

        let mut inputs = Dict::new();
        inputs.insert("json_data".into(), Value::Str(json_view_model.into()));

        let mut assets = HashMap::new();
        for &(name, bytes) in &crate::patterns::TILES {
            assets.insert(name, Bytes::from_static(bytes));
        }

        Self {
            library: LazyHash::new(typst::Library::builder().with_inputs(inputs).build()),
            book: LazyHash::new(book),
            fonts: vec![font],
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

    fn book(&self) -> &LazyHash<typst::text::FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.files
            .get(&id)
            .cloned()
            .ok_or(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.assets
            .get(id.vpath().as_rootless_path().to_str().unwrap_or(""))
            .cloned()
            .ok_or(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        let now = chrono::Utc::now();
        Datetime::from_ymd(
            now.format("%Y").to_string().parse().unwrap(),
            now.format("%m").to_string().parse().unwrap(),
            now.format("%d").to_string().parse().unwrap(),
        )
    }
}
