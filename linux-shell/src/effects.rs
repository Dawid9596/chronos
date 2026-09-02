use crate::context::AppContext;
use chronos::bridge::{Effect, ExportPdfResponse};
use chronos::bridge::{FilePickerRequest, FilePickerResponse};
use gtk4::glib;
use gtk4::prelude::*;
use std::path::PathBuf;

pub fn process_effects(ctx: &AppContext, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::Render(_) => {
                let view = ctx.core.view();
                let t = &view.translations;

                ctx.export_btn.set_sensitive(!view.is_exporting);
                ctx.export_btn.set_label(if view.is_exporting {
                    &t.exporting_pdf
                } else {
                    &t.export_pdf
                });

                if let Some(msg) = &view.last_error {
                    let color = if msg.starts_with("Error") {
                        "red"
                    } else {
                        "green"
                    };
                    ctx.status
                        .set_markup(&format!("<span foreground='{}'>{}</span>", color, msg));
                } else {
                    ctx.status
                        .set_text(&format!("Project: {}", view.document.project_name));
                }

                ctx.lang_btn.set_label(&t.flag_emoji);
                ctx.settings_btn.set_label(&t.settings_tooltip);
                ctx.edit_btn.set_label(&t.tool_edit);
                ctx.create_btn.set_label(&t.tool_create);
                ctx.delete_text_label.set_text(&t.tool_delete);

                let new_table = crate::ui::table::build_document_table(&view, ctx);
                ctx.scrolled_window.set_child(Some(&new_table));
            }
            Effect::ExportPdf(mut request) => {
                let ctx = ctx.clone();
                glib::spawn_future_local(async move {
                    let file_name = request.operation.file_name.clone();
                    let pdf_bytes = request.operation.pdf_bytes.clone();
                    let save_dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
                    let file_path = save_dir.join(file_name);

                    let response = match std::fs::write(&file_path, pdf_bytes) {
                        Ok(_) => {
                            let _ = std::process::Command::new("xdg-open")
                                .arg(&file_path)
                                .spawn();
                            ExportPdfResponse::Success(format!("Saved to {}", file_path.display()))
                        }
                        Err(e) => ExportPdfResponse::Error(format!("Error: {}", e)),
                    };

                    let next_effects = ctx
                        .core
                        .resolve(&mut request, response)
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

                            let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
                            let filter = gtk4::FileFilter::new();
                            filter.add_mime_type("application/json");
                            filter.add_suffix("json");
                            filter.set_name(Some(filter_name));
                            filters.append(&filter);
                            dialog.set_filters(Some(&filters));

                            match dialog.open_future(None::<&gtk4::Window>).await {
                                Ok(file) => {
                                    let path = file.path().unwrap().to_string_lossy().to_string();
                                    match std::fs::read_to_string(&path) {
                                        Ok(content) => FilePickerResponse::Opened { path, content },
                                        Err(e) => FilePickerResponse::Error(e.to_string()),
                                    }
                                }
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

                                let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
                                let filter = gtk4::FileFilter::new();
                                filter.add_mime_type("application/json");
                                filter.add_suffix("json");
                                filter.set_name(Some(filter_name));
                                filters.append(&filter);
                                dialog.set_filters(Some(&filters));

                                match dialog.save_future(None::<&gtk4::Window>).await {
                                    // Return an owned String here
                                    Ok(file) => file.path().unwrap().to_string_lossy().to_string(),
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
