use std::collections::HashMap;
use std::fs;
use std::path::Path;

use tera::{Context, Tera};

use crate::config::SiteConfig;
use crate::content::{load_content, Page};

fn empty_page_with_title(title: &str) -> serde_json::Value {
    serde_json::json!({"title": title, "description": "", "url": "", "date": "", "content": "", "meta": {}})
}

fn empty_page() -> serde_json::Value {
    empty_page_with_title("")
}

fn section_display_name(name: &str) -> String {
    name.replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn current_year() -> u64 {
    1970 + std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() / 31_557_600
}

fn deep_merge(base: &mut serde_json::Value, overlay: &serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                let base_val = base_map.entry(key.clone()).or_insert(serde_json::Value::Null);
                deep_merge(base_val, overlay_val);
            }
        }
        (base_val, overlay_val) => {
            *base_val = overlay_val.clone();
        }
    }
}

fn load_template_context(base_dir: &Path, config: &SiteConfig) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let vars_path = base_dir.join(&config.theme_dir).join("variables.json");
    let mut context = if vars_path.exists() {
        serde_json::from_str(&fs::read_to_string(&vars_path)?)?
    } else {
        serde_json::Value::Object(Default::default())
    };

    let site_json = serde_json::to_value(config)?;
    deep_merge(&mut context, &site_json);

    Ok(context)
}

fn build_section_map(pages: &[Page]) -> HashMap<String, Vec<Page>> {
    let mut section_map: HashMap<String, Vec<Page>> = HashMap::new();
    for page in pages {
        if let Some(ref section) = page.section {
            // Cloning is required here because Tera's Context::insert needs owned,
            // serializable data — it cannot work with references to Pages.
            section_map
                .entry(section.clone())
                .or_default()
                .push(page.clone());
        }
    }
    section_map
}

fn prepare_output_dir(base_dir: &Path, config: &SiteConfig) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let output = base_dir.join(&config.output_dir);
    if output.exists() {
        fs::remove_dir_all(&output)?;
    }
    fs::create_dir_all(&output)?;
    Ok(output)
}

fn render_pages(
    tera: &Tera,
    pages: &[Page],
    site_context: &serde_json::Value,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let year = current_year();
    for page in pages {
        let mut context = Context::new();
        context.insert("site", site_context);
        context.insert("page", page);
        context.insert("build_year", &year);
        if let Some(ref section) = page.section {
            context.insert("active_nav", &format!("/{section}/"));
        }

        let html = tera.render(&page.template, &context)?;
        let page_dir = output.join(page.url.trim_matches('/'));
        fs::create_dir_all(&page_dir)?;
        fs::write(page_dir.join("index.html"), html)?;
    }
    Ok(())
}

fn render_section_indexes(
    tera: &Tera,
    config: &SiteConfig,
    section_map: &HashMap<String, Vec<Page>>,
    site_context: &serde_json::Value,
    content_dir: &Path,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let year = current_year();
    let empty: Vec<Page> = Vec::new();

    let mut sections_to_render: HashMap<String, &[Page]> = HashMap::new();

    // Sections with content
    for (name, pages) in section_map {
        if !pages.iter().any(|p| p.slug == "index") {
            sections_to_render.insert(name.clone(), pages);
        }
    }

    // Also generate indexes for content subdirectories that exist but have no pages
    if content_dir.exists() {
        for entry in fs::read_dir(content_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                sections_to_render.entry(name).or_insert(&empty);
            }
        }
    }

    for (name, section_pages) in &sections_to_render {
        let mut context = Context::new();
        context.insert("site", site_context);
        context.insert("section_name", name);
        context.insert("pages", section_pages);
        context.insert("page", &empty_page_with_title(&section_display_name(name)));
        context.insert("active_nav", &format!("/{name}/"));
        context.insert("build_year", &year);

        let html = tera.render(&config.pages.section, &context)?;
        let section_dir = output.join(name);
        fs::create_dir_all(&section_dir)?;
        fs::write(section_dir.join("index.html"), html)?;
    }
    Ok(())
}

fn render_special_pages(
    tera: &Tera,
    config: &SiteConfig,
    site_context: &serde_json::Value,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let year = current_year();

    let mut context = Context::new();
    context.insert("site", site_context);
    context.insert("page", &empty_page());
    context.insert("build_year", &year);
    fs::write(output.join("index.html"), tera.render(&config.pages.home, &context)?)?;

    let mut context = Context::new();
    context.insert("site", site_context);
    context.insert("page", &empty_page_with_title("Not Found"));
    context.insert("build_year", &year);
    fs::write(output.join("404.html"), tera.render(&config.pages.not_found, &context)?)?;

    Ok(())
}

fn copy_static_assets(base_dir: &Path, config: &SiteConfig, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let theme_static = base_dir.join(&config.theme_dir).join("static");
    if theme_static.exists() {
        copy_dir_recursive(&theme_static, output)?;
    }

    let site_static = base_dir.join(&config.static_dir);
    if site_static.exists() {
        copy_dir_recursive(&site_static, output)?;
    }

    Ok(())
}

pub fn build_site(base_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config: SiteConfig = serde_json::from_str(
        &fs::read_to_string(base_dir.join("site.json"))
            .map_err(|e| format!("Failed to read site.json: {e}"))?
    ).map_err(|e| format!("Failed to parse site.json: {e}"))?;

    let template_glob = format!("{}/{}/templates/**/*.html", base_dir.display(), config.theme_dir);
    let tera = Tera::new(&template_glob)
        .map_err(|e| format!("Failed to load templates from {}: {e}", config.theme_dir))?;

    let site_context = load_template_context(base_dir, &config)?;

    let pages = load_content(
        &base_dir.join(&config.content_dir),
        &config.code_theme,
        config.include_drafts,
        &config.pages.default,
    )?;

    let section_map = build_section_map(&pages);
    let output = prepare_output_dir(base_dir, &config)?;

    render_pages(&tera, &pages, &site_context, &output)?;
    render_section_indexes(&tera, &config, &section_map, &site_context, &base_dir.join(&config.content_dir), &output)?;
    render_special_pages(&tera, &config, &site_context, &output)?;
    copy_static_assets(base_dir, &config, &output)?;

    println!("Built {} pages in {} sections", pages.len(), section_map.len());
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dest = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&dest)?;
            copy_dir_recursive(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_merge_overwrites_scalar() {
        let mut base = serde_json::json!({"a": 1, "b": 2});
        let overlay = serde_json::json!({"a": 10});
        deep_merge(&mut base, &overlay);
        assert_eq!(base["a"], 10);
        assert_eq!(base["b"], 2);
    }

    #[test]
    fn deep_merge_nested_objects() {
        let mut base = serde_json::json!({"colors": {"bg": "#fff", "text": "#000"}});
        let overlay = serde_json::json!({"colors": {"bg": "#111"}});
        deep_merge(&mut base, &overlay);
        assert_eq!(base["colors"]["bg"], "#111");
        assert_eq!(base["colors"]["text"], "#000");
    }

    #[test]
    fn deep_merge_adds_new_keys() {
        let mut base = serde_json::json!({"a": 1});
        let overlay = serde_json::json!({"b": 2});
        deep_merge(&mut base, &overlay);
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"], 2);
    }

    #[test]
    fn deep_merge_overlay_wins_type_mismatch() {
        let mut base = serde_json::json!({"a": {"nested": true}});
        let overlay = serde_json::json!({"a": "flat"});
        deep_merge(&mut base, &overlay);
        assert_eq!(base["a"], "flat");
    }

    #[test]
    fn empty_page_has_required_fields() {
        let page = empty_page();
        assert_eq!(page["title"], "");
        assert_eq!(page["description"], "");
        assert!(page["meta"].is_object());
    }

    #[test]
    fn empty_page_with_title_sets_title() {
        let page = empty_page_with_title("Test Title");
        assert_eq!(page["title"], "Test Title");
    }

    #[test]
    fn section_display_name_capitalizes() {
        assert_eq!(section_display_name("posts"), "Posts");
        assert_eq!(section_display_name("my_recipes"), "My Recipes");
    }

    #[test]
    fn current_year_is_reasonable() {
        let year = current_year();
        assert!(year >= 2024);
        assert!(year <= 2100);
    }

    use std::io::Write as _;
    use tempfile::TempDir;

    fn write_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn load_template_context_uses_variables_json_defaults() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "site.json", r#"{"title":"Test","profile":{"image":"/t.jpg","subtitle":"t"}}"#);
        write_file(dir.path(), "theme/variables.json", r##"{"colors":{"bg":"#fff","text":"#000"},"title":"Default"}"##);

        let config: SiteConfig = serde_json::from_str(
            &fs::read_to_string(dir.path().join("site.json")).unwrap()
        ).unwrap();
        let ctx = load_template_context(dir.path(), &config).unwrap();

        // site.json title wins over variables.json default
        assert_eq!(ctx["title"], "Test");
        // variables.json colors are preserved since site.json doesn't override them
        assert_eq!(ctx["colors"]["bg"], "#fff");
        assert_eq!(ctx["colors"]["text"], "#000");
    }

    #[test]
    fn load_template_context_without_variables_json() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "site.json", r#"{"title":"Test","profile":{"image":"/t.jpg","subtitle":"t"}}"#);

        let config: SiteConfig = serde_json::from_str(
            &fs::read_to_string(dir.path().join("site.json")).unwrap()
        ).unwrap();
        let ctx = load_template_context(dir.path(), &config).unwrap();

        assert_eq!(ctx["title"], "Test");
    }
}
