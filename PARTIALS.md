# Partials & macros — reference

## Askama macros

Pure-template; no Rust struct. Import at the top of any template that uses them.

| File | Import alias | Macros |
|---|---|---|
| `templates/macros/ui.html` | `ui` | `button`, `card_header`, `form_error_banner`, `status_badge`, `identity_cell`, `empty_row`, `modal_footer` |
| `templates/macros/forms.html` | `forms` | `email_rows_field` |
| `templates/macros/page.html` | `page` | `tabs_nav`, `list_page_header` |

Each macro's signature and arg meanings are in a comment block immediately above
the `{% macro %}` line — read the source.

## Struct-backed partials

Kept as Rust structs when there's a real invariant (escaping, formatting,
branching).

| Struct | Template | Why |
|---|---|---|
| `Modal` | `templates/partials/_modal.html` | Shared modal frame; multiple modes |
| `DestructiveConfirm` | `templates/partials/_destructive_confirm.html` | Type-to-confirm token + consequences list |
| `DeleteFooter` | `templates/partials/_delete_footer.html` | `with_hx_vals()` escapes apostrophes safely |
| `FormField` | `templates/people/_form_field.html` | Multiline / suffix / error branches |
| `OneTimeSecret` | `templates/partials/_one_time_secret.html` | Owns expiry + QR + optional action row |
| `IdentityRow` | `templates/partials/_identity_row.html` | Used inside destructive-confirm body |
| `Pagination` | `templates/partials/_pagination.html` (+ `_pagination_oob.html`) | Windowed numbering math |
| `TabsNavFragment` | `templates/<section>/_tabs_nav.html` (people/groups/oauth2) | Wraps `page::tabs_nav` for HTMX OOB swaps |

## Behaviors

Delegated DOM enhancement. See `islands/behaviors/README.md` for the contract
and the list of registered behaviors.

## When to extract something new

Three or more identical-shape markup chunks across templates → extract as an
Askama macro (default) or, if invariants demand it, a struct-backed partial.
Don't pre-extract for two call-sites or for hypothetical reuse.
