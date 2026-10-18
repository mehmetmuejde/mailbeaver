//! The search syntax (ADR-0027): free text plus filters such as `from:shop.test`,
//! `subject:"order 12"`, `is:unread`, `has:attachment`, `project:shop`,
//! `after:2026-10-01` and `before:2026-10-17`. Pure: no I/O.
//!
//! Text ends up in an FTS5 expression that is built here and never contains syntax from
//! the input. An unknown or invalid filter stays free text, so nothing is silently lost.

const MAX_SEARCH_TERMS: usize = 16;
const MILLIS_PER_DAY: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchProject {
    /// `project:none`: mail without a project.
    Unassigned,
    /// A project name, compared without regard to case.
    Named(String),
}

/// A parsed search. Every part must match.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchQuery {
    fts: Option<String>,
    pub is_read: Option<bool>,
    pub has_attachments: Option<bool>,
    pub project: Option<SearchProject>,
    /// Unix milliseconds, inclusive: the start of the `after:` day (UTC).
    pub received_from: Option<i64>,
    /// Unix milliseconds, exclusive: the start of the `before:` day (UTC).
    pub received_before: Option<i64>,
}

impl SearchQuery {
    /// `None` when the input holds nothing to search for.
    pub fn parse(input: &str) -> Option<Self> {
        let mut query = Self::default();
        let mut terms: Vec<String> = Vec::new();
        for token in tokenize(input) {
            if !query.apply_filter(&token) {
                terms.extend(text_term(None, &token.raw_text()));
            }
            if let Some(term) = token
                .key
                .as_deref()
                .and_then(column_of)
                .and_then(|column| text_term(Some(column), &token.value))
            {
                terms.push(term);
            }
        }
        terms.truncate(MAX_SEARCH_TERMS);
        query.fts = (!terms.is_empty()).then(|| terms.join(" "));
        (query != Self::default()).then_some(query)
    }

    /// The FTS5 `MATCH` expression of the text parts, if there are any.
    pub fn as_fts(&self) -> Option<&str> {
        self.fts.as_deref()
    }

    /// Takes a filter that is not a text column; `false` when `token` is none of them.
    fn apply_filter(&mut self, token: &Token) -> bool {
        let Some(key) = token.key.as_deref() else {
            return false;
        };
        let value = token.value.to_lowercase();
        match (key, value.as_str()) {
            ("from" | "to" | "subject" | "body", _) => true,
            ("is", "unread") => {
                self.is_read = Some(false);
                true
            }
            ("is", "read") => {
                self.is_read = Some(true);
                true
            }
            ("has", "attachment" | "attachments") => {
                self.has_attachments = Some(true);
                true
            }
            ("project", "none") => {
                self.project = Some(SearchProject::Unassigned);
                true
            }
            ("project", name) if !name.is_empty() => {
                self.project = Some(SearchProject::Named(token.value.clone()));
                true
            }
            ("after", date) => day_start(date).is_some_and(|start| {
                self.received_from = Some(start);
                true
            }),
            ("before", date) => day_start(date).is_some_and(|start| {
                self.received_before = Some(start);
                true
            }),
            _ => false,
        }
    }
}

/// The FTS5 column a text filter searches.
fn column_of(key: &str) -> Option<&'static str> {
    match key {
        "from" => Some("header_from"),
        "to" => Some("header_to"),
        "subject" => Some("subject"),
        "body" => Some("body_text"),
        _ => None,
    }
}

/// A quoted prefix phrase, limited to `column` when given. `None` when the text has no
/// letter or digit: it would produce no token and break the expression.
fn text_term(column: Option<&str>, text: &str) -> Option<String> {
    if !text.chars().any(char::is_alphanumeric) {
        return None;
    }
    let phrase = format!("\"{}\"*", text.replace('"', "\"\""));
    Some(match column {
        Some(column) => format!("{column} : {phrase}"),
        None => phrase,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct Token {
    /// Lowercase filter name before the colon, e.g. `from`.
    key: Option<String>,
    value: String,
}

impl Token {
    /// The token as typed, for free text.
    fn raw_text(&self) -> String {
        match &self.key {
            Some(key) => format!("{key}:{}", self.value),
            None => self.value.clone(),
        }
    }
}

/// Splits at whitespace outside double quotes; `key:"a b"` and `"a b"` stay one token.
fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    loop {
        while chars.next_if(|c| c.is_whitespace()).is_some() {}
        if chars.peek().is_none() {
            return tokens;
        }
        let mut raw = String::new();
        let mut quoted = false;
        while let Some(c) = chars.next_if(|c| quoted || !c.is_whitespace()) {
            if c == '"' {
                quoted = !quoted;
            }
            raw.push(c);
        }
        tokens.push(split_key(&raw));
    }
}

fn split_key(raw: &str) -> Token {
    let unquote = |value: &str| value.replace('"', "");
    match raw.split_once(':') {
        Some((key, value))
            if !key.is_empty()
                && key.chars().all(|c| c.is_ascii_alphabetic())
                && !value.is_empty() =>
        {
            Token {
                key: Some(key.to_ascii_lowercase()),
                value: unquote(value),
            }
        }
        _ => Token {
            key: None,
            value: unquote(raw),
        },
    }
}

/// The start of a `YYYY-MM-DD` day in UTC as Unix milliseconds.
fn day_start(date: &str) -> Option<i64> {
    let mut parts = date.splitn(3, '-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    let valid = (1970..=9999).contains(&year)
        && (1..=12).contains(&month)
        && (1..=days_in_month(year, month)).contains(&day);
    valid.then(|| days_from_civil(year, month, day) * MILLIS_PER_DAY)
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day_of_year =
        (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> SearchQuery {
        SearchQuery::parse(input).unwrap()
    }

    #[test]
    fn free_text_never_becomes_fts_syntax() {
        assert_eq!(
            parse("\"this\" NEAR(").as_fts(),
            Some(r#""this"* "NEAR("*"#)
        );
        assert_eq!(SearchQuery::parse(" - * "), None);
        assert_eq!(SearchQuery::parse(""), None);
    }

    #[test]
    fn text_filters_search_one_column() {
        let query = parse(r#"from:Shop.Test subject:"order 12" invoice"#);
        assert_eq!(
            query.as_fts(),
            Some(r#"header_from : "Shop.Test"* subject : "order 12"* "invoice"*"#)
        );
        assert_eq!(
            parse("to:dev@team").as_fts(),
            Some(r#"header_to : "dev@team"*"#)
        );
        assert_eq!(
            parse("body:reset").as_fts(),
            Some(r#"body_text : "reset"*"#)
        );
        assert_eq!(parse(r#"subject:a"b"#).as_fts(), Some(r#"subject : "ab"*"#));
    }

    #[test]
    fn reads_flag_and_project_filters() {
        let query = parse("is:unread has:attachment project:Shop");
        assert_eq!(query.is_read, Some(false));
        assert_eq!(query.has_attachments, Some(true));
        assert_eq!(query.project, Some(SearchProject::Named("Shop".into())));
        assert_eq!(query.as_fts(), None);
        assert_eq!(parse("is:read").is_read, Some(true));
        assert_eq!(
            parse("project:none").project,
            Some(SearchProject::Unassigned)
        );
        assert_eq!(
            parse(r#"project:"My Shop""#).project,
            Some(SearchProject::Named("My Shop".into()))
        );
    }

    #[test]
    fn reads_dates_as_utc_days() {
        let query = parse("after:2026-10-01 before:2026-10-17");
        assert_eq!(query.received_from, Some(1_790_812_800_000));
        assert_eq!(query.received_before, Some(1_792_195_200_000));
        assert_eq!(parse("after:1970-01-01").received_from, Some(0));
        assert_eq!(
            parse("after:2024-02-29").received_from,
            Some(1_709_164_800_000)
        );
    }

    #[test]
    fn keeps_unknown_or_invalid_filters_as_text() {
        assert_eq!(
            parse("is:starred after:2026-02-30 http://x").as_fts(),
            Some(r#""is:starred"* "after:2026-02-30"* "http://x"*"#)
        );
        assert_eq!(parse("from:").as_fts(), Some(r#""from:"*"#));
    }

    #[test]
    fn limits_the_number_of_terms() {
        let input = (0..40)
            .map(|n| format!("w{n}"))
            .collect::<Vec<_>>()
            .join(" ");
        let fts = parse(&input).fts.unwrap();
        assert_eq!(fts.matches('*').count(), MAX_SEARCH_TERMS);
    }
}
