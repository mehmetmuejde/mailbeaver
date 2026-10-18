//! Finds the links of a message: `href` targets in the HTML part, then URLs in the text
//! part. A small scanner, not an HTML parser — good enough for the links mail contains.

const SCHEMES: [&str; 3] = ["http://", "https://", "mailto:"];

/// Unique links in order of appearance; only http, https and mailto.
pub fn extract_links(html: Option<&str>, text: Option<&str>) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    let mut push = |link: String| {
        if SCHEMES
            .iter()
            .any(|scheme| link.to_ascii_lowercase().starts_with(scheme))
            && !links.contains(&link)
        {
            links.push(link);
        }
    };
    if let Some(html) = html {
        for link in href_values(html) {
            push(decode_entities(&link));
        }
    }
    if let Some(text) = text {
        for word in text.split(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\'')) {
            let start = ["http://", "https://"]
                .iter()
                .filter_map(|scheme| word.find(scheme))
                .min();
            if let Some(start) = start {
                let link = word[start..].trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']']);
                push(link.to_owned());
            }
        }
    }
    links
}

/// The quoted values of every `href=` attribute.
fn href_values(html: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let mut values = Vec::new();
    let mut rest = 0;
    while let Some(found) = lower[rest..].find("href=") {
        let value_start = rest + found + "href=".len();
        let Some(quote) = html[value_start..].chars().next() else {
            break;
        };
        rest = value_start;
        if quote != '"' && quote != '\'' {
            continue;
        }
        let inner = value_start + quote.len_utf8();
        if let Some(end) = html[inner..].find(quote) {
            values.push(html[inner..inner + end].trim().to_owned());
            rest = inner + end;
        }
    }
    values
}

/// `&amp;` is common in URLs of HTML mail; other entities do not occur in links.
fn decode_entities(link: &str) -> String {
    link.replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_html_and_text_links_once_and_in_order() {
        let html = r#"<a HREF="https://shop.example/reset?a=1&amp;b=2">Reset</a>
            <a href='mailto:help@shop.example'>Help</a>
            <a href="javascript:alert(1)">Bad</a> <a href="/relative">Rel</a>"#;
        let text = "Reset: https://shop.example/reset?a=1&b=2. Docs (https://docs.example/start).";
        assert_eq!(
            extract_links(Some(html), Some(text)),
            [
                "https://shop.example/reset?a=1&b=2",
                "mailto:help@shop.example",
                "https://docs.example/start",
            ]
        );
        assert!(extract_links(None, None).is_empty());
    }
}
