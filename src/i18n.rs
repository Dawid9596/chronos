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

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Translations {
    pub language: Language,
    // Menus
    pub menu_file: &'static str,
    pub menu_edit: &'static str,
    pub menu_about: &'static str,
    pub menu_help: &'static str,
    pub menu_check_updates: &'static str,
    pub menu_open: &'static str,
    pub menu_save: &'static str,
    pub menu_save_as: &'static str,
    pub menu_export: &'static str,
    pub menu_canvas_params: &'static str,
    pub menu_preferences: &'static str,
    pub menu_shortcuts: &'static str,
    pub menu_undo: &'static str,
    pub menu_redo: &'static str,
    // Columns
    pub col_no: &'static str,
    pub col_graphical: &'static str,
    pub col_woven: &'static str,
    pub col_char: &'static str,
    pub col_thickness: &'static str,
    pub col_chrono: &'static str,
    pub col_dating: &'static str,
    // Shortened (non-destructive) table-column titles
    pub col_no_short: &'static str,
    pub col_graphical_short: &'static str,
    pub col_woven_short: &'static str,
    pub col_char_short: &'static str,
    pub col_thickness_short: &'static str,
    pub col_chrono_short: &'static str,
    pub col_dating_short: &'static str,
    // Toolbar & Controls
    pub export_pdf: &'static str,
    pub exporting_pdf: &'static str,
    pub tool_edit: &'static str,
    pub tool_create: &'static str,
    pub tool_delete: &'static str,
    pub tool_paint: &'static str,
    pub tool_split: &'static str,
    pub tool_duplicate: &'static str,
    pub tool_merge: &'static str,
    /// The PDF footer credits the pattern artwork (CC BY-SA 4.0 requires it);
    /// passed to the typst template as plain text, already in this language.
    pub artwork_attribution: &'static str,
    pub status_ready: &'static str,
    pub project_prefix: &'static str,
    pub project_untitled: &'static str,
    /// Shown in the header as a click-to-edit hint while the report
    /// subtitle is empty.
    pub subtitle_placeholder: &'static str,
    // Settings Popover
    /// Legacy, Noto font is not bundled
    pub settings_tooltip: &'static str,
    pub delete_label: &'static str,
    // File Dialogs
    pub dialog_open_title: &'static str,
    pub dialog_save_title: &'static str,
    pub filter_json_name: &'static str,
    pub filter_project_name: &'static str,
    pub dialog_restore_defaults: &'static str,
    pub dialog_close: &'static str,
    pub update_available_title: &'static str,
    /// `{version}` is replaced with the latest release version, `{current}`
    /// with the running program version.
    pub update_available_body: &'static str,
    pub update_open_releases: &'static str,
    /// Silences the automatic check for the shown release.
    pub update_skip_version: &'static str,
    /// `{current}` is replaced with the running program version.
    pub update_up_to_date: &'static str,
    pub update_check_failed: &'static str,
    // Canvas Parameters & Preferences Dialogs
    pub dialog_paper_size: &'static str,
    pub dialog_landscape: &'static str,
    pub dialog_page_margin: &'static str,
    pub dialog_cell_padding: &'static str,
    pub dialog_bar_padding: &'static str,
    pub dialog_zero_width_line: &'static str,
    pub dialog_visible_columns_heading: &'static str,
    pub dialog_interface_language: &'static str,
    pub dialog_embed_project_data: &'static str,
    pub dialog_import_from_file: &'static str,
    // Keyboard Shortcuts Dialog & Tooltips
    pub shortcut_tooltip_prefix: &'static str,
    pub shortcuts_section_tools: &'static str,
    pub shortcuts_section_variants: &'static str,
    pub shortcuts_section_fixed: &'static str,
    pub action_next_variant: &'static str,
    pub action_prev_variant: &'static str,
    pub shortcuts_fixed_inline_edit: &'static str,
    pub shortcuts_fixed_cancel_edit: &'static str,
    pub shortcuts_key_double_click: &'static str,
    pub shortcuts_press_new: &'static str,
    pub shortcuts_prompt_cancel: &'static str,
    pub shortcuts_click_tooltip: &'static str,
    pub shortcuts_restore_default_tooltip: &'static str,
    pub shortcuts_restore_all: &'static str,
    pub shortcuts_err_invalid: &'static str,
    pub shortcuts_err_needs_modifier: &'static str,
    pub shortcuts_err_conflict_prefix: &'static str,
    // Status & Error Prefixes
    pub status_loaded_prefix: &'static str,
    pub status_saved_prefix: &'static str,
    /// Shown after silently restoring crash-recovery autosave on startup.
    pub status_recovered: &'static str,
    pub error_parse_prefix: &'static str,
    pub error_open_prefix: &'static str,
    pub error_save_prefix: &'static str,
    /// Prefix for `PdfProjectError::Unreadable` details.
    pub error_pdf_unreadable_prefix: &'static str,
    pub error_no_project_data: &'static str,
    pub error_no_export_settings: &'static str,
    pub error_newer_format: &'static str,
    pub warning_document_sanitized: &'static str,
    pub warning_export_settings_sanitized: &'static str,
    pub warning_export_settings_unreadable_prefix: &'static str,
    // Help & About documentation windows
    pub doc_about_title: &'static str,
    pub doc_help_title: &'static str,
    pub about_markdown: &'static str,
    pub help_markdown: &'static str,
}

impl Translations {
    /// The shortened table-column title by column index
    #[must_use]
    pub fn col_short(&self, index: usize) -> &str {
        match index {
            0 => self.col_no_short,
            1 => self.col_graphical_short,
            2 => self.col_woven_short,
            3 => self.col_char_short,
            4 => self.col_thickness_short,
            5 => self.col_chrono_short,
            _ => self.col_dating_short,
        }
    }

    pub fn get(lang: Language) -> Self {
        match lang {
            Language::English => Self {
                language: Language::English,
                menu_file: "File",
                menu_edit: "Edit",
                menu_about: "About",
                menu_help: "Help",
                menu_check_updates: "Check for Updates...",
                menu_open: "Open...",
                menu_save: "Save",
                menu_save_as: "Save As...",
                menu_export: "Export PDF",
                menu_canvas_params: "Output Canvas Parameters...",
                menu_preferences: "Preferences...",
                menu_shortcuts: "Keyboard Shortcuts...",
                menu_undo: "Undo",
                menu_redo: "Redo",
                col_no: "No.",
                col_graphical: "Graphical representation\nof the layer",
                col_woven: "Woven trim",
                col_char: "Layer characteristics",
                col_thickness: "Layer thickness",
                col_chrono: "Chronological phase",
                col_dating: "Dating period",
                col_no_short: "No.",
                col_graphical_short: "Graphical representation",
                col_woven_short: "Woven trim",
                col_char_short: "Characteristics",
                col_thickness_short: "Thickness",
                col_chrono_short: "Chronology",
                col_dating_short: "Dating",
                export_pdf: "Export PDF",
                exporting_pdf: "Exporting...",
                tool_edit: "Edit",
                tool_create: "Create",
                tool_delete: "Delete",
                tool_paint: "Paint",
                tool_split: "Split",
                tool_duplicate: "Duplicate",
                tool_merge: "Merge",
                artwork_attribution: "Pattern artwork: @kationart, CC BY-SA 4.0",
                status_ready: "Ready",
                project_prefix: "Project: ",
                project_untitled: "Untitled",
                subtitle_placeholder: "Add subtitle…",
                settings_tooltip: "<span font_family='Noto Color Emoji'>⚙️</span>",
                delete_label: "Delete",
                dialog_open_title: "Open File",
                dialog_save_title: "Save File",
                filter_json_name: "JSON Files",
                filter_project_name: "Project files (JSON, PDF)",
                dialog_restore_defaults: "Restore Defaults",
                dialog_close: "Close",
                update_available_title: "Update available",
                update_available_body: "Version {version} is now available. You are running version {current}.",
                update_open_releases: "Open Releases Page",
                update_skip_version: "Skip This Version",
                update_up_to_date: "You are running the latest version ({current}).",
                update_check_failed: "Could not check for updates.",
                dialog_paper_size: "Paper Size:",
                dialog_landscape: "Landscape Orientation",
                dialog_page_margin: "Page Margin (mm):",
                dialog_cell_padding: "Cell Padding (mm):",
                dialog_bar_padding: "Pattern Cell Padding (mm):",
                dialog_zero_width_line: "Zero-Width Line (pt):",
                dialog_visible_columns_heading: "Visible Output Columns & Ratios",
                dialog_interface_language: "Interface Language:",
                dialog_embed_project_data: "Embed project data in PDF",
                dialog_import_from_file: "Import from file...",
                shortcut_tooltip_prefix: "Shortcut: ",
                shortcuts_section_tools: "Tools",
                shortcuts_section_variants: "Variants",
                shortcuts_section_fixed: "Fixed",
                action_next_variant: "Next Variant",
                action_prev_variant: "Previous Variant",
                shortcuts_fixed_inline_edit: "Inline Edit / Reorder Layer",
                shortcuts_fixed_cancel_edit: "Cancel Inline Edit",
                shortcuts_key_double_click: "Double-Click",
                shortcuts_press_new: "Press new shortcut...",
                shortcuts_prompt_cancel: "Press the new shortcut, or Esc to cancel.",
                shortcuts_click_tooltip: "Click, then press the new shortcut",
                shortcuts_restore_default_tooltip: "Restore default",
                shortcuts_restore_all: "Restore all defaults",
                shortcuts_err_invalid: "That key combination can't be used.",
                shortcuts_err_needs_modifier: "File shortcuts must include Ctrl or Alt.",
                shortcuts_err_conflict_prefix: "Already used by ",
                status_loaded_prefix: "Loaded: ",
                status_saved_prefix: "Saved: ",
                status_recovered: "Recovered unsaved changes.",
                error_parse_prefix: "Error parsing file: ",
                error_open_prefix: "Error opening file: ",
                error_save_prefix: "Error saving file: ",
                error_pdf_unreadable_prefix: "Cannot read PDF: ",
                error_no_project_data: "This PDF has no embedded project data. Only PDFs exported with project data embedded can be opened.",
                error_no_export_settings: "This file contains no export settings.",
                error_newer_format: "this file was written by a newer version of the program (file format {}, supported up to {})",
                warning_document_sanitized: "Document contained invalid values and was adjusted.",
                warning_export_settings_sanitized: "Export settings contained invalid values and were adjusted.",
                warning_export_settings_unreadable_prefix: "Export settings could not be read and were ignored: ",
                doc_about_title: "About Chronos",
                doc_help_title: "Chronos Help",
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
"#,
                help_markdown: r#"# Chronos Help & Documentation

Welcome to the Conservation Editor documentation. Use the internal links below to jump to a topic:

- [Working with Tools](#tools)
- [Managing Layers & Groups](#layers)
- [Text Formatting](#formatting)
- [Saving & Exporting](#files)
- [Keyboard Shortcuts](#shortcuts)
- [About Chronos](doc:about)

---

## Working with Tools {#tools}

The top toolbar provides interactive tools for editing condition bars and cells:

- **Edit:** You can drag slot edges, or click a bar to cycle through patterns. Right-click cycles backward.
- **Create:** Click inside a condition bar to insert a new slot at the cursor position.
- **Delete:** Click an existing slot inside a condition bar to remove it.
- **Paint:** Select one of the 29 patterns (Bolus, Brick, Ceramics, Wood, Textile, Metal Foil, Embroidery, Stone, Glue, Bone, Spray Varnish, Painting, Metal, Paper, Patina, Parchment, Whitewash, Leather, Stucco, Glaze, Plasterwork, Cardboard, Plastic, Mixed Thread, Varnish, Wax, Nonwoven, Mortar, Gold Powder) and click any bar to apply it. There is search function available.
- **Split:** Click the highlighted horizontal divider inside a merged cell to split the group at that boundary.
- **Duplicate:** Click a row number or condition bar to duplicate the entire stratigraphic layer.
- **Merge:** Choose **Merge Down** or **Merge Up** and click a Chronological Phase or Dating Period cell to merge it with its neighbor.

---

## Managing Layers & Groups {#layers}

- **Insert Layer:** Click the thin gap above, below, or between groups to insert a blank layer.
- **Reorder Layer:** Double-click and hold a row number in the **No.** column, then drag vertically to reorder.
- **Edit Text:** Double-click any text cell or the project title to edit inline. Press `Enter` to commit or `Escape` to cancel.

---

## Text Formatting {#formatting}

This editor does not have sophisticated text editing capabilities.

Nonetheless all editable text — the table cells and the report subtitle — accepts a small markup inspired by Markdown. It is typed as-is while editing and rendered as soon as you click away, both on screen and in the exported PDF:

- `**bold**` is **bold**
- `*italic*` is *italic*
- `^14^C` is a superscript (^14^C)
- `m~3~` is a subscript (m~3~)

Markup characters can be printed literally by escaping them with a backslash: `\*`, `\_`, `\^`, `\~`. Styles combine freely, e.g. `**bold *and* italic**`.

The project title is always plain text and stays out of the markup.

---

## Saving & Exporting {#files}

Projects are saved as `.chr` files and can be exported directly to `.pdf` from the **File** menu or the toolbar button. Exported PDFs can carry the embedded project, so they can be re-opened for editing; that behaviour can be switched off in the **Output Canvas Parameters** dialog.

Export settings (column visibility and canvas parameters) are stored in every Save file. To apply settings from another file or from a PDF without opening it as a project, choose **Import from file...** in the output canvas parameters dialog.

---

## Keyboard Shortcuts {#shortcuts}
Defaults are
```
Ctrl + O          Open Project
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
`Ctrl + Mouse wheel` also cycles through the active tool's variants (up = next, down = previous).

Return to [Top of Help](#top) or read [About Conservation Editor](doc:about).
"#,
            },
            Language::Polish => Self {
                language: Language::Polish,
                menu_file: "Plik",
                menu_edit: "Edycja",
                menu_about: "O programie",
                menu_help: "Pomoc",
                menu_check_updates: "Sprawdź aktualizacje...",
                menu_open: "Otwórz...",
                menu_save: "Zapisz",
                menu_save_as: "Zapisz jako...",
                menu_export: "Eksportuj PDF",
                menu_canvas_params: "Parametry obszaru wyjściowego...",
                menu_preferences: "Preferencje...",
                menu_shortcuts: "Skróty klawiszowe...",
                menu_undo: "Cofnij",
                menu_redo: "Ponów",
                col_no: "Nr warstwy",
                col_graphical: "Oznaczenie graficzne warstwy",
                col_woven: "Krajka",
                col_char: "Charakterystyka warstwy",
                col_thickness: "Grubość warstwy",
                col_chrono: "Faza\nchronologiczna",
                col_dating: "Datowanie",
                col_no_short: "Nr",
                col_graphical_short: "Oznaczenie graficzne",
                col_woven_short: "Krajka",
                col_char_short: "Charakterystyka",
                col_thickness_short: "Grubość",
                col_chrono_short: "Faza",
                col_dating_short: "Datowanie",
                export_pdf: "Eksportuj PDF",
                exporting_pdf: "Eksportuję...",
                tool_edit: "Edytuj",
                tool_create: "Utwórz",
                tool_delete: "Usuń",
                tool_paint: "Maluj",
                tool_split: "Rozdziel",
                tool_duplicate: "Duplikuj",
                tool_merge: "Scal",
                artwork_attribution: "Grafika wzorów: @kationart, CC BY-SA 4.0",
                status_ready: "Gotowy",
                project_prefix: "Projekt: ",
                project_untitled: "Bez nazwy",
                subtitle_placeholder: "Dodaj podtytuł…",
                settings_tooltip: "<span font_family='Noto Color Emoji'>⚙️</span>",
                delete_label: "Usuń",
                dialog_open_title: "Otwórz plik",
                dialog_save_title: "Zapisz plik",
                filter_json_name: "Pliki JSON",
                filter_project_name: "Pliki projektu (JSON, PDF)",
                dialog_restore_defaults: "Przywróć domyślne",
                dialog_close: "Zamknij",
                update_available_title: "Dostępna aktualizacja",
                update_available_body: "Dostępna jest wersja {version}. Używasz wersji {current}.",
                update_open_releases: "Otwórz stronę wydań",
                update_skip_version: "Pomiń tę wersję",
                update_up_to_date: "Korzystasz z najnowszej wersji ({current}).",
                update_check_failed: "Nie udało się sprawdzić aktualizacji.",
                dialog_paper_size: "Format papieru:",
                dialog_landscape: "Orientacja pozioma",
                dialog_page_margin: "Margines strony (mm):",
                dialog_cell_padding: "Dopełnienie komórek (mm):",
                dialog_bar_padding: "Dopełnienie komórek wzorów (mm):",
                dialog_zero_width_line: "Linia zerowej szerokości (pt):",
                dialog_visible_columns_heading: "Widoczne kolumny wyjściowe i proporcje",
                dialog_interface_language: "Język interfejsu:",
                dialog_embed_project_data: "Osadź dane projektu w PDF",
                dialog_import_from_file: "Importuj z pliku...",
                shortcut_tooltip_prefix: "Skrót: ",
                shortcuts_section_tools: "Narzędzia",
                shortcuts_section_variants: "Warianty",
                shortcuts_section_fixed: "Stałe",
                action_next_variant: "Następny wariant",
                action_prev_variant: "Poprzedni wariant",
                shortcuts_fixed_inline_edit: "Edycja w miejscu / Zmiana kolejności warstwy",
                shortcuts_fixed_cancel_edit: "Anuluj edycję w miejscu",
                shortcuts_key_double_click: "Podwójne kliknięcie",
                shortcuts_press_new: "Naciśnij nowy skrót...",
                shortcuts_prompt_cancel: "Naciśnij nowy skrót lub Esc, aby anulować.",
                shortcuts_click_tooltip: "Kliknij, a następnie naciśnij nowy skrót",
                shortcuts_restore_default_tooltip: "Przywróć domyślny",
                shortcuts_restore_all: "Przywróć wszystkie domyślne",
                shortcuts_err_invalid: "Nie można użyć tej kombinacji klawiszy.",
                shortcuts_err_needs_modifier: "Skróty plikowe muszą zawierać Ctrl lub Alt.",
                shortcuts_err_conflict_prefix: "Już używane przez ",
                status_loaded_prefix: "Wczytano: ",
                status_saved_prefix: "Zapisano: ",
                status_recovered: "Przywrócono niezapisane zmiany.",
                error_parse_prefix: "Błąd parsowania pliku: ",
                error_open_prefix: "Błąd otwierania pliku: ",
                error_save_prefix: "Błąd zapisywania pliku: ",
                error_pdf_unreadable_prefix: "Nie można odczytać pliku PDF: ",
                error_no_project_data: "Ten plik PDF nie zawiera osadzonych danych projektu. Można otworzyć tylko pliki PDF wyeksportowane z osadzonymi danymi projektu.",
                error_no_export_settings: "Ten plik nie zawiera ustawień eksportu.",
                error_newer_format: "ten plik został zapisany przez nowszą wersję programu (format pliku {}, obsługiwane do {})",
                warning_document_sanitized: "Dokument zawierał nieprawidłowe wartości i został dostosowany.",
                warning_export_settings_sanitized: "Ustawienia eksportu zawierały nieprawidłowe wartości, które zostały zignorowane.",
                warning_export_settings_unreadable_prefix: "Nie udało się odczytać ustawień eksportu i zostały zignorowane: ",
                doc_about_title: "O programie",
                doc_help_title: "Pomoc Chronos",
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
"#,
                help_markdown: r#"# Pomoc i dokumentacja Chronos

Witaj w dokumentacji Conservation Editor. Odnośniki poniżej prowadzą do kolejnych tematów:

- [Praca z narzędziami](#tools)
- [Warstwy i grupy](#layers)
- [Formatowanie tekstu](#formatting)
- [Zapis i eksport](#files)
- [Skróty klawiszowe](#shortcuts)
- [O programie](doc:about)

---

## Praca z narzędziami {#tools}

Pasek narzędzi na górze okna zawiera narzędzia przeznaczone do edycji pasków stanu zachowania oraz komórek:

- **Edytuj:** Możesz przeciągać krawędzie slotów, lub klikaj pasek, aby przechodzić przez wzory; prawy przycisk myszy przechodzi przez nie od tyłu.
- **Utwórz:** Kliknięcie wewnątrz paska wstawia nowy slot w miejscu kursora.
- **Usuń:** Kliknięcie istniejącego slotu wewnątrz paska usuwa go.
- **Maluj:** Wybierz jeden z 29 wzorów (Bolus, Cegła, Ceramika, Drewno, Tkanina, Folie metalowe, Haft, Kamień, Klej, Kość, Laserunek, Malowidło, Metal, Papier, Patyna, Pergamin, Pobiała, Skóra, Stiuk, Szkliwo, Sztukateria, Tektura, Tworzywo sztuczne, Wątek mieszany, Werniks, Wosk, Włóknina, Zaprawa, Złoto w proszku) i kliknij dowolny pasek, aby go nanieść. Wyszukiwanie po nazwie jest dostępne.
- **Dziel:** Kliknij podświetloną poziomą linię podziału w połączonej komórce, aby rozdzielić grupę w tym miejscu.
- **Duplikuj:** Kliknij numer wiersza albo pasek, aby zduplikować całą warstwę.
- **Scalaj:** Wybierz **Scal w dół** lub **Scal w górę**, a następnie kliknij komórkę fazy chronologicznej lub okresu datowania, aby połączyć ją z sąsiednią.

---

## Warstwy i grupy {#layers}

- **Wstawianie warstwy:** Kliknij cienką przerwę nad, pod lub między grupami, aby wstawić pustą warstwę.
- **Zmiana kolejności:** Dwukrotnie kliknij i przytrzymaj numer wiersza w kolumnie **Nr warstwy**, a następnie przeciągnij go pionowo, aby zmienić kolejność.
- **Edycja tekstu:** Dwukrotne kliknięcie dowolnej komórki tekstowej lub tytułu projektu rozpoczyna edycję w miejscu. `Enter` zatwierdza zmianę, `Escape` ją anuluje.

---

## Formatowanie tekstu {#formatting}

Ten edytor nie posiada zaawansowanych możliwości edycji tekstu.

Niemniej cały edytowalny tekst — komórki tabeli i podtytuł raportu — przyjmuje prostą składnię zainspirowaną Markdownem. Wpisuje się ją dosłownie podczas edycji, a po kliknięciu poza pole jest renderowana — na ekranie i w eksportowanym PDF-ie:

- `**pogrubienie**` to **pogrubienie**
- `*kursywa*` to *kursywa*
- `^14^C` to indeks górny (^14^C)
- `m~3~` to indeks dolny (m~3~)

Znaki składni można wydrukować dosłownie, poprzedzając je ukośnikiem: `\*`, `\_`, `\^`, `\~`. Style można dowolnie łączyć, np. `**pogrubienie *i* kursywa**`.

Tytuł projektu jest zawsze zwykłym tekstem i nie podlega formatowaniu.

---

## Zapis i eksport {#files}

Projekty są zapisywane jako pliki `.chr`, a eksport bezpośrednio do `.pdf` jest dostępny z menu **Plik** lub przycisku na pasku narzędzi. Eksportowane pliki PDF mogą osadzać dane projektu, dzięki czemu można je ponownie otworzyć do dalszej edycji; to zachowanie można wyłączyć w oknie **Parametry obszaru wyjściowego**.

Ustawienia eksportu (widoczne kolumny i parametry obszaru wyjściowego) są zapisywane w każdym pliku projektu. Aby przyjąć ustawienia z innego pliku lub pliku PDF bez otwierania go jako projektu, wybierz **Importuj z pliku...** w oknie parametrów obszaru wyjściowego.

---

## Skróty klawiszowe {#shortcuts}
Domyślne skróty:
```
Ctrl + O          Otwórz projekt
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
Skrót `Ctrl + kółko myszy` również przechodzi przez warianty aktywnego narzędzia (w górę = następny, w dół = poprzedni).

Powrót na [początek pomocy](#top) lub odczytaj [informacje o programie](doc:about).
"#,
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
