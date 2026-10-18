//! Pausing capture: while paused, the SMTP server refuses mail with a temporary error,
//! so well-behaved senders retry later. Kept in memory only — after a restart
//! Mailbeaver always captures again, nothing is lost by forgetting to resume.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::changes::{Change, ChangeFeed};

#[derive(Clone)]
pub struct Capture {
    paused: Arc<AtomicBool>,
    changes: ChangeFeed,
}

impl Capture {
    pub fn new(changes: ChangeFeed) -> Self {
        Self {
            paused: Arc::default(),
            changes,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    /// Announces the new state when it changed.
    pub fn set_paused(&self, paused: bool) {
        if self.paused.swap(paused, Ordering::Relaxed) != paused {
            self.changes.announce(Change::CaptureChanged { paused });
        }
    }
}

impl fmt::Debug for Capture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Capture")
            .field("paused", &self.is_paused())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announces_only_real_changes() {
        let changes = ChangeFeed::new();
        let mut feed = changes.subscribe();
        let capture = Capture::new(changes);
        assert!(!capture.is_paused());

        capture.set_paused(true);
        capture.set_paused(true);
        assert!(capture.is_paused());
        assert_eq!(
            feed.try_recv().unwrap(),
            Change::CaptureChanged { paused: true }
        );
        assert!(feed.try_recv().is_err());
    }
}
