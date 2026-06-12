use serde_json::Value;

/// Parses JSON frontmatter delimited by `---` lines. Returns `(Value::Null, original_text)`
/// for files without valid frontmatter — this is intentional so the caller can skip them
/// rather than failing the entire build for a single malformed file.
pub fn parse_frontmatter(text: &str) -> (Value, String) {
    let Some(rest) = text.strip_prefix("---\n") else {
        return (Value::Null, text.to_string());
    };

    let end_pos = if let Some(pos) = rest.find("\n---\n") {
        pos
    } else if rest.ends_with("\n---") {
        rest.len() - 4
    } else {
        return (Value::Null, text.to_string());
    };

    let json_str = &rest[..end_pos];
    let body_start = end_pos + 5; // "\n---\n".len()
    let body = rest.get(body_start..).unwrap_or("").to_string();

    match serde_json::from_str(json_str) {
        Ok(value) => (value, body),
        Err(_) => (Value::Null, text.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_single_line_json() {
        let text = "---\n{\"title\": \"Hello\"}\n---\n\n# Content";
        let (meta, body) = parse_frontmatter(text);
        assert_eq!(meta["title"], "Hello");
        assert_eq!(body.trim(), "# Content");
    }

    #[test]
    fn valid_multiline_json() {
        let text = "---\n{\n  \"title\": \"Hello\",\n  \"date\": \"2024-01-01\"\n}\n---\n\nBody here";
        let (meta, body) = parse_frontmatter(text);
        assert_eq!(meta["title"], "Hello");
        assert_eq!(meta["date"], "2024-01-01");
        assert_eq!(body.trim(), "Body here");
    }

    #[test]
    fn no_frontmatter() {
        let text = "# Just content\n\nNo frontmatter here.";
        let (meta, body) = parse_frontmatter(text);
        assert!(meta.is_null());
        assert_eq!(body, text);
    }

    #[test]
    fn invalid_json() {
        let text = "---\nnot json at all\n---\n\nContent";
        let (meta, _body) = parse_frontmatter(text);
        assert!(meta.is_null());
    }

    #[test]
    fn draft_field() {
        let text = "---\n{\"title\": \"Draft\", \"draft\": true}\n---\n\nDraft content";
        let (meta, _body) = parse_frontmatter(text);
        assert_eq!(meta["draft"], true);
    }

    #[test]
    fn empty_body() {
        let text = "---\n{\"title\": \"Empty\"}\n---\n";
        let (meta, body) = parse_frontmatter(text);
        assert_eq!(meta["title"], "Empty");
        assert!(body.trim().is_empty());
    }
}
