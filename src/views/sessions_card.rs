//! Shared sessions-card view model.
//!
//! Both `/admin/people/{id}/sessions` and `/me/sessions` render the same
//! visual card. The handler builds a `SessionsCard` with context-specific
//! URLs + labels, renders it to HTML, and embeds the resulting string in its
//! parent template via `{{ card_html|safe }}` — same shape as
//! `crate::views::partials::Modal::body_html`.

use askama::Template;

use crate::handlers::people::sessions::SessionRow;

/// Data driving the shared sessions card UI.
///
/// Field semantics:
/// * `rows` — the rows to render (already context-formatted; each row carries
///   its own `revoke_url`).
/// * `error` — optional banner text rendered above the table.
/// * `hx_target_id` — DOM id of the parent fragment that bulk + per-row
///   actions swap. Admin: `"tab-content"`. Self: `"sessions-table"`.
/// * `bulk_revoke_url` — endpoint for the destructive top button
///   (`destroy_all` in admin, `destroy_others` in self).
/// * `bulk_revoke_label` — label on that button.
/// * `bulk_revoke_confirm` — `hx-confirm` text on that button.
/// * `revoke_row_confirm` — `hx-confirm` text on every per-row Revoke button.
/// * `empty_subtitle` — small grey description text under the header count.
/// * `current_session_id` — when `Some(uuid_str)`, the matching row renders a
///   "this session" pill and its Revoke button is disabled. Always `None` in
///   the admin context (no notion of "current" — admins viewing someone
///   else's sessions don't share a session id).
/// * `show_inactive` — whether the current view is including revoked +
///   past-expiry rows. Drives the toggle button's label.
/// * `show_inactive_url` — URL to navigate to when toggling. Flips between
///   `?show_inactive=1` and the bare base path.
#[derive(Template)]
#[template(path = "sessions/_card.html")]
pub struct SessionsCard {
    pub rows: Vec<SessionRow>,
    pub error: Option<String>,
    pub hx_target_id: String,
    pub bulk_revoke_url: String,
    pub bulk_revoke_label: String,
    pub bulk_revoke_confirm: String,
    pub revoke_row_confirm: String,
    pub empty_subtitle: String,
    pub current_session_id: Option<String>,
    pub show_inactive: bool,
    pub show_inactive_url: String,
}
