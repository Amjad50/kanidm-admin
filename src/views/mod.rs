use askama::Template;
use askama_web::WebTemplate;
use time::OffsetDateTime;

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
        Self {
            current_section,
            user_displayname: user.displayname.clone(),
            user_spn: user.spn.clone(),
            user_initials: initials(&user.displayname),
            privileged: user.privileged,
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

// ── Relative-time formatting ─────────────────────────────────────────────────

/// Format a past `OffsetDateTime` relative to now.
/// Examples: "just now", "38 minutes ago", "2 hours ago", "5 days ago".
pub fn format_relative_past(t: OffsetDateTime) -> String {
    format_relative_past_at(t, OffsetDateTime::now_utc())
}

/// Format a future `OffsetDateTime` relative to now.
/// Examples: "in less than a minute", "in 6 hours 22 minutes", "in 3 days".
pub fn format_relative_future(t: OffsetDateTime) -> String {
    format_relative_future_at(t, OffsetDateTime::now_utc())
}

/// Format the remaining duration until a future `OffsetDateTime`, without a
/// leading "in " prefix. Examples: "less than a minute", "22 minutes",
/// "6 hours 22 minutes", "3 days".
pub fn format_relative_remaining(t: OffsetDateTime) -> String {
    format_relative_remaining_at(t, OffsetDateTime::now_utc())
}

fn format_relative_remaining_at(t: OffsetDateTime, now: OffsetDateTime) -> String {
    let secs = (t - now).whole_seconds();
    if secs <= 0 {
        return "expired".to_string();
    }
    let secs = secs as u64;
    if secs < 60 {
        "less than a minute".to_string()
    } else if secs < 3600 {
        let n = secs / 60;
        if n == 1 { "1 minute".to_string() } else { format!("{n} minutes") }
    } else if secs < 86400 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        if hours == 1 && mins < 5 {
            "1 hour".to_string()
        } else if mins < 5 {
            format!("{hours} hours")
        } else {
            format!("{hours} hours {mins} minutes")
        }
    } else if secs < 2592000 {
        let n = secs / 86400;
        if n == 1 { "1 day".to_string() } else { format!("{n} days") }
    } else {
        let n = secs / 2592000;
        if n == 1 { "1 month".to_string() } else { format!("{n} months") }
    }
}

fn format_relative_past_at(t: OffsetDateTime, now: OffsetDateTime) -> String {
    let secs = (now - t).whole_seconds().max(0) as u64;
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        let n = secs / 60;
        if n == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{n} minutes ago")
        }
    } else if secs < 86400 {
        let n = secs / 3600;
        if n == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{n} hours ago")
        }
    } else if secs < 604800 {
        let n = secs / 86400;
        if n == 1 {
            "1 day ago".to_string()
        } else {
            format!("{n} days ago")
        }
    } else if secs < 2592000 {
        let n = secs / 604800;
        if n == 1 {
            "1 week ago".to_string()
        } else {
            format!("{n} weeks ago")
        }
    } else {
        let n = secs / 2592000;
        if n == 1 {
            "1 month ago".to_string()
        } else {
            format!("{n} months ago")
        }
    }
}

fn format_relative_future_at(t: OffsetDateTime, now: OffsetDateTime) -> String {
    let secs = (t - now).whole_seconds();
    if secs <= 0 {
        return "just now".to_string();
    }
    let secs = secs as u64;
    if secs < 60 {
        "in less than a minute".to_string()
    } else if secs < 3600 {
        let n = secs / 60;
        if n == 1 {
            "in 1 minute".to_string()
        } else {
            format!("in {n} minutes")
        }
    } else if secs < 86400 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        if hours == 1 && mins < 5 {
            "in 1 hour".to_string()
        } else if mins < 5 {
            format!("in {hours} hours")
        } else {
            format!("in {hours} hours {mins} minutes")
        }
    } else if secs < 2592000 {
        let n = secs / 86400;
        if n == 1 {
            "in 1 day".to_string()
        } else {
            format!("in {n} days")
        }
    } else {
        let n = secs / 2592000;
        if n == 1 {
            "in 1 month".to_string()
        } else {
            format!("in {n} months")
        }
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::{format_relative_future_at, format_relative_past_at, format_relative_remaining_at, initials};

    // ── initials ──────────────────────────────────────────────────────────────

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

    // ── format_relative_past ──────────────────────────────────────────────────

    #[test]
    fn past_just_now() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 11:59:30 UTC);
        assert_eq!(format_relative_past_at(t, now), "just now");
    }

    #[test]
    fn past_one_minute() {
        let now = datetime!(2026-05-15 12:01:00 UTC);
        let t = datetime!(2026-05-15 12:00:00 UTC);
        assert_eq!(format_relative_past_at(t, now), "1 minute ago");
    }

    #[test]
    fn past_minutes() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 11:22:00 UTC);
        assert_eq!(format_relative_past_at(t, now), "38 minutes ago");
    }

    #[test]
    fn past_one_hour() {
        let now = datetime!(2026-05-15 13:00:00 UTC);
        let t = datetime!(2026-05-15 12:00:00 UTC);
        assert_eq!(format_relative_past_at(t, now), "1 hour ago");
    }

    #[test]
    fn past_hours() {
        let now = datetime!(2026-05-15 15:00:00 UTC);
        let t = datetime!(2026-05-15 12:00:00 UTC);
        assert_eq!(format_relative_past_at(t, now), "3 hours ago");
    }

    #[test]
    fn past_one_day() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-14 12:00:00 UTC);
        assert_eq!(format_relative_past_at(t, now), "1 day ago");
    }

    #[test]
    fn past_days() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-10 12:00:00 UTC);
        assert_eq!(format_relative_past_at(t, now), "5 days ago");
    }

    #[test]
    fn past_weeks() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-04-24 12:00:00 UTC);
        assert_eq!(format_relative_past_at(t, now), "3 weeks ago");
    }

    #[test]
    fn past_months() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-02-01 12:00:00 UTC);
        assert_eq!(format_relative_past_at(t, now), "3 months ago");
    }

    // ── format_relative_future ────────────────────────────────────────────────

    #[test]
    fn future_past_is_just_now() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 11:59:00 UTC);
        assert_eq!(format_relative_future_at(t, now), "just now");
    }

    #[test]
    fn future_less_than_a_minute() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 12:00:30 UTC);
        assert_eq!(format_relative_future_at(t, now), "in less than a minute");
    }

    #[test]
    fn future_one_minute() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 12:01:00 UTC);
        assert_eq!(format_relative_future_at(t, now), "in 1 minute");
    }

    #[test]
    fn future_minutes() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 12:45:00 UTC);
        assert_eq!(format_relative_future_at(t, now), "in 45 minutes");
    }

    #[test]
    fn future_one_hour_exact() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 13:00:00 UTC);
        assert_eq!(format_relative_future_at(t, now), "in 1 hour");
    }

    #[test]
    fn future_hours_no_minutes() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 15:02:00 UTC);
        assert_eq!(format_relative_future_at(t, now), "in 3 hours");
    }

    #[test]
    fn future_hours_and_minutes() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 18:22:00 UTC);
        assert_eq!(format_relative_future_at(t, now), "in 6 hours 22 minutes");
    }

    #[test]
    fn future_one_day() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-16 12:00:00 UTC);
        assert_eq!(format_relative_future_at(t, now), "in 1 day");
    }

    #[test]
    fn future_days() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-18 12:00:00 UTC);
        assert_eq!(format_relative_future_at(t, now), "in 3 days");
    }

    #[test]
    fn future_months() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-08-15 12:00:00 UTC);
        assert_eq!(format_relative_future_at(t, now), "in 3 months");
    }

    // ── format_relative_remaining ─────────────────────────────────────────────

    #[test]
    fn remaining_minutes() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 12:22:00 UTC);
        assert_eq!(format_relative_remaining_at(t, now), "22 minutes");
    }

    #[test]
    fn remaining_hours_and_minutes() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 18:22:00 UTC);
        assert_eq!(format_relative_remaining_at(t, now), "6 hours 22 minutes");
    }

    #[test]
    fn remaining_expired() {
        let now = datetime!(2026-05-15 12:00:00 UTC);
        let t = datetime!(2026-05-15 11:00:00 UTC);
        assert_eq!(format_relative_remaining_at(t, now), "expired");
    }
}
