use std::fs;
use std::io::Write as _;
use std::path::Path;
use tempfile::TempDir;
use wilde_ssg::builder::build_site;

fn write_file(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

fn setup_site(dir: &Path) {
    write_file(dir, "site.json", r#"{"title":"Test Site","description":"A test","author":"Tester","base_url":"https://example.com","nav":[{"name":"Posts","url":"/posts/"}],"social":[],"profile":{"image":"/img/test.jpg","subtitle":"Testing"}}"#);
    write_file(dir, "content/posts/hello.md", "---\n{\"title\": \"Hello\", \"date\": \"2024-01-01\", \"description\": \"A greeting\"}\n---\n\n# Hello World");
    write_file(dir, "theme/templates/base.html", "<!DOCTYPE html><html><head><title>{% if page.title %}{{ page.title }} | {% endif %}{{ site.title }}</title></head><body>{% block content %}{% endblock content %}</body></html>");
    write_file(dir, "theme/templates/page.html", "{% extends \"base.html\" %}{% block content %}<article><h1>{{ page.title }}</h1>{{ page.content | safe }}</article>{% endblock content %}");
    write_file(dir, "theme/templates/section.html", "{% extends \"base.html\" %}{% block content %}<h1>{{ section_name }}</h1>{% for p in pages %}<a href=\"{{ p.url }}\">{{ p.title }}</a>{% endfor %}{% endblock content %}");
    write_file(dir, "theme/templates/home.html", "{% extends \"base.html\" %}{% block content %}<h1>{{ site.title }}</h1>{% endblock content %}");
    write_file(dir, "theme/templates/404.html", "{% extends \"base.html\" %}{% block content %}<h1>404</h1>{% endblock content %}");
    write_file(dir, "theme/static/css/style.css", "body { margin: 0; }");
    write_file(dir, "static/img/test.jpg", "fake-image-data");
}

#[test]
fn creates_output_directory() {
    let dir = TempDir::new().unwrap();
    setup_site(dir.path());
    build_site(dir.path()).unwrap();
    assert!(dir.path().join("public").is_dir());
}

#[test]
fn renders_content_pages() {
    let dir = TempDir::new().unwrap();
    setup_site(dir.path());
    build_site(dir.path()).unwrap();
    let page = dir.path().join("public/posts/hello/index.html");
    assert!(page.exists());
    let html = fs::read_to_string(page).unwrap();
    assert!(html.contains("Hello"));
    assert!(html.contains("<h1>"));
}

#[test]
fn renders_section_index() {
    let dir = TempDir::new().unwrap();
    setup_site(dir.path());
    build_site(dir.path()).unwrap();
    let index = dir.path().join("public/posts/index.html");
    assert!(index.exists());
    let html = fs::read_to_string(index).unwrap();
    assert!(html.contains("Hello"));
}

#[test]
fn renders_homepage() {
    let dir = TempDir::new().unwrap();
    setup_site(dir.path());
    build_site(dir.path()).unwrap();
    let home = dir.path().join("public/index.html");
    assert!(home.exists());
    assert!(fs::read_to_string(home).unwrap().contains("Test Site"));
}

#[test]
fn renders_404() {
    let dir = TempDir::new().unwrap();
    setup_site(dir.path());
    build_site(dir.path()).unwrap();
    let page = dir.path().join("public/404.html");
    assert!(page.exists());
    assert!(fs::read_to_string(page).unwrap().contains("404"));
}

#[test]
fn copies_static_files() {
    let dir = TempDir::new().unwrap();
    setup_site(dir.path());
    build_site(dir.path()).unwrap();
    assert!(dir.path().join("public/css/style.css").exists());
}

#[test]
fn clean_build_removes_old_output() {
    let dir = TempDir::new().unwrap();
    setup_site(dir.path());
    let output = dir.path().join("public");
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("stale.html"), "old stuff").unwrap();
    build_site(dir.path()).unwrap();
    assert!(!output.join("stale.html").exists());
    assert!(output.join("index.html").exists());
}

#[test]
fn missing_site_json_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = build_site(dir.path());
    assert!(result.is_err());
}

#[test]
fn missing_templates_dir_returns_error() {
    let dir = TempDir::new().unwrap();
    write_file(dir.path(), "site.json", r#"{"title":"Test","profile":{"image":"/img/t.jpg","subtitle":"t"}}"#);
    write_file(dir.path(), "content/posts/hello.md", "---\n{\"title\": \"Hello\"}\n---\n\nHi");
    let result = build_site(dir.path());
    assert!(result.is_err());
}

#[test]
fn section_index_has_title() {
    let dir = TempDir::new().unwrap();
    setup_site(dir.path());
    build_site(dir.path()).unwrap();
    let index = dir.path().join("public/posts/index.html");
    let html = fs::read_to_string(index).unwrap();
    assert!(html.contains("Posts |"), "Section index page should have a title derived from the section name");
}

#[test]
fn copies_both_theme_and_site_static() {
    let dir = TempDir::new().unwrap();
    setup_site(dir.path());
    build_site(dir.path()).unwrap();
    assert!(dir.path().join("public/css/style.css").exists(), "theme static should be copied");
    assert!(dir.path().join("public/img/test.jpg").exists(), "site static should be copied");
}

#[test]
fn custom_page_template_names() {
    let dir = TempDir::new().unwrap();
    write_file(dir.path(), "site.json", r#"{"title":"Test","profile":{"image":"/t.jpg","subtitle":"t"},"pages":{"home":"landing.html","not_found":"oops.html"}}"#);
    write_file(dir.path(), "content/posts/hi.md", "---\n{\"title\":\"Hi\"}\n---\n\nHi");
    write_file(dir.path(), "theme/templates/base.html", "<html><body>{% block content %}{% endblock content %}</body></html>");
    write_file(dir.path(), "theme/templates/page.html", "{% extends \"base.html\" %}{% block content %}{{ page.content | safe }}{% endblock content %}");
    write_file(dir.path(), "theme/templates/section.html", "{% extends \"base.html\" %}{% block content %}section{% endblock content %}");
    write_file(dir.path(), "theme/templates/landing.html", "{% extends \"base.html\" %}{% block content %}LANDING{% endblock content %}");
    write_file(dir.path(), "theme/templates/oops.html", "{% extends \"base.html\" %}{% block content %}OOPS{% endblock content %}");
    build_site(dir.path()).unwrap();
    assert!(fs::read_to_string(dir.path().join("public/index.html")).unwrap().contains("LANDING"));
    assert!(fs::read_to_string(dir.path().join("public/404.html")).unwrap().contains("OOPS"));
}
