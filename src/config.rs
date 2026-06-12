use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NavItem {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SocialLink {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub image: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageTemplates {
    #[serde(default = "default_home_template")]
    pub home: String,
    #[serde(default = "default_section_template")]
    pub section: String,
    #[serde(default = "default_not_found_template")]
    pub not_found: String,
    #[serde(default = "default_page_template")]
    pub default: String,
}

impl Default for PageTemplates {
    fn default() -> Self {
        Self {
            home: default_home_template(),
            section: default_section_template(),
            not_found: default_not_found_template(),
            default: default_page_template(),
        }
    }
}

fn default_home_template() -> String { "home.html".into() }
fn default_section_template() -> String { "section.html".into() }
fn default_not_found_template() -> String { "404.html".into() }
fn default_page_template() -> String { "page.html".into() }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteConfig {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub nav: Vec<NavItem>,
    #[serde(default)]
    pub social: Vec<SocialLink>,
    pub profile: Profile,

    #[serde(default = "default_content_dir")]
    pub content_dir: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_static_dir")]
    pub static_dir: String,
    #[serde(default = "default_theme_dir")]
    pub theme_dir: String,
    #[serde(default = "default_code_theme")]
    pub code_theme: String,
    #[serde(default)]
    pub include_drafts: bool,
    #[serde(default)]
    pub pages: PageTemplates,
}

fn default_content_dir() -> String {
    "content".into()
}
fn default_output_dir() -> String {
    "public".into()
}
fn default_static_dir() -> String {
    "static".into()
}
fn default_theme_dir() -> String {
    "theme".into()
}
fn default_code_theme() -> String {
    "base16-ocean.dark".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_page_templates() {
        let templates = PageTemplates::default();
        assert_eq!(templates.home, "home.html");
        assert_eq!(templates.section, "section.html");
        assert_eq!(templates.not_found, "404.html");
        assert_eq!(templates.default, "page.html");
    }

    #[test]
    fn config_without_theme_fields_uses_defaults() {
        let json = r#"{"title":"Test","profile":{"image":"/img/t.jpg","subtitle":"t"}}"#;
        let config: SiteConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.theme_dir, "theme");
        assert_eq!(config.pages.home, "home.html");
        assert_eq!(config.pages.section, "section.html");
        assert_eq!(config.pages.not_found, "404.html");
        assert_eq!(config.pages.default, "page.html");
    }

    #[test]
    fn config_with_custom_theme_dir() {
        let json = r#"{"title":"Test","profile":{"image":"/img/t.jpg","subtitle":"t"},"theme_dir":"my-theme"}"#;
        let config: SiteConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.theme_dir, "my-theme");
    }

    #[test]
    fn config_with_custom_page_templates() {
        let json = r#"{"title":"Test","profile":{"image":"/img/t.jpg","subtitle":"t"},"pages":{"home":"landing.html","not_found":"oops.html"}}"#;
        let config: SiteConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.pages.home, "landing.html");
        assert_eq!(config.pages.not_found, "oops.html");
        assert_eq!(config.pages.section, "section.html");
        assert_eq!(config.pages.default, "page.html");
    }
}
