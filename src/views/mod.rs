use askama::Template;
use askama_web::WebTemplate;

use crate::auth::AdminUser;

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
#[template(path = "error_unauthenticated.html")]
pub struct UnauthenticatedView {
    pub kanidm_url: String,
}

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
}

impl BaseFields {
    pub fn new(user: &AdminUser, current_section: &'static str) -> Self {
        let initials = initials(&user.displayname);
        Self {
            current_section,
            user_displayname: user.displayname.clone(),
            user_spn: user.spn.clone(),
            user_initials: initials,
            privileged: false, // populated in Task 0D
        }
    }
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
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
        assert_eq!(initials("admin"), "A");
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
