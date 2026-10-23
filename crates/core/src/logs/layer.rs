//! A `tracing` layer that copies the backend's log events into the database. Events go
//! through a bounded channel, so logging never waits for SQLite; when the channel is
//! full, entries are dropped rather than blocking the caller.

use std::fmt::{self, Write as _};

use tokio::sync::mpsc;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber, warn};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

use crate::clock::now_millis;

use super::domain::{Level, NewLogEntry, Source};
use super::service::LogService;

const CHANNEL_CAPACITY: usize = 1024;
const BATCH_SIZE: usize = 100;
/// Mailbeaver's own crates; other crates only get in with warnings and errors.
const OWN_TARGET_PREFIX: &str = "mailbeaver";
/// Events of the log writer itself would feed back into it.
const WRITER_TARGET: &str = module_path!();

/// Add to the host's subscriber; [`LogReceiver`] goes to `server::start`.
pub struct DatabaseLayer {
    sender: mpsc::Sender<NewLogEntry>,
}

/// The receiving end of [`DatabaseLayer`], drained by [`persist`].
pub struct LogReceiver(mpsc::Receiver<NewLogEntry>);

pub fn database_layer() -> (DatabaseLayer, LogReceiver) {
    let (sender, receiver) = mpsc::channel(CHANNEL_CAPACITY);
    (DatabaseLayer { sender }, LogReceiver(receiver))
}

impl<S: Subscriber> Layer<S> for DatabaseLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let Some(level) = level_of(*metadata.level()) else {
            return;
        };
        if !is_stored(metadata.target(), level) {
            return;
        }
        let mut text = Text::default();
        event.record(&mut text);
        let entry = NewLogEntry::new(
            now_millis(),
            level,
            Source::Backend,
            metadata.target(),
            &text.finish(),
        );
        // Full or closed: the entry is lost, the application goes on.
        let _ = self.sender.try_send(entry);
    }
}

/// Writes what the layer collected until the channel closes.
pub async fn persist(receiver: LogReceiver, logs: LogService) {
    let LogReceiver(mut receiver) = receiver;
    while let Some(first) = receiver.recv().await {
        let mut batch = vec![first];
        while batch.len() < BATCH_SIZE {
            match receiver.try_recv() {
                Ok(next) => batch.push(next),
                Err(_) => break,
            }
        }
        if let Err(err) = logs.write(batch).await {
            warn!(error = %err, "could not store log entries");
        }
    }
}

fn level_of(level: tracing::Level) -> Option<Level> {
    match level {
        tracing::Level::ERROR => Some(Level::Error),
        tracing::Level::WARN => Some(Level::Warn),
        tracing::Level::INFO => Some(Level::Info),
        _ => None,
    }
}

fn is_stored(target: &str, level: Level) -> bool {
    !target.starts_with(WRITER_TARGET)
        && (target.starts_with(OWN_TARGET_PREFIX) || level <= Level::Warn)
}

/// The event's message followed by its fields as `key=value`.
#[derive(Default)]
struct Text {
    message: String,
    fields: String,
}

impl Text {
    fn finish(self) -> String {
        match (self.message.is_empty(), self.fields.is_empty()) {
            (_, true) => self.message,
            (true, false) => self.fields,
            (false, false) => format!("{} {}", self.message, self.fields),
        }
    }

    fn push_field(&mut self, name: &str, value: fmt::Arguments<'_>) {
        if !self.fields.is_empty() {
            self.fields.push(' ');
        }
        let _ = write!(self.fields, "{name}={value}");
    }
}

impl Visit for Text {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_owned();
        } else {
            self.push_field(field.name(), format_args!("{value}"));
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.push_field(field.name(), format_args!("{value:?}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn keeps_own_events_and_foreign_warnings() {
        assert!(is_stored("mailbeaver_core::smtp", Level::Info));
        assert!(is_stored("hyper::server", Level::Warn));
        assert!(!is_stored("hyper::server", Level::Info));
        assert!(!is_stored(WRITER_TARGET, Level::Error));
        assert_eq!(level_of(tracing::Level::DEBUG), None);
    }

    #[test]
    fn turns_events_into_entries() {
        let (layer, LogReceiver(mut receiver)) = database_layer();
        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "mailbeaver_core::smtp", id = 7, from = "a@b.test", "message received");
            tracing::debug!(target: "mailbeaver_core::smtp", "not stored");
        });
        let entry = receiver.try_recv().unwrap();
        assert_eq!(entry.level, Level::Info);
        assert_eq!(entry.target, "mailbeaver_core::smtp");
        assert_eq!(entry.message, "message received id=7 from=a@b.test");
        assert!(receiver.try_recv().is_err());
    }
}
