//! Projects and the rules that assign incoming mail to them. Pure: no I/O.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const MAX_NAME_CHARS: usize = 64;
const MAX_AUTH_USER_CHARS: usize = 64;
const MAX_ADDRESS_LEN: usize = 254;
const MAX_DOMAIN_LEN: usize = 253;
pub const MAX_RULES_PER_PROJECT: usize = 50;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(transparent)]
pub struct ProjectId(pub i64);

/// Trimmed, 1–64 characters, no control characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
#[schema(value_type = String)]
pub struct ProjectName(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("a project name needs 1 to 64 characters without control characters")]
pub struct InvalidProjectName;

impl ProjectName {
    pub fn parse(value: &str) -> Result<Self, InvalidProjectName> {
        let value = value.trim();
        let length = value.chars().count();
        let valid = (1..=MAX_NAME_CHARS).contains(&length) && !value.chars().any(char::is_control);
        valid
            .then(|| Self(value.to_owned()))
            .ok_or(InvalidProjectName)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ProjectName {
    type Error = InvalidProjectName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<ProjectName> for String {
    fn from(name: ProjectName) -> Self {
        name.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// One exact sender address, e.g. `noreply@shop.example`.
    SenderAddress,
    /// Every sender of a domain and its subdomains, e.g. `shop.example`.
    SenderDomain,
    /// The username the SMTP client authenticated with.
    AuthUser,
}

impl RuleKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SenderAddress => "sender_address",
            Self::SenderDomain => "sender_domain",
            Self::AuthUser => "auth_user",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "sender_address" => Some(Self::SenderAddress),
            "sender_domain" => Some(Self::SenderDomain),
            "auth_user" => Some(Self::AuthUser),
            _ => None,
        }
    }
}

/// A validated rule; the value is normalized to lowercase so matching ignores case.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, ToSchema)]
pub struct Rule {
    pub kind: RuleKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidRule {
    #[error("{0:?} is not an email address")]
    Address(String),
    #[error("{0:?} is not a domain")]
    Domain(String),
    #[error("{0:?} is not a valid SMTP username")]
    AuthUser(String),
}

impl Rule {
    pub fn parse(kind: RuleKind, raw: &str) -> Result<Self, InvalidRule> {
        let trimmed = raw.trim();
        let value = match kind {
            RuleKind::SenderAddress => {
                let value = trimmed.to_lowercase();
                is_address(&value)
                    .then_some(value)
                    .ok_or_else(|| InvalidRule::Address(raw.to_owned()))?
            }
            RuleKind::SenderDomain => {
                let value = trimmed.trim_start_matches('@').to_lowercase();
                is_domain(&value)
                    .then_some(value)
                    .ok_or_else(|| InvalidRule::Domain(raw.to_owned()))?
            }
            RuleKind::AuthUser => {
                let length = trimmed.chars().count();
                let valid = (1..=MAX_AUTH_USER_CHARS).contains(&length)
                    && !trimmed.chars().any(char::is_control);
                valid
                    .then(|| trimmed.to_lowercase())
                    .ok_or_else(|| InvalidRule::AuthUser(raw.to_owned()))?
            }
        };
        Ok(Self { kind, value })
    }
}

fn is_address(value: &str) -> bool {
    let Some((local, domain)) = value.rsplit_once('@') else {
        return false;
    };
    value.len() <= MAX_ADDRESS_LEN
        && !local.is_empty()
        && !local.contains('@')
        && !value.chars().any(char::is_whitespace)
        && is_domain(domain)
}

fn is_domain(value: &str) -> bool {
    value.len() <= MAX_DOMAIN_LEN
        && value.contains('.')
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..")
        && value
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '.'))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Project {
    pub id: ProjectId,
    pub name: ProjectName,
    pub rules: Vec<Rule>,
    /// Raises notifications when they are limited to projects.
    pub notify: bool,
}

/// A value seen in captured mail that a new rule could use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct RuleSuggestion {
    pub value: String,
    /// Messages outside Trash with this value.
    pub count: i64,
}

/// All rules of all projects, ready to assign messages.
#[derive(Debug, Clone, Default)]
pub struct ProjectRules {
    rules: Vec<(ProjectId, Rule)>,
}

impl ProjectRules {
    pub fn new(rules: Vec<(ProjectId, Rule)>) -> Self {
        Self { rules }
    }

    /// The project a message belongs to. The SMTP user wins over the exact address,
    /// the exact address over the domain, and a subdomain matches its domain rule.
    pub fn resolve(&self, sender_address: &str, auth_user: Option<&str>) -> Option<ProjectId> {
        let sender = sender_address.to_lowercase();
        let auth_user = auth_user.map(|user| user.trim().to_lowercase());
        let sender_domain = sender.rsplit_once('@').map(|(_, domain)| domain);

        let find = |kind: RuleKind, matches: &dyn Fn(&str) -> bool| {
            self.rules
                .iter()
                .find(|(_, rule)| rule.kind == kind && matches(&rule.value))
                .map(|(id, _)| *id)
        };
        auth_user
            .as_deref()
            .and_then(|user| find(RuleKind::AuthUser, &|value| value == user))
            .or_else(|| find(RuleKind::SenderAddress, &|value| value == sender))
            .or_else(|| {
                let domain = sender_domain?;
                // The most specific domain rule wins: `mail.shop.example` before `shop.example`.
                self.rules
                    .iter()
                    .filter(|(_, rule)| rule.kind == RuleKind::SenderDomain)
                    .filter(|(_, rule)| {
                        domain == rule.value || domain.ends_with(&format!(".{}", rule.value))
                    })
                    .max_by_key(|(_, rule)| rule.value.len())
                    .map(|(id, _)| *id)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(kind: RuleKind, value: &str) -> Rule {
        Rule::parse(kind, value).unwrap()
    }

    #[test]
    fn normalizes_and_validates_rules() {
        assert_eq!(
            rule(RuleKind::SenderAddress, " NoReply@Shop.Example ").value,
            "noreply@shop.example"
        );
        assert_eq!(
            rule(RuleKind::SenderDomain, "@Shop.Example").value,
            "shop.example"
        );
        assert_eq!(rule(RuleKind::AuthUser, " Shop ").value, "shop");
        assert!(Rule::parse(RuleKind::SenderAddress, "no-at-sign").is_err());
        assert!(Rule::parse(RuleKind::SenderAddress, "a b@shop.example").is_err());
        assert!(Rule::parse(RuleKind::SenderDomain, "localhost").is_err());
        assert!(Rule::parse(RuleKind::SenderDomain, "shop..example").is_err());
        assert!(Rule::parse(RuleKind::AuthUser, "  ").is_err());
    }

    #[test]
    fn validates_project_names() {
        assert_eq!(ProjectName::parse(" Shop ").unwrap().as_str(), "Shop");
        assert!(ProjectName::parse("").is_err());
        assert!(ProjectName::parse("a\nb").is_err());
        assert!(ProjectName::parse(&"x".repeat(65)).is_err());
    }

    #[test]
    fn resolves_by_priority() {
        let shop = ProjectId(1);
        let blog = ProjectId(2);
        let billing = ProjectId(3);
        let rules = ProjectRules::new(vec![
            (shop, rule(RuleKind::SenderDomain, "shop.example")),
            (
                billing,
                rule(RuleKind::SenderDomain, "billing.shop.example"),
            ),
            (blog, rule(RuleKind::SenderAddress, "news@shop.example")),
            (blog, rule(RuleKind::AuthUser, "blog")),
        ]);

        assert_eq!(rules.resolve("orders@shop.example", None), Some(shop));
        assert_eq!(rules.resolve("orders@mail.shop.example", None), Some(shop));
        assert_eq!(
            rules.resolve("invoice@billing.shop.example", None),
            Some(billing)
        );
        assert_eq!(rules.resolve("News@Shop.Example", None), Some(blog));
        assert_eq!(
            rules.resolve("orders@shop.example", Some("Blog")),
            Some(blog)
        );
        assert_eq!(rules.resolve("orders@notshop.example", None), None);
        assert_eq!(rules.resolve("", Some("unknown")), None);
    }
}
