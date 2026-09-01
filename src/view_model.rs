use comemo::Prehashed;
use std::collections::HashMap;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::Font;

pub struct ConservationWorld {
    library: Prehashed<typst::Library>,
    book: Prehashed<typst::text::FontBook>,
    fonts: Vec<typst::text::Font>,
    files: HashMap<FileId, Source>,
    main_id: FileId,
    json_id: FileId,
    json_bytes: Bytes,
}

impl ConservationWorld {
    pub fn new(template_source: String, json_view_model: String) -> Self {
        let mut files = HashMap::new();

        let main_path = VirtualPath::new("main.typ");
        let main_id = FileId::new(None, main_path);
        files.insert(main_id, Source::new(main_id, template_source));

        let json_path = VirtualPath::new("view_model.json");
        let json_id = FileId::new(None, json_path);
        // Convert string to raw bytes for Typst's file() method
        let json_bytes = Bytes::from(json_view_model.into_bytes());

        let font_data = include_bytes!("../assets/LinLibertine_R.ttf");
        let font = Font::new(font_data.as_slice().into(), 0).expect("Invalid font");
        let mut book = typst::text::FontBook::new();
        book.push(font.info().clone());

        Self {
            library: Prehashed::new(typst::Library::default()),
            book: Prehashed::new(book),
            fonts: vec![font],
            files,
            main_id,
            json_id,
            json_bytes,
        }
    }
}

impl typst::World for ConservationWorld {
    fn library(&self) -> &Prehashed<typst::Library> {
        &self.library
    }
    fn book(&self) -> &Prehashed<typst::text::FontBook> {
        &self.book
    }
    fn main(&self) -> Source {
        self.files[&self.main_id].clone()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.files
            .get(&id)
            .cloned()
            .ok_or(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.json_id {
            Ok(self.json_bytes.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
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
