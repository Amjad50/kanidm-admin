pub mod dropdown;
pub mod icons;
pub mod pagination;
pub mod partials;
pub mod sessions_card;
pub mod time;
pub mod toast;

use askama::Template;
use askama_web::WebTemplate;

use crate::auth::AdminUser;

pub use time::{
    format_absolute, format_relative_future, format_relative_past, format_relative_remaining,
};

// ── Placeholder ──────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "placeholder.html")]
pub struct PlaceholderView {
    pub base: BaseFields,
    pub section_label: &'static str,
    pub message: &'static str,
    pub phase_label: &'static str,
}

// ── Error views (standalone — no BaseFields needed) ──────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "error_forbidden.html")]
pub struct ForbiddenView {
    pub admin_group: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "error_not_found.html")]
pub struct NotFoundView {}

#[derive(Template, WebTemplate)]
#[template(path = "error_server.html")]
pub struct ServerErrorView {
    pub category: &'static str,
}

// ── BaseFields ────────────────────────────────────────────────────────────────

pub struct BaseFields {
    pub current_section: &'static str,
    pub user_displayname: String,
    pub user_spn: String,
    pub user_initials: String,
    pub privileged: bool,
    /// Pre-rendered JSON for the topbar user-menu dropdown, safe to embed
    /// inside `data-dropdown='...'`.
    pub user_menu_json: String,
}

impl BaseFields {
    pub fn new(user: &AdminUser, current_section: &'static str) -> Self {
        Self {
            current_section,
            user_displayname: user.displayname.clone(),
            user_spn: user.spn.clone(),
            user_initials: initials(&user.displayname),
            privileged: user.privileged,
            user_menu_json: user_menu_json(),
        }
    }
}

fn user_menu_json() -> String {
    use crate::views::dropdown::{DropdownConfig, DropdownItem};
    let items = vec![
        DropdownItem::link("My profile", "/me").with_icon("user"),
        DropdownItem::link("My sessions", "/me/sessions").with_icon("users"),
        DropdownItem::Divider,
        DropdownItem::htmx_post("Log out", "/logout")
            .with_icon("log-out")
            .danger(),
    ];
    let mut cfg = DropdownConfig::new(items);
    cfg.align = Some("right");
    cfg.to_attr_value()
}

pub fn initials_for_login(name: &str) -> String {
    initials(name)
}

pub(crate) fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.len() {
        0 => String::new(),
        1 => words[0].chars().take(2).collect::<String>().to_uppercase(),
        _ => {
            let first = words.first().and_then(|w| w.chars().next()).unwrap_or(' ');
            let last = words.last().and_then(|w| w.chars().next()).unwrap_or(' ');
            format!("{}{}", first, last).to_uppercase()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::initials;

    #[test]
    fn test_initials_two_words() {
        assert_eq!(initials("System Administrator"), "SA");
    }

    #[test]
    fn test_initials_two_words_alice() {
        assert_eq!(initials("Alice Smith"), "AS");
    }

    #[test]
    fn test_initials_single_word() {
        assert_eq!(initials("admin"), "AD");
    }

    #[test]
    fn test_initials_empty() {
        assert_eq!(initials(""), "");
    }

    #[test]
    fn test_initials_extra_spaces() {
        assert_eq!(initials("  Alice   Smith  "), "AS");
    }
}
