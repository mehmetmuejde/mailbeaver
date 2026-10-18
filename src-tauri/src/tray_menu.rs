//! What the tray menu shows, computed without Tauri so it can be tested: the newest
//! inbox messages, grouped as the settings say, with short labels and ages.

use mailbeaver_core::messages::{MessageId, MessageSummary};
use mailbeaver_core::settings::Grouping;

const MAX_LABEL_CHARS: usize = 48;
const UNKNOWN_SENDER: &str = "(unknown sender)";
const NO_SUBJECT: &str = "(no subject)";
const NO_PROJECT: &str = "No project";

const MILLIS_PER_MINUTE: i64 = 60 * 1000;
const MILLIS_PER_HOUR: i64 = 60 * MILLIS_PER_MINUTE;
const MILLIS_PER_DAY: i64 = 24 * MILLIS_PER_HOUR;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayMessage {
    pub id: MessageId,
    pub label: String,
    pub unread: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayNode {
    Message(TrayMessage),
    Group {
        label: String,
        messages: Vec<TrayMessage>,
    },
}

/// The actions below the inbox, with their labels and whether they can run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayActions {
    pub mark_all_read: (String, bool),
    pub empty_trash: (String, bool),
    pub capture: String,
}

pub fn actions(unread: i64, in_trash: i64, paused: bool) -> TrayActions {
    TrayActions {
        mark_all_read: ("Mark All as Read".to_owned(), unread > 0),
        empty_trash: (
            if in_trash > 0 {
                format!("Empty Trash ({in_trash})")
            } else {
                "Empty Trash".to_owned()
            },
            in_trash > 0,
        ),
        capture: if paused {
            "Resume Capture".to_owned()
        } else {
            "Pause Capture".to_owned()
        },
    }
}

/// Groups keep the order of their newest message.
pub fn layout(messages: &[MessageSummary], grouping: Grouping, now: i64) -> Vec<TrayNode> {
    let key = |message: &MessageSummary| match grouping {
        Grouping::All => None,
        Grouping::Sender => Some(sender(message).to_owned()),
        Grouping::Project => Some(
            message
                .project
                .as_ref()
                .map_or(NO_PROJECT.to_owned(), |project| project.name.clone()),
        ),
    };
    let mut nodes: Vec<TrayNode> = Vec::new();
    for message in messages {
        let entry = TrayMessage {
            id: message.id,
            label: message_label(message, grouping, now),
            unread: !message.is_read,
        };
        match key(message) {
            None => nodes.push(TrayNode::Message(entry)),
            Some(group) => {
                let existing = nodes.iter_mut().find_map(|node| match node {
                    TrayNode::Group { label, messages } if *label == group => Some(messages),
                    _ => None,
                });
                match existing {
                    Some(messages) => messages.push(entry),
                    None => nodes.push(TrayNode::Group {
                        label: group,
                        messages: vec![entry],
                    }),
                }
            }
        }
    }
    nodes
}

/// `Subject – Sender · 5m`; the sender is left out when the group already names it.
fn message_label(message: &MessageSummary, grouping: Grouping, now: i64) -> String {
    let subject = if message.subject.is_empty() {
        NO_SUBJECT
    } else {
        &message.subject
    };
    let text = match grouping {
        Grouping::Sender => subject.to_owned(),
        Grouping::All | Grouping::Project => format!("{subject} – {}", sender(message)),
    };
    format!(
        "{} · {}",
        truncate(&text, MAX_LABEL_CHARS),
        age(now - message.received_at)
    )
}

fn sender(message: &MessageSummary) -> &str {
    if message.from.is_empty() {
        UNKNOWN_SENDER
    } else {
        &message.from
    }
}

/// Short relative age: `now`, `5m`, `3h`, `2d`.
pub fn age(elapsed_ms: i64) -> String {
    if elapsed_ms < MILLIS_PER_MINUTE {
        "now".to_owned()
    } else if elapsed_ms < MILLIS_PER_HOUR {
        format!("{}m", elapsed_ms / MILLIS_PER_MINUTE)
    } else if elapsed_ms < MILLIS_PER_DAY {
        format!("{}h", elapsed_ms / MILLIS_PER_HOUR)
    } else {
        format!("{}d", elapsed_ms / MILLIS_PER_DAY)
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let kept: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{kept}…")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mailbeaver_core::messages::Folder;

    const NOW: i64 = 10 * MILLIS_PER_DAY;

    fn message(id: i64, from: &str, project: Option<&str>, is_read: bool) -> MessageSummary {
        MessageSummary {
            id: MessageId(id),
            received_at: NOW - 5 * MILLIS_PER_MINUTE,
            from: from.into(),
            to: vec![],
            subject: format!("Mail {id}"),
            size: 1,
            has_attachments: false,
            is_read,
            folder: Folder::Inbox,
            project: project.map(|name| mailbeaver_core::messages::ProjectRef {
                id: mailbeaver_core::projects::ProjectId(1),
                name: name.into(),
            }),
            preview: String::new(),
        }
    }

    #[test]
    fn lists_messages_flat_with_sender_and_age() {
        let nodes = layout(&[message(1, "shop@x", None, false)], Grouping::All, NOW);
        assert_eq!(
            nodes,
            [TrayNode::Message(TrayMessage {
                id: MessageId(1),
                label: "Mail 1 – shop@x · 5m".into(),
                unread: true,
            })]
        );
    }

    #[test]
    fn groups_by_sender_and_project_in_order_of_the_newest() {
        let messages = [
            message(3, "shop@x", Some("Shop"), false),
            message(2, "", None, true),
            message(1, "shop@x", Some("Shop"), true),
        ];
        let by_sender = layout(&messages, Grouping::Sender, NOW);
        let labels: Vec<_> = by_sender
            .iter()
            .map(|node| match node {
                TrayNode::Group { label, messages } => (label.as_str(), messages.len()),
                TrayNode::Message(_) => ("", 0),
            })
            .collect();
        assert_eq!(labels, [("shop@x", 2), (UNKNOWN_SENDER, 1)]);
        let TrayNode::Group {
            messages: grouped, ..
        } = &by_sender[0]
        else {
            panic!("expected a group");
        };
        assert_eq!(grouped[0].label, "Mail 3 · 5m");

        let by_project = layout(&messages, Grouping::Project, NOW);
        assert!(matches!(&by_project[1], TrayNode::Group { label, .. } if label == NO_PROJECT));
    }

    #[test]
    fn formats_ages_and_truncates() {
        assert_eq!(age(10_000), "now");
        assert_eq!(age(3 * MILLIS_PER_HOUR), "3h");
        assert_eq!(age(2 * MILLIS_PER_DAY + 1), "2d");
        assert_eq!(truncate("abcdef", 4), "abc…");
    }

    #[test]
    fn labels_the_actions() {
        let idle = actions(0, 0, false);
        assert_eq!(idle.mark_all_read, ("Mark All as Read".to_owned(), false));
        assert_eq!(idle.empty_trash, ("Empty Trash".to_owned(), false));
        assert_eq!(idle.capture, "Pause Capture");

        let busy = actions(3, 2, true);
        assert!(busy.mark_all_read.1);
        assert_eq!(busy.empty_trash, ("Empty Trash (2)".to_owned(), true));
        assert_eq!(busy.capture, "Resume Capture");
    }
}
