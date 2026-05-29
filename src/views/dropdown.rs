// Server-side helpers for action menus.
//
// Two concerns share this module:
//   1. `DropdownConfig` + `DropdownItem` — the JSON config consumed by the
//      Preact <Dropdown> component when a row's kebab is clicked. Mirror of
//      islands/dropdown.tsx.
//   2. `render_actions_cell` — a row-level helper that decides whether to
//      render a single icon button (when there's exactly one non-divider
//      action) or a kebab + dropdown menu (when there are several). One call
//      site per row replaces hand-rolled per-template kebab markup.
//
// Construct a `DropdownItems` vec, call `.to_attr_value()` to get a string
// safe for putting inside an HTML attribute (single-quote delimited, with
// double-quoted JSON keys). Stick it on a kebab button as
//   data-dropdown='{{ row.actions_json|safe }}'
// and the global mountDropdowns() click handler will wire the rest.

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum DropdownItem {
    Link {
        label: String,
        href: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<&'static str>,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        danger: bool,
    },
    Htmx {
        label: String,
        #[serde(rename = "hxGet", skip_serializing_if = "Option::is_none")]
        hx_get: Option<String>,
        #[serde(rename = "hxPost", skip_serializing_if = "Option::is_none")]
        hx_post: Option<String>,
        #[serde(rename = "hxTarget", skip_serializing_if = "Option::is_none")]
        hx_target: Option<&'static str>,
        #[serde(rename = "hxSwap", skip_serializing_if = "Option::is_none")]
        hx_swap: Option<&'static str>,
        #[serde(rename = "hxConfirm", skip_serializing_if = "Option::is_none")]
        hx_confirm: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<&'static str>,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        danger: bool,
    },
    Divider,
}

#[derive(Debug, Serialize)]
pub struct DropdownConfig {
    pub items: Vec<DropdownItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<&'static str>,
}

impl DropdownConfig {
    pub fn new(items: Vec<DropdownItem>) -> Self {
        Self { items, align: None }
    }

    /// Serialize to a single-quoted-attribute-safe JSON string. Apostrophes in
    /// any user-controlled fields are escaped as `'` so they can't break
    /// the surrounding `data-dropdown='...'` attribute.
    pub fn to_attr_value(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| "{\"items\":[]}".to_string())
            .replace('\'', "\\u0027")
    }
}

// ── Convenience constructors for common items ─────────────────────────────────

#[allow(dead_code)]
impl DropdownItem {
    pub fn link(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self::Link {
            label: label.into(),
            href: href.into(),
            icon: None,
            danger: false,
        }
    }

    pub fn with_icon(mut self, icon_name: &'static str) -> Self {
        match &mut self {
            Self::Link { icon, .. } | Self::Htmx { icon, .. } => {
                *icon = Some(icon_name);
            }
            Self::Divider => {}
        }
        self
    }

    pub fn danger(mut self) -> Self {
        match &mut self {
            Self::Link { danger, .. } | Self::Htmx { danger, .. } => {
                *danger = true;
            }
            Self::Divider => {}
        }
        self
    }

    pub fn htmx_get(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Htmx {
            label: label.into(),
            hx_get: Some(url.into()),
            hx_post: None,
            hx_target: None,
            hx_swap: None,
            hx_confirm: None,
            icon: None,
            danger: false,
        }
    }

    pub fn htmx_post(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Htmx {
            label: label.into(),
            hx_get: None,
            hx_post: Some(url.into()),
            hx_target: None,
            hx_swap: None,
            hx_confirm: None,
            icon: None,
            danger: false,
        }
    }

    pub fn with_confirm(mut self, prompt: impl Into<String>) -> Self {
        if let Self::Htmx { hx_confirm, .. } = &mut self {
            *hx_confirm = Some(prompt.into());
        }
        self
    }

    pub fn with_target(mut self, target: &'static str) -> Self {
        if let Self::Htmx { hx_target, .. } = &mut self {
            *hx_target = Some(target);
        }
        self
    }

    pub fn with_swap(mut self, swap: &'static str) -> Self {
        if let Self::Htmx { hx_swap, .. } = &mut self {
            *hx_swap = Some(swap);
        }
        self
    }
}

// ── Cell renderer ─────────────────────────────────────────────────────────────

use askama::Template;

#[derive(Template)]
#[template(path = "partials/_actions_kebab.html")]
struct ActionsKebab {
    aria_label: String,
    items_json: String,
}

#[derive(Template)]
#[template(path = "partials/_actions_single_link.html")]
struct ActionsSingleLink {
    label: String,
    href: String,
    icon_paths: &'static str,
    color_classes: &'static str,
}

#[derive(Template)]
#[template(path = "partials/_actions_single_htmx.html")]
struct ActionsSingleHtmx {
    label: String,
    icon_paths: &'static str,
    color_classes: &'static str,
    hx_get: Option<String>,
    hx_post: Option<String>,
    hx_target: &'static str,
    hx_swap: &'static str,
    hx_confirm: Option<String>,
}

/// Render the actions cell for a row.
///
/// Rules:
///   - 0 actionable items → empty string (the cell renders empty).
///   - 1 actionable item → that item's icon button directly. No menu chrome.
///     Divider entries are ignored when counting.
///   - 2+ actionable items → a kebab button whose dropdown JSON config is
///     emitted as `data-dropdown`. The Preact handler opens the menu on
///     click.
///
/// `aria_label` is the screen-reader label for the kebab; ignored when a
/// single-item rendering wins (the item's own label is used instead).
pub fn render_actions_cell(items: Vec<DropdownItem>, aria_label: impl Into<String>) -> String {
    let actionable: Vec<&DropdownItem> = items
        .iter()
        .filter(|i| !matches!(i, DropdownItem::Divider))
        .collect();

    match actionable.len() {
        0 => String::new(),
        1 => render_single(actionable[0]),
        _ => render_kebab(items, aria_label.into()),
    }
}

fn render_kebab(items: Vec<DropdownItem>, aria_label: String) -> String {
    let cfg = DropdownConfig::new(items);
    let items_json = cfg.to_attr_value();
    ActionsKebab {
        aria_label,
        items_json,
    }
    .render()
    .unwrap_or_default()
}

fn render_single(item: &DropdownItem) -> String {
    match item {
        DropdownItem::Link {
            label,
            href,
            icon,
            danger,
        } => ActionsSingleLink {
            label: label.clone(),
            href: href.clone(),
            icon_paths: icon.map(icon_paths_for).unwrap_or(GENERIC_DOT),
            color_classes: color_classes(*danger),
        }
        .render()
        .unwrap_or_default(),
        DropdownItem::Htmx {
            label,
            hx_get,
            hx_post,
            hx_target,
            hx_swap,
            hx_confirm,
            icon,
            danger,
        } => ActionsSingleHtmx {
            label: label.clone(),
            icon_paths: icon.map(icon_paths_for).unwrap_or(GENERIC_DOT),
            color_classes: color_classes(*danger),
            hx_get: hx_get.clone(),
            hx_post: hx_post.clone(),
            hx_target: hx_target.unwrap_or("#overlay-slot"),
            hx_swap: hx_swap.unwrap_or("innerHTML"),
            hx_confirm: hx_confirm.clone(),
        }
        .render()
        .unwrap_or_default(),
        DropdownItem::Divider => String::new(),
    }
}

fn color_classes(danger: bool) -> &'static str {
    if danger {
        "text-tertiary hover:text-danger hover:bg-danger-soft"
    } else {
        "text-tertiary hover:text-primary hover:bg-hover"
    }
}

// ── Icon registry (mirrors the JS map in islands/dropdown.tsx) ────────────────

const GENERIC_DOT: &str = r#"<circle cx="12" cy="12" r="3"/>"#;

fn icon_paths_for(name: &str) -> &'static str {
    match name {
        "edit" => r#"<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5z"/>"#,
        "delete" => {
            r#"<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>"#
        }
        "reset" | "key" => {
            r#"<circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6"/><path d="m15.5 7.5 3 3L22 7l-3-3"/>"#
        }
        "user" => {
            r#"<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>"#
        }
        "members" => {
            r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>"#
        }
        "external" => {
            r#"<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>"#
        }
        "x" => r#"<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>"#,
        _ => GENERIC_DOT,
    }
}
