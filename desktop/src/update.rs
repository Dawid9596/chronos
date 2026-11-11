//! Update check against the project's GitHub Releases.
//!
//! Fired automatically at most once a day when the main window is opened.

use crate::context::AppContext;
use gtk4::glib;
use gtk4::prelude::*;

const RELEASES_API_URL: &str = "https://api.github.com/repos/Dawid9596/chronos/releases/latest";
const RELEASES_PAGE_URL: &str = "https://github.com/Dawid9596/chronos/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub fn schedule_update_check(ctx: &AppContext) {
    if ctx.last_update_check_day.get() == Some(today_day_number()) {
        return; // Already checked today.
    }
    let ctx = ctx.clone();
    glib::spawn_future_local(async move {
        run_check(&ctx, CheckKind::Automatic).await;
    });
}

pub fn check_now(ctx: &AppContext) {
    let ctx = ctx.clone();
    glib::spawn_future_local(async move {
        run_check(&ctx, CheckKind::Manual).await;
    });
}

enum CheckKind {
    Automatic,
    Manual,
}

async fn run_check(ctx: &AppContext, kind: CheckKind) {
    let latest = gtk4::gio::spawn_blocking(fetch_latest_version)
        .await
        .ok()
        .flatten();

    let Some(latest) = latest else {
        if matches!(kind, CheckKind::Manual) {
            let t = ctx.core.view().translations;
            show_alert(ctx, &t.update_check_failed);
        }
        return;
    };

    ctx.last_update_check_day.set(Some(today_day_number()));
    ctx.persist_settings();

    if is_newer(&latest, CURRENT_VERSION) {
        if matches!(kind, CheckKind::Automatic)
            && ctx.skipped_version.borrow().as_deref() == Some(latest.as_str())
        {
            return; // User chose to skip this release.
        }
        show_update_dialog(ctx, &latest);
    } else if matches!(kind, CheckKind::Manual) {
        let t = ctx.core.view().translations;
        show_alert(
            ctx,
            &t.update_up_to_date.replace("{current}", CURRENT_VERSION),
        );
    }
}

/// Fetches the latest release tag from the GitHub API, `None` on any failure.
fn fetch_latest_version() -> Option<String> {
    let text = ureq::AgentBuilder::new()
        .timeout(TIMEOUT)
        .build()
        .get(RELEASES_API_URL)
        .set("User-Agent", &format!("chronos/{CURRENT_VERSION}"))
        .set("Accept", "application/vnd.github+json")
        .call()
        .ok()?
        .into_string()
        .ok()?;

    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let tag = json.get("tag_name")?.as_str()?;
    Some(tag.trim_start_matches(['v', 'V']).to_string())
}

/// Days since the epoch, in UTC; persisted as `Settings::last_update_check_day`.
fn today_day_number() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0)
}

/// Numeric dot-separated comparison (`v0.5.0` > `v0.4.11` > `v0.4.9`);
/// extra components on either side count as zero. Non-numeric parts parse
/// as zero, so pre-release-like tags never look newer than their base.
fn is_newer(latest: &str, current: &str) -> bool {
    fn parse(version: &str) -> Vec<u64> {
        version
            .trim_start_matches(['v', 'V', ' '])
            .split('.')
            .map(|part| {
                part.split(['-', '+'])
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0)
            })
            .collect()
    }

    let (latest, current) = (parse(latest), parse(current));
    for i in 0..latest.len().max(current.len()) {
        let l = latest.get(i).copied().unwrap_or(0);
        let c = current.get(i).copied().unwrap_or(0);
        if l != c {
            return l > c;
        }
    }
    false
}

fn show_alert(ctx: &AppContext, message: &str) {
    gtk4::AlertDialog::builder()
        .message(message)
        .modal(true)
        .build()
        .show(Some(&ctx.window));
}

fn show_update_dialog(ctx: &AppContext, latest_version: &str) {
    let t = ctx.core.view().translations;

    let win = gtk4::Window::builder()
        .transient_for(&ctx.window)
        .modal(true)
        .title(&t.update_available_title)
        .default_width(380)
        .resizable(false)
        .build();

    let vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let message = gtk4::Label::builder()
        .label(
            t.update_available_body
                .replace("{version}", latest_version)
                .replace("{current}", CURRENT_VERSION),
        )
        .wrap(true)
        .xalign(0.0)
        .build();
    vbox.append(&message);

    let button_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::End)
        .build();

    let close_btn = gtk4::Button::with_label(&t.dialog_close);
    let win_close = win.clone();
    close_btn.connect_clicked(move |_| win_close.close());
    button_row.append(&close_btn);

    let skip_btn = gtk4::Button::with_label(&t.update_skip_version);
    {
        let ctx = ctx.clone();
        let win_skip = win.clone();
        let latest = latest_version.to_string();
        skip_btn.connect_clicked(move |_| {
            *ctx.skipped_version.borrow_mut() = Some(latest.clone());
            ctx.save_settings_now();
            win_skip.close();
        });
    }
    button_row.append(&skip_btn);

    let open_btn = gtk4::Button::with_label(&t.update_open_releases);
    open_btn.add_css_class("suggested-action");
    let win_open = win.clone();
    open_btn.connect_clicked(move |_| {
        let _ = open::that(RELEASES_PAGE_URL);
        win_open.close();
    });
    button_row.append(&open_btn);

    vbox.append(&button_row);

    win.set_child(Some(&vbox));
    win.present();
}
