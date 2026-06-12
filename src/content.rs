use std::path::Path;
use std::sync::OnceLock;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html::push_html};
use serde::Serialize;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use walkdir::WalkDir;

use crate::frontmatter::parse_frontmatter;

fn str_field(meta: &serde_json::Value, key: &str) -> String {
    meta.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub title: String,
    pub date: String,
    pub description: String,
    pub template: String,
    pub content: String,
    pub url: String,
    pub section: Option<String>,
    pub slug: String,
    pub meta: serde_json::Value,
}

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

pub fn render_markdown(text: &str, theme_name: &str) -> String {
    let ss = SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);
    let ts = THEME_SET.get_or_init(ThemeSet::load_defaults);
    let theme = ts
        .themes
        .get(theme_name)
        .unwrap_or_else(|| {
            eprintln!(
                "Warning: theme '{}' not found, falling back to 'base16-ocean.dark'",
                theme_name
            );
            &ts.themes["base16-ocean.dark"]
        });

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(text, options);
    let mut events: Vec<Event> = Vec::new();
    let mut code_block: Option<(String, String)> = None;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(ref lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                code_block = Some((lang, String::new()));
            }
            Event::Text(ref t) if code_block.is_some() => {
                code_block.as_mut().unwrap().1.push_str(t);
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((lang, code)) = code_block.take() {
                    let syntax = ss
                        .find_syntax_by_token(&lang)
                        .unwrap_or_else(|| ss.find_syntax_plain_text());
                    let highlighted =
                        highlighted_html_for_string(&code, ss, syntax, theme).unwrap_or_else(
                            |_| {
                                format!(
                                    "<pre><code>{}</code></pre>",
                                    code.replace('&', "&amp;")
                                        .replace('<', "&lt;")
                                        .replace('>', "&gt;")
                                )
                            },
                        );
                    events.push(Event::Html(highlighted.into()));
                }
            }
            other => events.push(other),
        }
    }

    let mut output = String::new();
    push_html(&mut output, events.into_iter());
    output
}

pub fn load_content(
    content_dir: &Path,
    code_theme: &str,
    include_drafts: bool,
    default_template: &str,
) -> Result<Vec<Page>, Box<dyn std::error::Error>> {
    let mut pages = Vec::new();

    for entry in WalkDir::new(content_dir).into_iter() {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Warning: skipping unreadable entry: {}", err);
                continue;
            }
        };

        if !entry.path().extension().is_some_and(|ext| ext == "md") {
            continue;
        }

        let text = std::fs::read_to_string(entry.path())?;
        let (meta, body) = parse_frontmatter(&text);

        if meta.is_null() {
            continue;
        }

        if !include_drafts && meta.get("draft").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }

        let rel_path = entry.path().strip_prefix(content_dir).unwrap();
        let components: Vec<_> = rel_path.components().collect();

        let section = if components.len() > 1 {
            Some(components[0].as_os_str().to_string_lossy().to_string())
        } else {
            None
        };

        let slug = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let url = if slug == "index" {
            match &section {
                Some(s) => format!("/{s}/"),
                None => "/".to_string(),
            }
        } else {
            match &section {
                Some(s) => format!("/{s}/{slug}/"),
                None => format!("/{slug}/"),
            }
        };

        let page = Page {
            title: str_field(&meta, "title"),
            date: str_field(&meta, "date"),
            description: str_field(&meta, "description"),
            template: meta.get("template")
                .and_then(|v| v.as_str())
                .map(|t| format!("{t}.html"))
                .unwrap_or_else(|| default_template.to_string()),
            content: render_markdown(&body, code_theme),
            url,
            section,
            slug,
            meta: meta.clone(),
        };

        pages.push(page);
    }

    pages.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading() {
        let html = render_markdown("# Hello World", "base16-ocean.dark");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello World"));
    }

    #[test]
    fn paragraph() {
        let html = render_markdown("Some text here.", "base16-ocean.dark");
        assert!(html.contains("<p>"));
    }

    #[test]
    fn fenced_code_block_with_language() {
        let md = "```python\nprint(\"hello\")\n```";
        let html = render_markdown(md, "base16-ocean.dark");
        assert!(html.contains("print"));
        assert!(html.contains("<pre"));
    }

    #[test]
    fn fenced_code_block_without_language() {
        let md = "```\nsome code\n```";
        let html = render_markdown(md, "base16-ocean.dark");
        assert!(html.contains("some code"));
        assert!(html.contains("<pre"));
    }

    #[test]
    fn table() {
        let md = "| Name | Value |\n|------|-------|\n| a    | 1     |";
        let html = render_markdown(md, "base16-ocean.dark");
        assert!(html.contains("<table>"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn link() {
        let html = render_markdown("[click](https://example.com)", "base16-ocean.dark");
        assert!(html.contains("href=\"https://example.com\""));
    }

    #[test]
    fn unordered_list() {
        let md = "* item one\n* item two";
        let html = render_markdown(md, "base16-ocean.dark");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
    }

    #[test]
    fn inline_code() {
        let html = render_markdown("Use `cargo build` to compile.", "base16-ocean.dark");
        assert!(html.contains("<code>"));
        assert!(html.contains("cargo build"));
    }

    #[test]
    fn invalid_theme_falls_back() {
        let html = render_markdown("```rust\nlet x = 1;\n```", "nonexistent-theme");
        assert!(html.contains("<pre"));
    }

    use std::io::Write as _;
    use tempfile::TempDir;

    fn write_file(dir: &std::path::Path, name: &str, content: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn loads_markdown_files() {
        let dir = TempDir::new().unwrap();
        write_file(
            dir.path(),
            "posts/hello.md",
            "---\n{\"title\": \"Hello\", \"date\": \"2024-01-01\"}\n---\n\n# Hello World",
        );
        let pages = load_content(dir.path(), "base16-ocean.dark", false, "page.html").unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].title, "Hello");
        assert!(pages[0].content.contains("<h1>"));
    }

    #[test]
    fn skips_drafts() {
        let dir = TempDir::new().unwrap();
        write_file(
            dir.path(),
            "posts/draft.md",
            "---\n{\"title\": \"Draft\", \"draft\": true}\n---\n\nDraft",
        );
        write_file(
            dir.path(),
            "posts/published.md",
            "---\n{\"title\": \"Published\"}\n---\n\nPublished",
        );
        let pages = load_content(dir.path(), "base16-ocean.dark", false, "page.html").unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].title, "Published");
    }

    #[test]
    fn includes_drafts_when_requested() {
        let dir = TempDir::new().unwrap();
        write_file(
            dir.path(),
            "posts/draft.md",
            "---\n{\"title\": \"Draft\", \"draft\": true}\n---\n\nDraft",
        );
        write_file(
            dir.path(),
            "posts/published.md",
            "---\n{\"title\": \"Published\"}\n---\n\nPublished",
        );
        let pages = load_content(dir.path(), "base16-ocean.dark", true, "page.html").unwrap();
        assert_eq!(pages.len(), 2);
    }

    #[test]
    fn url_from_section_and_slug() {
        let dir = TempDir::new().unwrap();
        write_file(
            dir.path(),
            "recipes/chili.md",
            "---\n{\"title\": \"Chili\"}\n---\n\nSpicy",
        );
        let pages = load_content(dir.path(), "base16-ocean.dark", false, "page.html").unwrap();
        assert_eq!(pages[0].url, "/recipes/chili/");
        assert_eq!(pages[0].section.as_deref(), Some("recipes"));
        assert_eq!(pages[0].slug, "chili");
    }

    #[test]
    fn index_file_url() {
        let dir = TempDir::new().unwrap();
        write_file(
            dir.path(),
            "resume/index.md",
            "---\n{\"title\": \"Resume\", \"template\": \"resume\"}\n---\n",
        );
        let pages = load_content(dir.path(), "base16-ocean.dark", false, "page.html").unwrap();
        assert_eq!(pages[0].url, "/resume/");
        assert_eq!(pages[0].template, "resume.html");
    }

    #[test]
    fn sorts_by_date_descending() {
        let dir = TempDir::new().unwrap();
        write_file(
            dir.path(),
            "posts/old.md",
            "---\n{\"title\": \"Old\", \"date\": \"2023-01-01\"}\n---\n\nOld",
        );
        write_file(
            dir.path(),
            "posts/new.md",
            "---\n{\"title\": \"New\", \"date\": \"2024-06-01\"}\n---\n\nNew",
        );
        let pages = load_content(dir.path(), "base16-ocean.dark", false, "page.html").unwrap();
        assert_eq!(pages[0].title, "New");
        assert_eq!(pages[1].title, "Old");
    }

    #[test]
    fn multiple_sections() {
        let dir = TempDir::new().unwrap();
        write_file(
            dir.path(),
            "posts/item.md",
            "---\n{\"title\": \"Post\"}\n---\n\nContent",
        );
        write_file(
            dir.path(),
            "recipes/item.md",
            "---\n{\"title\": \"Recipe\"}\n---\n\nContent",
        );
        let pages = load_content(dir.path(), "base16-ocean.dark", false, "page.html").unwrap();
        assert_eq!(pages.len(), 2);
        let sections: std::collections::HashSet<_> =
            pages.iter().filter_map(|p| p.section.as_deref()).collect();
        assert!(sections.contains("posts"));
        assert!(sections.contains("recipes"));
    }
}
