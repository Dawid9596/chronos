use crate::context::AppContext;
use crate::tool::Tool;
use chronos::bridge::{Effect, Event, ExportPdfResponse};
use chronos::bridge::{FilePickerRequest, FilePickerResponse};
use chronos::storage::StatusMessage;
use glib::source::timeout_add_local_once;
use gtk4::glib;
use gtk4::prelude::*;
use std::path::PathBuf;

fn cancel_status_timer(ctx: &AppContext) {
    if let Some(id) = ctx.status_timer.borrow_mut().take() {
        id.remove();
    }
}

fn schedule_status_restore(ctx: &AppContext) {
    cancel_status_timer(ctx);
    let ctx_idle = ctx.clone();
    let restore_id = timeout_add_local_once(std::time::Duration::from_secs(3), move || {
        ctx_idle.status_timer.borrow_mut().take(); // Clear ID without .remove() since it is firing
        let effects = ctx_idle.core.process_event(Event::ClearStatus);
        process_effects(&ctx_idle, effects);
    });
    *ctx.status_timer.borrow_mut() = Some(restore_id);
}

pub fn process_effects(ctx: &AppContext, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::Render(_) => {
                let view = ctx.core.view();
                let t = &view.translations;

                ctx.export_btn.set_sensitive(!view.is_exporting);
                ctx.export_btn.set_tooltip_text(Some(if view.is_exporting {
                    &t.exporting_pdf
                } else {
                    &t.export_pdf
                }));

                if let Some(msg) = &view.last_error {
                    let text = match msg {
                        StatusMessage::Info(s) => s.as_str(),
                        StatusMessage::Error(s) => s.as_str(),
                    };
                    let is_error = matches!(msg, StatusMessage::Error(_));
                    let is_new = ctx.last_message.borrow().as_deref() != Some(text);
                    if is_new {
                        *ctx.last_message.borrow_mut() = Some(text.to_string());
                        cancel_status_timer(ctx);
                        let color = if is_error { "red" } else { "green" };
                        let escaped = glib::markup_escape_text(text);
                        ctx.status.set_markup(&format!(
                            "<span foreground='{}'>{}</span>",
                            color, escaped
                        ));
                        if color == "green" {
                            schedule_status_restore(ctx);
                        }
                    }
                } else {
                    cancel_status_timer(ctx);
                    *ctx.last_message.borrow_mut() = None;
                    ctx.status.set_text(&format!(
                        "{}{}",
                        t.project_prefix,
                        if view.document.project_name.is_empty() {
                            &t.project_untitled
                        } else {
                            &view.document.project_name
                        }
                    ));
                }

                if ctx.current_lang.get() != t.language {
                    ctx.current_lang.set(t.language);
                    ctx.persist_settings();
                }

                for (tool_variant, btn) in ctx.variant_buttons.borrow().iter() {
                    if let Tool::Paint(pattern) = tool_variant {
                        btn.set_tooltip_text(Some(pattern.display_name(t.language)));
                    }
                }
                ctx.settings_label.set_markup(&t.settings_tooltip);
                ctx.menu_labels.update(t);
                ctx.shortcuts.update_translations(t);
                ctx.history_buttons.update_translations(t);
                ctx.history_buttons.sync(view.can_undo, view.can_redo);
                for (tool_variant, label) in ctx.tool_labels.iter() {
                    label.set_text(tool_variant.label(t));
                }

                crate::ui::table::sync_document_table(&view, ctx);
            }
            Effect::ExportPdf(mut request) => {
                let ctx = ctx.clone();
                glib::spawn_future_local(async move {
                    let file_name = request.operation.file_name.clone();
                    let view_model = request.operation.view_model.clone();
                    let project_json = request.operation.project_json.clone();

                    let response = gtk4::gio::spawn_blocking(move || {
                        let pdf_bytes = chronos::generate_pdf::typst_compile_to_bytes(
                            &view_model,
                            project_json.as_deref(),
                        )
                        .map_err(|e| format!("Error: {e}"))?;

                        let save_dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
                        let file_path = save_dir.join(file_name);

                        std::fs::write(&file_path, pdf_bytes).map_err(|e| format!("Error: {e}"))?;

                        let _ = open::that(&file_path);
                        Ok::<String, String>(format!("Saved to {}", file_path.display()))
                    })
                    .await
                    .unwrap_or_else(|_| Err("Error: PDF export worker panicked".to_string()));

                    let export_response = match response {
                        Ok(msg) => ExportPdfResponse::Success(msg),
                        Err(err) => ExportPdfResponse::Error(err),
                    };

                    let next_effects = ctx
                        .core
                        .resolve(&mut request, export_response)
                        .expect("Failed to resolve effect");
                    process_effects(&ctx, next_effects);
                });
            }
            Effect::FilePicker(mut request) => {
                let ctx = ctx.clone();
                glib::spawn_future_local(async move {
                    let response = match &request.operation {
                        FilePickerRequest::OpenFile { title, filter_name } => {
                            let dialog = gtk4::FileDialog::new();
                            dialog.set_title(title);

                            dialog.set_filters(Some(&crate::ui::project_io::project_file_filters(
                                filter_name,
                            )));

                            match dialog.open_future(Some(&ctx.window)).await {
                                Ok(file) => match file.path() {
                                    Some(path_buf) => {
                                        let path = path_buf.to_string_lossy().to_string();
                                        match crate::ui::project_io::read_project_text(&path_buf) {
                                            Ok(content) => {
                                                FilePickerResponse::Opened { path, content }
                                            }
                                            Err(e) => FilePickerResponse::Error(
                                                e.describe(&ctx.core.view().translations),
                                            ),
                                        }
                                    }
                                    None => FilePickerResponse::Error("Invalid file path".into()),
                                },
                                Err(_) => FilePickerResponse::Cancelled,
                            }
                        }
                        FilePickerRequest::SaveFile {
                            path,
                            content,
                            title,
                            filter_name,
                        } => {
                            let final_path: String = if let Some(p) = path {
                                p.clone()
                            } else {
                                let dialog = gtk4::FileDialog::new();
                                dialog.set_title(title);
                                dialog.set_initial_name(Some("project.json"));

                                let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
                                let filter = gtk4::FileFilter::new();
                                filter.add_pattern("*.json");
                                filter.add_suffix("json");
                                filter.set_name(Some(filter_name));
                                filters.append(&filter);
                                dialog.set_filters(Some(&filters));

                                match dialog.save_future(Some(&ctx.window)).await {
                                    Ok(file) => match file.path() {
                                        Some(p) => p.to_string_lossy().to_string(),
                                        None => {
                                            let next_effects = ctx
                                                .core
                                                .resolve(
                                                    &mut request,
                                                    FilePickerResponse::Cancelled,
                                                )
                                                .expect("resolve");
                                            process_effects(&ctx, next_effects);
                                            return;
                                        }
                                    },
                                    Err(_) => {
                                        let next_effects = ctx
                                            .core
                                            .resolve(&mut request, FilePickerResponse::Cancelled)
                                            .expect("resolve");
                                        process_effects(&ctx, next_effects);
                                        return;
                                    }
                                }
                            };

                            match std::fs::write(&final_path, content) {
                                Ok(_) => FilePickerResponse::Saved { path: final_path },
                                Err(e) => FilePickerResponse::Error(e.to_string()),
                            }
                        }
                    };

                    let next_effects = ctx.core.resolve(&mut request, response).expect("resolve");
                    process_effects(&ctx, next_effects);
                });
            }
        }
    }
}
