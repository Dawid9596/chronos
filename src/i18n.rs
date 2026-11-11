use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Polish,
}

impl Language {
    /// All languages in declaration order.
    pub const ALL: [Language; 2] = [Self::English, Self::Polish];

    /// Native display name of the language, for language pickers only.
    /// Deliberately *never* translated into the current UI language: a
    /// selector is expected to list every language in its own words.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Polish => "Polski",
        }
    }

    /// ISO 639-1 code used to persist the language in the settings file.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Polish => "pl",
        }
    }

    /// The language stored under a settings-code string (case-insensitive),
    /// or `None` for anything unrecognized — which keeps defaults instead of
    /// guessing.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|lang| lang.code().eq_ignore_ascii_case(code))
    }

    /// The next language in [`Language::ALL`], wrapping around. Drives the
    /// core's toggle event and the toolbar's language flag button.
    #[must_use]
    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|l| *l == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Translations {
    pub language: Language,
    // Menus
    pub menu_file: String,
    pub menu_edit: String,
    pub menu_about: String,
    pub menu_help: String,
    pub menu_check_updates: String,
    pub menu_open: String,
    pub menu_save: String,
    pub menu_save_as: String,
    pub menu_export: String,
    pub menu_canvas_params: String,
    pub menu_preferences: String,
    pub menu_shortcuts: String,
    pub menu_undo: String,
    pub menu_redo: String,
    // Columns
    pub col_no: String,
    pub col_graphical: String,
    pub col_woven: String,
    pub col_char: String,
    pub col_thickness: String,
    pub col_chrono: String,
    pub col_dating: String,
    // Shortened (non-destructive) table-column titles
    pub col_no_short: String,
    pub col_graphical_short: String,
    pub col_woven_short: String,
    pub col_char_short: String,
    pub col_thickness_short: String,
    pub col_chrono_short: String,
    pub col_dating_short: String,
    // Toolbar & Controls
    pub export_pdf: String,
    pub exporting_pdf: String,
    pub tool_edit: String,
    pub tool_create: String,
    pub tool_delete: String,
    pub tool_paint: String,
    pub tool_split: String,
    pub tool_duplicate: String,
    pub tool_merge: String,
    /// The PDF footer credits the pattern artwork (CC BY-SA 4.0 requires it);
    /// passed to the typst template as plain text, already in this language.
    pub artwork_attribution: String,
    pub status_ready: String,
    pub project_prefix: String,
    pub project_untitled: String,
    // Settings Popover
    /// Legacy, Noto font is not bundled
    pub settings_tooltip: String,
    pub delete_label: String,
    // File Dialogs
    pub dialog_open_title: String,
    pub dialog_save_title: String,
    pub filter_json_name: String,
    pub filter_project_name: String,
    pub dialog_restore_defaults: String,
    pub dialog_close: String,
    pub update_available_title: String,
    /// `{version}` is replaced with the latest release version, `{current}`
    /// with the running program version.
    pub update_available_body: String,
    pub update_open_releases: String,
    /// Silences the automatic check for the shown release.
    pub update_skip_version: String,
    /// `{current}` is replaced with the running program version.
    pub update_up_to_date: String,
    pub update_check_failed: String,
    // Canvas Parameters & Preferences Dialogs
    pub dialog_paper_size: String,
    pub dialog_landscape: String,
    pub dialog_page_margin: String,
    pub dialog_cell_padding: String,
    pub dialog_bar_padding: String,
    pub dialog_zero_width_line: String,
    pub dialog_visible_columns_heading: String,
    pub dialog_interface_language: String,
    pub dialog_embed_project_data: String,
    pub dialog_import_from_file: String,
    // Keyboard Shortcuts Dialog & Tooltips
    pub shortcut_tooltip_prefix: String,
    pub shortcuts_section_tools: String,
    pub shortcuts_section_variants: String,
    pub shortcuts_section_fixed: String,
    pub action_next_variant: String,
    pub action_prev_variant: String,
    pub shortcuts_fixed_inline_edit: String,
    pub shortcuts_fixed_cancel_edit: String,
    pub shortcuts_key_double_click: String,
    pub shortcuts_press_new: String,
    pub shortcuts_prompt_cancel: String,
    pub shortcuts_click_tooltip: String,
    pub shortcuts_restore_default_tooltip: String,
    pub shortcuts_restore_all: String,
    pub shortcuts_err_invalid: String,
    pub shortcuts_err_needs_modifier: String,
    pub shortcuts_err_conflict_prefix: String,
    // Status & Error Prefixes
    pub status_loaded_prefix: String,
    pub status_saved_prefix: String,
    /// Shown after silently restoring crash-recovery autosave on startup.
    pub status_recovered: String,
    pub error_parse_prefix: String,
    pub error_open_prefix: String,
    pub error_save_prefix: String,
    pub error_no_project_data: String,
    pub error_no_export_settings: String,
    // Help & About documentation windows
    pub doc_about_title: String,
    pub doc_help_title: String,
    pub about_markdown: String,
    pub help_markdown: String,
}

impl Translations {
    /// The shortened table-column title by column index
    #[must_use]
    pub fn col_short(&self, index: usize) -> &str {
        match index {
            0 => &self.col_no_short,
            1 => &self.col_graphical_short,
            2 => &self.col_woven_short,
            3 => &self.col_char_short,
            4 => &self.col_thickness_short,
            5 => &self.col_chrono_short,
            _ => &self.col_dating_short,
        }
    }

    pub fn get(lang: Language) -> Self {
        match lang {
            Language::English => Self {
                language: Language::English,
                menu_file: "File".into(),
                menu_edit: "Edit".into(),
                menu_about: "About".into(),
                menu_help: "Help".into(),
                menu_check_updates: "Check for Updates...".into(),
                menu_open: "Open...".into(),
                menu_save: "Save".into(),
                menu_save_as: "Save As...".into(),
                menu_export: "Export PDF".into(),
                menu_canvas_params: "Output Canvas Parameters...".into(),
                menu_preferences: "Preferences...".into(),
                menu_shortcuts: "Keyboard Shortcuts...".into(),
                menu_undo: "Undo".into(),
                menu_redo: "Redo".into(),
                col_no: "No.".into(),
                col_graphical: "Graphical representation\nof the layer".into(),
                col_woven: "Woven trim".into(),
                col_char: "Layer characteristics".into(),
                col_thickness: "Layer thickness".into(),
                col_chrono: "Chronological phase".into(),
                col_dating: "Dating period".into(),
                col_no_short: "No.".into(),
                col_graphical_short: "Graphical representation".into(),
                col_woven_short: "Woven trim".into(),
                col_char_short: "Characteristics".into(),
                col_thickness_short: "Thickness".into(),
                col_chrono_short: "Chronology".into(),
                col_dating_short: "Dating".into(),
                export_pdf: "Export PDF".into(),
                exporting_pdf: "Exporting...".into(),
                tool_edit: "Edit".into(),
                tool_create: "Create".into(),
                tool_delete: "Delete".into(),
                tool_paint: "Paint".into(),
                tool_split: "Split".into(),
                tool_duplicate: "Duplicate".into(),
                tool_merge: "Merge".into(),
                artwork_attribution: "Pattern artwork: @kationart, CC BY-SA 4.0".into(),
                status_ready: "Ready".into(),
                project_prefix: "Project: ".into(),
                project_untitled: "Untitled".into(),
                settings_tooltip: "<span font_family='Noto Color Emoji'>⚙️</span>".into(),
                delete_label: "Delete".into(),
                dialog_open_title: "Open File".into(),
                dialog_save_title: "Save File".into(),
                filter_json_name: "JSON Files".into(),
                filter_project_name: "Project files (JSON, PDF)".into(),
                dialog_restore_defaults: "Restore Defaults".into(),
                dialog_close: "Close".into(),
                update_available_title: "Update available".into(),
                update_available_body: "Version {version} is now available. You are running version {current}.".into(),
                update_open_releases: "Open Releases Page".into(),
                update_skip_version: "Skip This Version".into(),
                update_up_to_date: "You are running the latest version ({current}).".into(),
                update_check_failed: "Could not check for updates.".into(),
                dialog_paper_size: "Paper Size:".into(),
                dialog_landscape: "Landscape Orientation".into(),
                dialog_page_margin: "Page Margin (mm):".into(),
                dialog_cell_padding: "Cell Padding (mm):".into(),
                dialog_bar_padding: "Pattern Cell Padding (mm):".into(),
                dialog_zero_width_line: "Zero-Width Line (pt):".into(),
                dialog_visible_columns_heading: "Visible Output Columns & Ratios".into(),
                dialog_interface_language: "Interface Language:".into(),
                dialog_embed_project_data: "Embed project data in PDF".into(),
                dialog_import_from_file: "Import from file...".into(),
                shortcut_tooltip_prefix: "Shortcut: ".into(),
                shortcuts_section_tools: "Tools".into(),
                shortcuts_section_variants: "Variants".into(),
                shortcuts_section_fixed: "Fixed".into(),
                action_next_variant: "Next Variant".into(),
                action_prev_variant: "Previous Variant".into(),
                shortcuts_fixed_inline_edit: "Inline Edit / Reorder Layer".into(),
                shortcuts_fixed_cancel_edit: "Cancel Inline Edit".into(),
                shortcuts_key_double_click: "Double-Click".into(),
                shortcuts_press_new: "Press new shortcut...".into(),
                shortcuts_prompt_cancel: "Press the new shortcut, or Esc to cancel.".into(),
                shortcuts_click_tooltip: "Click, then press the new shortcut".into(),
                shortcuts_restore_default_tooltip: "Restore default".into(),
                shortcuts_restore_all: "Restore all defaults".into(),
                shortcuts_err_invalid: "That key combination can't be used.".into(),
                shortcuts_err_needs_modifier: "File shortcuts must include Ctrl or Alt.".into(),
                shortcuts_err_conflict_prefix: "Already used by ".into(),
                status_loaded_prefix: "Loaded: ".into(),
                status_saved_prefix: "Saved: ".into(),
                status_recovered: "Recovered unsaved changes.".into(),
                error_parse_prefix: "Error parsing file: ".into(),
                error_open_prefix: "Error opening file: ".into(),
                error_save_prefix: "Error saving file: ".into(),
                error_no_project_data: "This PDF has no embedded project data. Only PDFs exported with project data embedded can be opened.".into(),
                error_no_export_settings: "This file contains no export settings.".into(),
                doc_about_title: "About Chronos".into(),
                doc_help_title: "Chronos Help".into(),
                // `{version}` is replaced with the program version on display;
                // `{authors}` with the author line (from the desktop crate's
                // `[package] authors`); the `{#anchor}` markers must stay
                // stable, they are link targets.
                about_markdown: r#"# Conservation Editor (Chronos)

**Version:** {version}
**Author:** {authors}

---

## Overview {#overview}

Conservation Editor is a stratigraphic layer and condition documentation tool.

- **Editor:** Manage documents in standard format including layer characteristics, thickness, chronological phases, and dating periods.
- **WYSIWYG:** Visually specify graphical representations.
- **PDF Generation:** Export reports to PDF.

Need assistance? Open the [User Guide](doc:help) or jump directly to [Keyboard Shortcuts](doc:help#shortcuts).

---

## License {#license}

This program is free software: you can redistribute it and/or modify it under the terms of the **GNU General Public License v3** as published by the Free Software Foundation (see the `LICENSE` file shipped with the program sources). It comes without any warranty; see the license for details. Its complete source code is available at [github.com/Dawid9596/chronos](https://github.com/Dawid9596/chronos).

The bar-pattern artwork embedded in exported PDFs is copyright **@kationart** and licensed under **CC BY-SA 4.0** (see the `patterns/LICENSE` file).

**Documents you create with this program are yours.** Neither the program's license nor the artwork license restricts the use, publishing, or sale of the reports you produce; the artwork attribution shown at the end of each export satisfies the artwork's license.
"#.into(),
                help_markdown: r#"# Chronos Help & Documentation

Welcome to the Conservation Editor documentation. Use the internal links below to jump to a topic:

- [Working with Tools](#tools)
- [Managing Layers & Groups](#layers)
- [Saving & Exporting](#files)
- [Keyboard Shortcuts](#shortcuts)
- [About Chronos](doc:about)

---

## Working with Tools {#tools}

The top toolbar provides interactive tools for editing condition bars and cells:

- **Edit:** Drag slot edges to resize coverage, or click a bar to cycle through patterns. Right-click cycles backward.
- **Create:** Click inside a condition bar to insert a new slot at the cursor position.
- **Delete:** Click an existing slot inside a condition bar to remove it.
- **Paint:** Select one of the 29 patterns (Bolus, Brick, Ceramics, Wood, Textile, Metal Foil, Embroidery, Stone, Glue, Bone, Spray Varnish, Painting, Metal, Paper, Patina, Parchment, Whitewash, Leather, Stucco, Glaze, Plasterwork, Cardboard, Plastic, Mixed Thread, Varnish, Wax, Nonwoven, Mortar, Gold Powder) and click any bar to apply it.
- **Split:** Click the highlighted horizontal divider inside a merged cell to split the group at that boundary.
- **Duplicate:** Click a row number or condition bar to duplicate the entire stratigraphic layer.
- **Merge:** Choose **Merge Down** or **Merge Up** and click a Chronological Phase or Dating Period cell to merge it with its neighbor.

---

## Managing Layers & Groups {#layers}

- **Insert Layer:** Click the thin gap above, below, or between groups to insert a blank layer.
- **Reorder Layer:** Double-click and hold a row number in the **No.** column, then drag vertically to reorder.
- **Edit Text:** Double-click any text cell or the project title to edit inline. Press `Enter` to commit or `Escape` to cancel.

---

## Saving & Exporting {#files}

Projects are saved as `.json` files and can be exported directly to `.pdf` from the **File** menu or the toolbar button. Exported PDFs can carry the embedded project, so they can be re-opened for editing; that behaviour can be switched off in the **Output Canvas Parameters** dialog.

Export settings (column visibility and canvas parameters) are stored in every Save file. To apply settings from another file or from a PDF without opening it as a project, choose **Import from file...** in the output canvas parameters dialog.

---

## Keyboard Shortcuts {#shortcuts}
Defaults are
```
Ctrl + O          Open Project (.json)
Ctrl + S          Save Project
Ctrl + Shift + S  Save Project As...
Ctrl + E          Export to PDF
Ctrl + Z          Undo
Ctrl + Y          Redo
E                 Edit Tool
C                 Create Tool
D                 Delete Tool
P                 Paint Tool
S                 Split Tool
R                 Duplicate Tool
M                 Merge Tool
]                 Next Variant
[                 Previous Variant
```
`Ctrl + Mouse wheel` also cycles through the active tool's variants. All of these, as well as extra mouse buttons, can be rebound in the **Keyboard Shortcuts** dialog.

Return to [Top of Help](#top) or read [About Conservation Editor](doc:about).
"#.into(),
            },
            Language::Polish => Self {
                language: Language::Polish,
                menu_file: "Plik".into(),
                menu_edit: "Edycja".into(),
                menu_about: "O programie".into(),
                menu_help: "Pomoc".into(),
                menu_check_updates: "Sprawdź aktualizacje...".into(),
                menu_open: "Otwórz...".into(),
                menu_save: "Zapisz".into(),
                menu_save_as: "Zapisz jako...".into(),
                menu_export: "Eksportuj PDF".into(),
                menu_canvas_params: "Parametry obszaru wyjściowego...".into(),
                menu_preferences: "Preferencje...".into(),
                menu_shortcuts: "Skróty klawiszowe...".into(),
                menu_undo: "Cofnij".into(),
                menu_redo: "Ponów".into(),
                col_no: "Nr warstwy".into(),
                col_graphical: "Oznaczenie graficzne warstwy".into(),
                col_woven: "Krajka".into(),
                col_char: "Charakterystyka warstwy".into(),
                col_thickness: "Grubość warstwy".into(),
                col_chrono: "Faza\nchronologiczna".into(),
                col_dating: "Datowanie".into(),
                col_no_short: "Nr".into(),
                col_graphical_short: "Oznaczenie graficzne".into(),
                col_woven_short: "Krajka".into(),
                col_char_short: "Charakterystyka".into(),
                col_thickness_short: "Grubość".into(),
                col_chrono_short: "Faza".into(),
                col_dating_short: "Datowanie".into(),
                export_pdf: "Eksportuj PDF".into(),
                exporting_pdf: "Eksportuję...".into(),
                tool_edit: "Edytuj".into(),
                tool_create: "Utwórz".into(),
                tool_delete: "Usuń".into(),
                tool_paint: "Maluj".into(),
                tool_split: "Rozdziel".into(),
                tool_duplicate: "Duplikuj".into(),
                tool_merge: "Scal".into(),
                artwork_attribution: "Grafika wzorów: @kationart, CC BY-SA 4.0".into(),
                status_ready: "Gotowy".into(),
                project_prefix: "Projekt: ".into(),
                project_untitled: "Bez nazwy".into(),
                settings_tooltip: "<span font_family='Noto Color Emoji'>⚙️</span>".into(),
                delete_label: "Usuń".into(),
                dialog_open_title: "Otwórz plik".into(),
                dialog_save_title: "Zapisz plik".into(),
                filter_json_name: "Pliki JSON".into(),
                filter_project_name: "Pliki projektu (JSON, PDF)".into(),
                dialog_restore_defaults: "Przywróć domyślne".into(),
                dialog_close: "Zamknij".into(),
                update_available_title: "Dostępna aktualizacja".into(),
                update_available_body: "Dostępna jest wersja {version}. Używasz wersji {current}.".into(),
                update_open_releases: "Otwórz stronę wydań".into(),
                update_skip_version: "Pomiń tę wersję".into(),
                update_up_to_date: "Korzystasz z najnowszej wersji ({current}).".into(),
                update_check_failed: "Nie udało się sprawdzić aktualizacji.".into(),
                dialog_paper_size: "Format papieru:".into(),
                dialog_landscape: "Orientacja pozioma".into(),
                dialog_page_margin: "Margines strony (mm):".into(),
                dialog_cell_padding: "Dopełnienie komórek (mm):".into(),
                dialog_bar_padding: "Dopełnienie komórek wzorów (mm):".into(),
                dialog_zero_width_line: "Linia zerowej szerokości (pt):".into(),
                dialog_visible_columns_heading: "Widoczne kolumny wyjściowe i proporcje".into(),
                dialog_interface_language: "Język interfejsu:".into(),
                dialog_embed_project_data: "Osadź dane projektu w PDF".into(),
                dialog_import_from_file: "Importuj z pliku...".into(),
                shortcut_tooltip_prefix: "Skrót: ".into(),
                shortcuts_section_tools: "Narzędzia".into(),
                shortcuts_section_variants: "Warianty".into(),
                shortcuts_section_fixed: "Stałe".into(),
                action_next_variant: "Następny wariant".into(),
                action_prev_variant: "Poprzedni wariant".into(),
                shortcuts_fixed_inline_edit: "Edycja w miejscu / Zmiana kolejności warstwy".into(),
                shortcuts_fixed_cancel_edit: "Anuluj edycję w miejscu".into(),
                shortcuts_key_double_click: "Podwójne kliknięcie".into(),
                shortcuts_press_new: "Naciśnij nowy skrót...".into(),
                shortcuts_prompt_cancel: "Naciśnij nowy skrót lub Esc, aby anulować.".into(),
                shortcuts_click_tooltip: "Kliknij, a następnie naciśnij nowy skrót".into(),
                shortcuts_restore_default_tooltip: "Przywróć domyślny".into(),
                shortcuts_restore_all: "Przywróć wszystkie domyślne".into(),
                shortcuts_err_invalid: "Nie można użyć tej kombinacji klawiszy.".into(),
                shortcuts_err_needs_modifier: "Skróty plikowe muszą zawierać Ctrl lub Alt.".into(),
                shortcuts_err_conflict_prefix: "Już używane przez ".into(),
                status_loaded_prefix: "Wczytano: ".into(),
                status_saved_prefix: "Zapisano: ".into(),
                status_recovered: "Przywrócono niezapisane zmiany.".into(),
                error_parse_prefix: "Błąd parsowania pliku: ".into(),
                error_open_prefix: "Błąd otwierania pliku: ".into(),
                error_save_prefix: "Błąd zapisywania pliku: ".into(),
                error_no_project_data: "Ten plik PDF nie zawiera osadzonych danych projektu. Można otworzyć tylko pliki PDF wyeksportowane z osadzonymi danymi projektu.".into(),
                error_no_export_settings: "Ten plik nie zawiera ustawień eksportu.".into(),
                doc_about_title: "O programie".into(),
                doc_help_title: "Pomoc Chronos".into(),
                about_markdown: r#"# Conservation Editor (Chronos)

**Wersja:** {version}
**Autor:** {authors}

---

## Przegląd {#overview}

Conservation Editor to narzędzie do dokumentacji warstw stratygraficznych i ich stanu.

- **Edytor:** Zarządzanie dokumentami w standardowym formacie, obejmującym charakterystykę warstw, miąższość, fazy chronologiczne i okresy datowania.
- **WYSIWYG:** Wizualne określanie przedstawienia graficznego.
- **Generowanie PDF:** Eksport raportów do formatu PDF.

Potrzebujesz pomocy? Otwórz [Przewodnik użytkownika](doc:help) lub przejdź od razu do [Skrótów klawiszowych](doc:help#shortcuts).

---

## Licencja {#license}

Ten program jest wolnym oprogramowaniem: można go redystrybuować i modyfikować na warunkach licencji **GNU General Public License v3**, opublikowanej przez Free Software Foundation (zob. plik `LICENSE` dołączony do kodu źródłowego programu). Program jest oferowany bez jakichkolwiek gwarancji; szczegóły znajdują się w treści licencji. Pełny kod źródłowy jest dostępny pod adresem [github.com/Dawid9596/chronos](https://github.com/Dawid9596/chronos).

Grafika wzorów komórek osadzana w eksportowanych plikach PDF podlega prawom autorskim podmiotu **@kationart** i jest licencjonowana na warunkach **CC BY-SA 4.0** (zob. plik `patterns/LICENSE`).

**Dokumenty utworzone w tym programie są własnością użytkownika.** Ani licencja programu, ani licencja grafiki nie ograniczają wykorzystywania, publikowania ani sprzedaży wygenerowanych raportów; nota o autorstwie grafiki pokazana na końcu każdego eksportu spełnia wymagania licencji grafiki.
"#.into(),
                help_markdown: r#"# Pomoc i dokumentacja Chronos

Witaj w dokumentacji Conservation Editor. Odnośniki poniżej prowadzą do kolejnych tematów:

- [Praca z narzędziami](#tools)
- [Warstwy i grupy](#layers)
- [Zapis i eksport](#files)
- [Skróty klawiszowe](#shortcuts)
- [O programie](doc:about)

---

## Praca z narzędziami {#tools}

Pasek narzędzi na górze okna zawiera narzędzia przeznaczone do edycji pasków stanu zachowania oraz komórek:

- **Edytuj:** Przeciągaj krawędzie slotów, aby zmieniać pokrycie, lub klikaj pasek, aby przechodzić przez wzory; prawy przycisk myszy przechodzi przez nie w drugą stronę.
- **Utwórz:** Kliknięcie wewnątrz paska wstawia nowy slot w miejscu kursora.
- **Usuń:** Kliknięcie istniejącego slotu wewnątrz paska usuwa go.
- **Maluj:** Wybierz jeden z 29 wzorów (Bolus, Brick, Ceramics, Wood, Textile, Metal Foil, Embroidery, Stone, Glue, Bone, Spray Varnish, Painting, Metal, Paper, Patina, Parchment, Whitewash, Leather, Stucco, Glaze, Plasterwork, Cardboard, Plastic, Mixed Thread, Varnish, Wax, Nonwoven, Mortar, Gold Powder) i kliknij dowolny pasek, aby go nanieść.
- **Dziel:** Kliknij podświetloną poziomą linię podziału w połączonej komórce, aby rozdzielić grupę w tym miejscu.
- **Duplikuj:** Kliknij numer wiersza albo pasek, aby zduplikować całą warstwę.
- **Scalaj:** Wybierz **Scal w dół** lub **Scal w górę**, a następnie kliknij komórkę fazy chronologicznej lub okresu datowania, aby połączyć ją z sąsiednią.

---

## Warstwy i grupy {#layers}

- **Wstawianie warstwy:** Kliknij cienką przerwę nad, pod lub między grupami, aby wstawić pustą warstwę.
- **Zmiana kolejności:** Dwukrotnie kliknij i przytrzymaj numer wiersza w kolumnie **Nr warstwy**, a następnie przeciągnij go pionowo, aby zmienić kolejność.
- **Edycja tekstu:** Dwukrotne kliknięcie dowolnej komórki tekstowej lub tytułu projektu rozpoczyna edycję w miejscu. `Enter` zatwierdza zmianę, `Escape` ją anuluje.

---

## Zapis i eksport {#files}

Projekty są zapisywane jako pliki `.json`, a eksport bezpośrednio do `.pdf` jest dostępny z menu **Plik** lub przycisku na pasku narzędzi. Eksportowane pliki PDF mogą osadzać dane projektu, dzięki czemu można je ponownie otworzyć do dalszej edycji; to zachowanie można wyłączyć w oknie **Parametry obszaru wyjściowego**.

Ustawienia eksportu (widoczne kolumny i parametry obszaru wyjściowego) są zapisywane w każdym pliku projektu. Aby przyjąć ustawienia z innego pliku lub pliku PDF bez otwierania go jako projektu, wybierz **Importuj z pliku...** w oknie parametrów obszaru wyjściowego.

---

## Skróty klawiszowe {#shortcuts}
Domyślne skróty:
```
Ctrl + O          Otwórz projekt (.json)
Ctrl + S          Zapisz projekt
Ctrl + Shift + S  Zapisz projekt jako...
Ctrl + E          Eksport do PDF
Ctrl + Z          Cofnij
Ctrl + Y          Ponów
E                 narzędzie Edytuj
C                 narzędzie Utwórz
D                 narzędzie Usuń
P                 narzędzie Maluj
S                 narzędzie Dziel
R                 narzędzie Duplikuj
M                 narzędzie Scalaj
]                 następny wariant
[                 poprzedni wariant
```
Skrót `Ctrl + kółko myszy` również przechodzi przez warianty aktywnego narzędzia. Wszystkie te skróty, a także dodatkowe przyciski myszy, można zmienić w oknie **Skróty klawiszowe**.

Powrót na [początek pomocy](#top) lub odczytaj [informacje o programie](doc:about).
"#.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Language, Translations};

    /// Every picker iterates `ALL` instead of hard-mapping indices; ALL must
    /// be exactly the declared variants, in order.
    #[test]
    fn all_lists_every_language_in_declaration_order() {
        for (i, lang) in Language::ALL.iter().enumerate() {
            assert_eq!(*lang as usize, i);
        }
    }

    #[test]
    fn codes_are_stable_and_round_trip() {
        for lang in Language::ALL {
            // Written to the settings file and read back at startup.
            assert_eq!(Language::from_code(lang.code()), Some(lang));
            // The native picker name must never collide with a code.
            assert_ne!(lang.display_name(), lang.code());
            // Every language must have usable translations for all fields.
            Translations::get(lang);
        }
    }

    #[test]
    fn next_wraps_through_every_language() {
        let mut seen = vec![Language::English];
        while *seen.last().unwrap() != seen[0] || seen.len() == 1 {
            let next = seen.last().unwrap().next();
            if seen.contains(&next) {
                break;
            }
            seen.push(next);
        }
        assert_eq!(seen.len(), Language::ALL.len(), "next() must visit all");
    }
}
