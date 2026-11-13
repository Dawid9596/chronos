//! Crash-resilient autosave.
//!
//! The core emits an `Effect::Autosave` whenever the document changes.
//! The shell coalesces those into one debounced write of
//! the regular `ProjectFile` envelope to `data_dir/chronos/autosave.json`,
//! off the UI thread.
//!
//! Invariant: the autosave file exists only while a session is running with
//! changes not persisted anywhere else, and is deleted by every normal exit
//! path. So on the next launch, a surviving file means the previous session
//! ended abnormally — and it is subsequently restored.

use crate::context::AppContext;
use chronos::bridge::Event;
use chronos::project::ProjectFile;
use chronos::storage::ConservationDocument;
use gtk4::glib;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

const DEBOUNCE: std::time::Duration = std::time::Duration::from_secs(1);

#[derive(Default)]
pub struct AutosaveState {
    /// The most recent document snapshot, replaced by every change effect.
    latest: Option<ConservationDocument>,
    timer: Option<glib::SourceId>,
    /// A write is running; new changes must not start a second one.
    in_flight: Cell<bool>,
    /// Bumped by `clear`/`on_exit`; an in-flight write whose generation no
    /// longer matches must not resurrect the file after it was deleted.
    generation: Cell<u64>,
    /// Set when the session is closing: no further writes, ever.
    exiting: Cell<bool>,
}

fn autosave_path() -> Option<PathBuf> {
    let app_dir = if cfg!(windows) { "Chronos" } else { "chronos" };
    dirs::data_dir().map(|dir| dir.join(app_dir).join("autosave.json"))
}

/// Called from the effect loop for every document change. Starts (or resets)
/// the debounced write; while a write is in flight the snapshot is merely
/// stored, and the write completion picks it up.
pub fn note_document(ctx: &AppContext, document: ConservationDocument) {
    let schedule = {
        let mut state = ctx.autosave.borrow_mut();
        if state.exiting.get() {
            return;
        }
        state.latest = Some(document);
        !state.in_flight.get()
    };
    if schedule {
        schedule_write(ctx);
    }
}

/// The work just became fully persisted (explicit save) or was replaced
/// (opening another file): drop the pending state and any file on disk.
pub fn clear(ctx: &AppContext) {
    {
        let mut state = ctx.autosave.borrow_mut();
        state.generation.set(state.generation.get() + 1);
        state.latest = None;
        if let Some(id) = state.timer.take() {
            id.remove();
        }
    }
    delete_files();
}

/// Finalize the session: no further writes, pending work discarded, file
/// removed — a normal exit must never leave a recoverable autosave behind.
pub fn on_exit(ctx: &AppContext) {
    {
        let mut state = ctx.autosave.borrow_mut();
        state.exiting.set(true);
        state.generation.set(state.generation.get() + 1);
        if let Some(id) = state.timer.take() {
            id.remove();
        }
    }
    delete_files();
}

/// On startup: a surviving autosave means the previous session crashed;
/// restore it through a `RestoreDocument` event and consume the file.
pub fn try_recover(ctx: &AppContext) {
    let Some(path) = autosave_path() else {
        return;
    };
    // A leftover tmp from an interrupted exit is junk in every scenario.
    let _ = std::fs::remove_file(path.with_extension("json.tmp"));
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
        Err(e) => {
            eprintln!("autosave: cannot read {}: {e}", path.display());
            let _ = std::fs::remove_file(&path);
            return;
        }
    };
    match ProjectFile::parse(&text, ctx.current_lang.get()) {
        Ok(project) => {
            // Delete before dispatching: recovery does not go through the
            // file-picker effect, which is what normally clears the file.
            let _ = std::fs::remove_file(&path);
            let effects = ctx.core.process_event(Event::RestoreDocument {
                document: project.document,
            });
            crate::effects::process_effects(ctx, effects);
        }
        Err(e) => {
            // Corrupt or written by a newer app version: not recoverable.
            eprintln!("autosave: discarding unusable autosave: {e}");
            let _ = std::fs::remove_file(&path);
        }
    }
}

fn schedule_write(ctx: &AppContext) {
    let mut state = ctx.autosave.borrow_mut();
    if let Some(id) = state.timer.take() {
        id.remove();
    }
    let ctx_idle = ctx.clone();
    state.timer = Some(glib::timeout_add_local_once(DEBOUNCE, move || {
        ctx_idle.autosave.borrow_mut().timer = None;
        write_latest(&ctx_idle);
    }));
}

fn write_latest(ctx: &AppContext) {
    let (document, generation) = {
        let mut state = ctx.autosave.borrow_mut();
        (state.latest.take(), state.generation.get())
    };
    let Some(document) = document else {
        return;
    };
    let Some(path) = autosave_path() else {
        return;
    };
    let tmp = path.with_extension("json.tmp");
    ctx.autosave.borrow().in_flight.set(true);

    // Serialization is a few KB and cheap enough for the UI thread; only the
    // file I/O (including the fsync) goes to a worker.
    let envelope = ProjectFile::new(document, ctx.export_settings()).to_json();

    let ctx_write = ctx.clone();
    glib::spawn_future_local(async move {
        let tmp_write = tmp.clone();
        let written = gtk4::gio::spawn_blocking(move || -> Result<(), std::io::Error> {
            use std::io::Write;
            // the app dir may not exist yet on first run.
            if let Some(parent) = tmp_write.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::File::create(&tmp_write)?;
            file.write_all(envelope.as_bytes())?;
            file.sync_all()?;
            Ok(())
        })
        .await
        .unwrap_or_else(|_| Err(std::io::Error::other("autosave worker panicked")));

        // The state may have been cleared or the session may have ended
        // while the write was running: never resurrect the file then.
        let stale = {
            let state = ctx_write.autosave.borrow();
            state.exiting.get() || state.generation.get() != generation
        };
        let outcome = if stale {
            let _ = std::fs::remove_file(&tmp);
            Ok(())
        } else {
            match written {
                Ok(()) => std::fs::rename(&tmp, &path),
                Err(e) => Err(e),
            }
        };
        if let Err(e) = outcome {
            eprintln!("autosave: write failed: {e}");
        }

        let reschedule = {
            let state = ctx_write.autosave.borrow_mut();
            state.in_flight.set(false);
            !stale && state.latest.is_some()
        };
        // If changes arrived while the write ran, write again once they
        // settle — the in-flight write never blocks the next one for long.
        if reschedule {
            schedule_write(&ctx_write);
        }
    });
}

fn delete_files() {
    if let Some(path) = autosave_path() {
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("json.tmp"));
    }
}

/// Constructor helper for `AppContext`.
pub fn new_state() -> Rc<RefCell<AutosaveState>> {
    Rc::new(RefCell::new(AutosaveState::default()))
}
