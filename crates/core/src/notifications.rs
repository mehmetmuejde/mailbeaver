//! Which new mail raises a system notification and what it says (ADR-0026). Pure: the
//! desktop host shows the notices.

use std::collections::BTreeSet;

use crate::messages::{Folder, MessageId, MessageSummary};
use crate::projects::ProjectId;
use crate::settings::Notifications;

/// Senders and subjects listed in a notice about several messages.
const LISTED_MESSAGES: usize = 3;
const NO_SUBJECT: &str = "(no subject)";
const NO_SENDER: &str = "Unknown sender";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub body: String,
    /// The message a click opens; `None` opens the inbox.
    pub message: Option<MessageId>,
}

/// Whether `message` should notify. `notifying_projects` are the projects with
/// notifications switched on; they only matter for [`Notifications::Projects`].
pub fn wants_notice(
    setting: Notifications,
    message: &MessageSummary,
    notifying_projects: &BTreeSet<ProjectId>,
) -> bool {
    if message.folder != Folder::Inbox {
        return false;
    }
    match setting {
        Notifications::Off => false,
        Notifications::All => true,
        Notifications::Projects => message
            .project
            .as_ref()
            .is_some_and(|project| notifying_projects.contains(&project.id)),
    }
}

/// One notice for mail that arrived together: the message itself when it is alone, a
/// summary of the first few otherwise.
pub fn notice_for(messages: &[MessageSummary]) -> Option<Notice> {
    match messages {
        [] => None,
        [message] => {
            let sender = or_default(&message.from, NO_SENDER);
            let title = match &message.project {
                Some(project) => format!("{sender} · {}", project.name),
                None => sender.to_owned(),
            };
            let subject = or_default(&message.subject, NO_SUBJECT);
            let body = if message.preview.is_empty() {
                subject.to_owned()
            } else {
                format!("{subject}\n{}", message.preview)
            };
            Some(Notice {
                title,
                body,
                message: Some(message.id),
            })
        }
        many => {
            let mut lines: Vec<String> = many
                .iter()
                .take(LISTED_MESSAGES)
                .map(|message| {
                    format!(
                        "{} – {}",
                        or_default(&message.from, NO_SENDER),
                        or_default(&message.subject, NO_SUBJECT)
                    )
                })
                .collect();
            if many.len() > LISTED_MESSAGES {
                lines.push(format!("and {} more", many.len() - LISTED_MESSAGES));
            }
            Some(Notice {
                title: format!("{} new messages", many.len()),
                body: lines.join("\n"),
                message: None,
            })
        }
    }
}

fn or_default<'a>(value: &'a str, default: &'a str) -> &'a str {
    if value.trim().is_empty() {
        default
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::ProjectRef;

    fn message(id: i64, project: Option<i64>) -> MessageSummary {
        MessageSummary {
            id: MessageId(id),
            received_at: 0,
            from: format!("sender{id}@shop.test"),
            to: vec![],
            subject: format!("Order {id}"),
            size: 1,
            has_attachments: false,
            is_read: false,
            folder: Folder::Inbox,
            project: project.map(|id| ProjectRef {
                id: ProjectId(id),
                name: format!("Project {id}"),
            }),
            preview: String::new(),
        }
    }

    #[test]
    fn follows_the_setting() {
        let notifying = BTreeSet::from([ProjectId(1)]);
        let in_project = message(1, Some(1));
        let in_other_project = message(2, Some(2));
        let without_project = message(3, None);

        assert!(!wants_notice(Notifications::Off, &in_project, &notifying));
        assert!(wants_notice(
            Notifications::All,
            &without_project,
            &notifying
        ));
        assert!(wants_notice(
            Notifications::Projects,
            &in_project,
            &notifying
        ));
        assert!(!wants_notice(
            Notifications::Projects,
            &in_other_project,
            &notifying
        ));
        assert!(!wants_notice(
            Notifications::Projects,
            &without_project,
            &notifying
        ));

        let archived = MessageSummary {
            folder: Folder::Archive,
            ..message(4, None)
        };
        assert!(!wants_notice(Notifications::All, &archived, &notifying));
    }

    #[test]
    fn describes_a_single_message() {
        let notice = notice_for(&[MessageSummary {
            preview: "Thanks for your order".into(),
            ..message(7, Some(1))
        }])
        .unwrap();
        assert_eq!(notice.title, "sender7@shop.test · Project 1");
        assert_eq!(notice.body, "Order 7\nThanks for your order");
        assert_eq!(notice.message, Some(MessageId(7)));

        let blank = MessageSummary {
            from: String::new(),
            subject: " ".into(),
            ..message(8, None)
        };
        let notice = notice_for(&[blank]).unwrap();
        assert_eq!(
            (notice.title.as_str(), notice.body.as_str()),
            (NO_SENDER, NO_SUBJECT)
        );
    }

    #[test]
    fn summarizes_a_burst() {
        assert_eq!(notice_for(&[]), None);
        let burst: Vec<_> = (1..=5).map(|id| message(id, None)).collect();
        let notice = notice_for(&burst).unwrap();
        assert_eq!(notice.title, "5 new messages");
        assert_eq!(
            notice.body,
            "sender1@shop.test – Order 1\nsender2@shop.test – Order 2\n\
             sender3@shop.test – Order 3\nand 2 more"
        );
        assert_eq!(notice.message, None);
    }
}
