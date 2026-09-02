use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Polish,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Translations {
    // Columns
    pub col_no: String,
    pub col_graphical: String,
    pub col_woven: String,
    pub col_char: String,
    pub col_thickness: String,
    pub col_chrono: String,
    pub col_dating: String,

    // Symbols
    pub flag_emoji: String,

    // Toolbar & Controls
    pub export_pdf: String,
    pub exporting_pdf: String,
    pub tool_edit: String,
    pub tool_create: String,
    pub tool_delete: String,
    pub status_ready: String,
    pub project_prefix: String,

    // Settings Popover
    pub settings_tooltip: String,
    pub delete_label: String,
}

impl Translations {
    pub fn get(lang: Language) -> Self {
        match lang {
            Language::English => Self {
                col_no: "No.".into(),
                col_graphical: "Graphical representation\nof the layer".into(),
                col_woven: "Woven trim".into(),
                col_char: "Layer characteristics".into(),
                col_thickness: "Layer thickness".into(),
                col_chrono: "Chronological phase".into(),
                col_dating: "Dating period".into(),
                flag_emoji: "🇬🇧".into(),
                export_pdf: "Export PDF".into(),
                exporting_pdf: "Exporting...".into(),
                tool_edit: "✥ Edit".into(),
                tool_create: "＋ Create".into(),
                tool_delete: "Delete".into(),
                status_ready: "Ready".into(),
                project_prefix: "Project: ".into(),
                settings_tooltip: "⚙️".into(),
                delete_label: "Delete".into(),
            },
            Language::Polish => Self {
                col_no: "Nr warstwy".into(),
                col_graphical: "Oznaczenie graficzne warstwy".into(),
                col_woven: "Krajka".into(),
                col_char: "Charakterystyka warstwy".into(),
                col_thickness: "Grubość warstwy".into(),
                col_chrono: "Faza\nchronologiczna".into(),
                col_dating: "Datowanie".into(),
                flag_emoji: "🇵🇱".into(),
                export_pdf: "Eksportuj PDF".into(),
                exporting_pdf: "Eksportuję...".into(),
                tool_edit: "✥ Edytuj".into(),
                tool_create: "＋ Utwórz".into(),
                tool_delete: "Usuń".into(),
                status_ready: "Gotowy".into(),
                project_prefix: "Projekt: ".into(),
                settings_tooltip: "⚙️".into(),
                delete_label: "Usuń".into(),
            },
        }
    }
}
