//! Change notifications. Every successful write announces what changed, so the UI
//! (through Server-Sent Events) and the tray can refresh.

use serde::Serialize;
use tokio::sync::broadcast;

use crate::messages::{Folder, MessageId};

const CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Change {
    Received {
        id: MessageId,
    },
    Read {
        id: MessageId,
    },
    Unread {
        id: MessageId,
    },
    Moved {
        id: MessageId,
        folder: Folder,
    },
    Deleted {
        id: MessageId,
    },
    TrashEmptied {
        count: usize,
    },
    MarkedRead {
        count: usize,
    },
    Cleared,
    /// All data was removed; everything must be reloaded.
    Reset,
    ProjectsChanged,
    SettingsChanged,
    /// Capture was paused or resumed.
    CaptureChanged {
        paused: bool,
    },
}

#[derive(Clone)]
pub struct ChangeFeed {
    sender: broadcast::Sender<Change>,
}

impl Default for ChangeFeed {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeFeed {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Change> {
        self.sender.subscribe()
    }

    pub fn announce(&self, change: Change) {
        // No subscribers just means no UI is open; the change is persisted either way.
        let _ = self.sender.send(change);
    }
}
