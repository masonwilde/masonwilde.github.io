# Wilde SSG Theme

## Template Context Variables

All templates receive these variables:

### `site` (merged from variables.json + site.json)
- `site.title` — Site title
- `site.description` — Site description  
- `site.author` — Author name
- `site.base_url` — Base URL for absolute links
- `site.nav[]` — Nav items, each with `.name` and `.url`
- `site.social[]` — Social links, each with `.name` and `.url`
- `site.profile.image` — Profile image path
- `site.profile.subtitle` — Profile subtitle
- `site.favicon` — Favicon path (falls back to profile image)
- `site.colors.light.*` / `site.colors.dark.*` — Theme colors
- `site.fonts.sans` / `site.fonts.mono` — Font stacks
- `site.layout.max_width` / `site.layout.nav_height` — Layout dimensions
- `site.not_found.title` / `.message` / `.image` / `.home_text` — 404 page content
- `site.footer.text` — Optional footer text

### `page` (content pages only)
- `page.title` — Page title
- `page.date` — Publication date (YYYY-MM-DD)
- `page.description` — Page description
- `page.content` — Rendered HTML (use `| safe`)
- `page.url` — Page URL path
- `page.section` — Section name (e.g., "posts")
- `page.slug` — URL slug
- `page.template` — Template filename
- `page.meta` — Raw frontmatter JSON (access custom fields like `page.meta.pdf`)

### `build_year` (all pages)
Current year as integer, for copyright notices.

### `active_nav` (content and section pages)
URL of the active nav item (e.g., "/posts/").

### Section-specific
- `section_name` — Section name (section index pages)
- `pages[]` — List of pages in the section (section index pages)
- `sections[]` — All sections with `.name` and `.pages[]` (home page)

## Customization

Edit `variables.json` to change theme defaults (colors, fonts, layout, text).
Users override specific values in their `site.json` — the engine deep-merges them.

## File Structure

```
theme/
  variables.json              # Theme defaults (colors, fonts, text)
  templates/
    base.html                 # Shell — includes components, loads JS
    components/
      head.html               # <head> with meta, CSS vars, OG tags
      nav.html                # Site header with navigation
      footer.html             # Footer with copyright
    page.html                 # Article page
    section.html              # Section index
    home.html                 # Homepage
    resume.html               # PDF embed page  
    404.html                  # Error page
  static/
    css/style.css             # Layout and structure (uses CSS vars)
    js/main.js                # Theme toggle + hamburger menu
```
