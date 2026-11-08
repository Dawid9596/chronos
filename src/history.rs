//! Bounded undo/redo history
//!
//! Based on snapshots rather than action reversal.
//!
//! # Coalescing
//!
//! Callers tag consecutive edits which really should have one undo with a group key `G`;
//! Undo entry shared until the group is [`seal`](History::seal)ed,
//! another kind of edit is recorded, or an undo/redo happens.

use std::collections::VecDeque;

pub const DEFAULT_HISTORY_LIMIT: usize = 100;

#[derive(Clone, Debug)]
pub struct History<T, G = ()> {
    undo: VecDeque<T>,
    redo: Vec<T>,
    limit: usize,
    open_group: Option<G>,
}

impl<T, G> Default for History<T, G> {
    fn default() -> Self {
        Self::new(DEFAULT_HISTORY_LIMIT)
    }
}

impl<T, G> History<T, G> {
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            undo: VecDeque::new(),
            redo: Vec::new(),
            limit,
            open_group: None,
        }
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.open_group = None;
    }

    /// End the current coalescing group.
    pub fn seal(&mut self) {
        self.open_group = None;
    }

    pub fn undo(&mut self, current: &mut T) -> bool {
        self.open_group = None;
        match self.undo.pop_back() {
            Some(previous) => {
                self.redo.push(std::mem::replace(current, previous));
                true
            }
            None => false,
        }
    }

    pub fn redo(&mut self, current: &mut T) -> bool {
        self.open_group = None;
        match self.redo.pop() {
            Some(next) => {
                let previous = std::mem::replace(current, next);
                self.push_undo(previous);
                true
            }
            None => false,
        }
    }

    fn push_undo(&mut self, snapshot: T) {
        self.undo.push_back(snapshot);
        while self.undo.len() > self.limit {
            self.undo.pop_front();
        }
    }
}

impl<T: PartialEq, G: PartialEq> History<T, G> {
    /// Record an edit.
    ///
    /// * No-op edits (`before == after`) are ignored.
    /// * An edit whose `group` equals the currently open group is folded into
    ///   the existing undo step.
    /// * Otherwise a new undo step is pushed and the redo stack is cleared.
    pub fn record(&mut self, before: T, after: &T, group: Option<G>) {
        if before == *after {
            return;
        }
        if group.is_some() && group == self.open_group {
            return;
        }
        self.push_undo(before);
        self.redo.clear();
        self.open_group = group;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(
        h: &mut History<i32, &'static str>,
        state: &mut i32,
        new: i32,
        g: Option<&'static str>,
    ) {
        let before = *state;
        *state = new;
        h.record(before, state, g);
    }

    #[test]
    fn undo_and_redo_walk_the_timeline() {
        let mut h = History::<i32, &str>::default();
        let mut s = 0;
        edit(&mut h, &mut s, 1, None);
        edit(&mut h, &mut s, 2, None);

        assert!(h.undo(&mut s));
        assert_eq!(s, 1);
        assert!(h.undo(&mut s));
        assert_eq!(s, 0);
        assert!(!h.undo(&mut s));
        assert_eq!(s, 0);

        assert!(h.redo(&mut s));
        assert_eq!(s, 1);
        assert!(h.redo(&mut s));
        assert_eq!(s, 2);
        assert!(!h.redo(&mut s));
        assert_eq!(s, 2);
    }

    #[test]
    fn new_edit_clears_redo() {
        let mut h = History::<i32, &str>::default();
        let mut s = 0;
        edit(&mut h, &mut s, 1, None);
        h.undo(&mut s);
        assert!(h.can_redo());
        edit(&mut h, &mut s, 5, None);
        assert!(!h.can_redo());
    }

    #[test]
    fn no_op_edit_is_ignored_and_keeps_redo() {
        let mut h = History::<i32, &str>::default();
        let mut s = 0;
        edit(&mut h, &mut s, 1, None);
        h.undo(&mut s);
        edit(&mut h, &mut s, 0, None); // unchanged
        assert!(h.can_redo());
        assert!(!h.can_undo());
    }

    #[test]
    fn same_group_coalesces_into_one_step() {
        let mut h = History::<i32, &str>::default();
        let mut s = 10;
        for v in [11, 12, 13, 14] {
            edit(&mut h, &mut s, v, Some("drag"));
        }
        assert!(h.undo(&mut s));
        assert_eq!(s, 10);
        assert!(!h.can_undo());
        assert!(h.redo(&mut s));
        assert_eq!(s, 14);
    }

    #[test]
    fn seal_and_different_groups_split_steps() {
        let mut h = History::<i32, &str>::default();
        let mut s = 0;
        edit(&mut h, &mut s, 1, Some("a"));
        edit(&mut h, &mut s, 2, Some("a"));
        h.seal();
        edit(&mut h, &mut s, 3, Some("a"));
        edit(&mut h, &mut s, 4, Some("b"));
        edit(&mut h, &mut s, 5, None);

        let mut seen = vec![];
        while h.undo(&mut s) {
            seen.push(s);
        }
        assert_eq!(seen, vec![4, 3, 2, 0]); // 1 and 2 shared one step
    }

    #[test]
    fn undo_closes_the_open_group() {
        let mut h = History::<i32, &str>::default();
        let mut s = 0;
        edit(&mut h, &mut s, 1, Some("a"));
        h.undo(&mut s);
        edit(&mut h, &mut s, 2, Some("a"));
        // Must be a fresh step, not folded into the (already undone) one.
        assert!(h.undo(&mut s));
        assert_eq!(s, 0);
    }

    #[test]
    fn limit_drops_oldest_steps() {
        let mut h = History::<i32, &str>::new(3);
        let mut s = 0;
        for v in 1..=5 {
            edit(&mut h, &mut s, v, None);
        }
        let mut seen = vec![];
        while h.undo(&mut s) {
            seen.push(s);
        }
        assert_eq!(seen, vec![4, 3, 2]); // 0 and 1 fell off the front
    }

    #[test]
    fn clear_resets_everything() {
        let mut h = History::<i32, &str>::default();
        let mut s = 0;
        edit(&mut h, &mut s, 1, None);
        h.undo(&mut s);
        h.clear();
        assert!(!h.can_undo() && !h.can_redo());
    }
}
