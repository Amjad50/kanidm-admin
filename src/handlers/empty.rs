/// GET /empty — returns 200 with an empty body.
///
/// Used by every modal's Cancel button and backdrop-click to clear
/// `#overlay-slot` via HTMX innerHTML swap.
pub async fn empty() -> &'static str {
    ""
}
