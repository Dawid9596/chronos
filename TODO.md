Phase 3: Feature Completeness & Workspace Hygiene
Tracking:Track a has_unsaved_changes: bool flag in AppState (set on document-mutating events, cleared on FileOpenResponse::Opened and FileSaveResponse::Saved). Append "\*" to the window/project title when dirty, and prompt the user in window.connect_close_request and before Event::RequestOpen so they don't accidentally lose work.

.chr.json extension support

Display shortcuts next to respective buttons on editing shortcuts.

Add option to edit description in output pdf, prune out default template from default rows

Animate dragging rows?

Import custom .svg patterns?
