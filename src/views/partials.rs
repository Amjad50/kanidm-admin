use askama::Template;

// ── Modal frame ───────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/_modal.html")]
pub struct Modal {
    pub title: String,
    /// Pre-rendered SVG markup; `None` = no icon.
    pub icon_svg: Option<&'static str>,
    /// Tailwind class for the icon colour, e.g. `"text-danger"`.
    /// Use `"text-tertiary"` when no specific colour is desired.
    pub icon_color_class: &'static str,
    /// Pre-rendered HTML for the modal body (rendered via a nested Template).
    pub body_html: String,
    /// Pre-rendered HTML for the modal footer (rendered via a nested Template).
    pub footer_html: String,
    /// Tailwind max-w utility, e.g. `"max-w-md"`.
    pub size_class: &'static str,
}

// ── Destructive confirm body ──────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/_destructive_confirm.html")]
pub struct DestructiveConfirm {
    /// Short lead sentence, e.g. `"You're about to delete:"`. Plain text — auto-escaped.
    pub lead_text: String,
    /// Pre-rendered identity row HTML for the thing being deleted.
    pub target_html: String,
    /// Bullet points shown under "What happens".
    pub consequences: Vec<String>,
    /// The exact string the user must type to unlock the confirm button.
    pub confirm_token: String,
    /// JSON-encoded version of `confirm_token` for safe embedding in `<script>`.
    pub confirm_token_js: String,
    /// Label above the confirm input, e.g. `"Type the SPN to confirm:"`.
    pub confirm_label: String,
    /// DOM id suffix; used to build `confirm-input-{id}` and `confirm-submit-{id}`.
    pub input_id: String,
    /// Server-side error message to display inside the modal body.
    pub error: Option<String>,
}

// ── Identity row fragment ─────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/_identity_row.html")]
pub struct IdentityRow {
    pub initials: String,
    pub displayname: String,
    pub spn: String,
}

// ── One-time secret reveal ────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/_one_time_secret.html")]
pub struct OneTimeSecret {
    /// Display label, e.g. "Reset URL", "RADIUS secret".
    pub label: String,
    /// The secret or URL value itself.
    pub value: String,
    /// Optional helper text shown below the value box.
    pub helper: Option<String>,
    /// Aria label for the copy button.
    pub copy_aria: String,
    /// Relative expiry string, e.g. "in 1 hour".
    pub expires_relative: Option<String>,
    /// Absolute expiry string, e.g. "2026-05-14 17:22 UTC".
    pub expires_absolute: Option<String>,
    /// Pre-rendered inline SVG for the QR code, or `None` if not applicable.
    pub qr_svg: Option<String>,
    /// Pre-rendered HTML for any action buttons shown below the value, optional.
    pub action_html: Option<String>,
}

// ── Delete footer fragment ────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/_delete_footer.html")]
pub struct DeleteFooter {
    /// Full relative URL for the confirm action, e.g. `"/people/alice@example.com/delete"`.
    pub action_url: String,
    /// Label on the confirm button, e.g. `"Delete person"`, `"Delete group"`.
    pub confirm_label: String,
    /// Must match the `input_id` used in `DestructiveConfirm` so the JS wiring
    /// connects the input to the right submit button.
    pub input_id: String,
}
