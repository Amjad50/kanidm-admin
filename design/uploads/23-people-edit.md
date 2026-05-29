# 23 — People: Edit Person

The edit form for a person's basic attributes: username (rename), display name, legal name, mail addresses.

## Purpose

Allow editing of attributes that are mutable via `PATCH /v1/person/{id}` or `kanidm person update`. Provide a clear way to manage the mail list (add, reorder, remove, set primary). Warn the user when renaming the username since it changes the SPN.

## Layout

Inline within the person detail page (replacing the Overview tab) OR a separate page at `/people/alice.smith/edit` — designer's call. Modal is also acceptable for compact density (Linear variant). For the Cloudflare and Stripe variants, an inline / dedicated-page layout reads better.

```
┌─────────────────────────────────────────────────────────────────────┐
│ People > alice.smith > Edit                                         │
│                                                                     │
│ Edit Alice Smith                                                    │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Username                                                        │ │
│ │ ┌───────────────────────┐ @idm.example.com                      │ │
│ │ │ alice.smith           │                                       │ │
│ │ └───────────────────────┘                                       │ │
│ │ ⚠ Renaming changes the SPN. Update any external systems that    │ │
│ │   reference alice.smith.                                        │ │
│ │                                                                 │ │
│ │ Display name                                                    │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Alice Smith                                                 │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │                                                                 │ │
│ │ Legal name                                                      │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Alice Marion Smith                                          │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ Used in reports and audit logs.                                 │ │
│ │                                                                 │ │
│ │ Emails                                                          │ │
│ │ ┌──────────────────────────────────────────────────┬─────────┐ │ │
│ │ │ ★ alice.smith@example.com                        │ ⋯  ✕    │ │ │
│ │ │   alice@example.com                              │ ★  ✕    │ │ │
│ │ └──────────────────────────────────────────────────┴─────────┘ │ │
│ │ [+ Add email]                                                   │ │
│ │ The starred email is primary. Drag to reorder.                  │ │
│ │                                                                 │ │
│ │             [Cancel]          [Save changes]                    │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- Breadcrumb: `People > alice.smith > Edit`
- Title: "Edit Alice Smith"

## Form

### Username field

- Label "Username"
- Suffix `@idm.example.com`
- Same character constraints as create (screen 21)
- **Warning callout below** (only shown if user has modified the field): "⚠ Renaming changes the SPN. Update any external systems that reference {original-username}." in `--warning` tone.

### Display name field

- Label "Display name"
- Required (can't be empty)

### Legal name field

- Label "Legal name"
- Optional; helper text "Used in reports and audit logs."
- "Clear" button to remove the value (sets to null)

### Emails list

A reorderable list with primary indicator. Each row:
- Drag handle (left, optional — keyboard support via up/down arrow buttons too)
- Star icon: filled (★) for primary, outline (☆) for others. Click to set as primary.
- Email address (display)
- "×" button to remove

Below the list: "+ Add email" — adds a new row with an inline input. Validation on blur (basic email format).

Helper: "The starred email is primary. Drag to reorder."

If list is empty: a single placeholder row "No emails set" + the add button.

### Footer

- Cancel — discards changes, navigates back to detail page. If unsaved changes exist, show a confirm dialog: "You have unsaved changes. Discard?"
- Save changes — primary. Disabled until at least one field changed. On click:
  - Calls `PATCH /v1/person/{id}` with the changeset
  - On success: toast "Changes saved" + navigate back to detail Overview
  - On error: inline errors or toast per response

## Unsaved changes warning

If the user navigates away (browser back, clicks a sidebar link) with unsaved changes, show a confirm modal: "You have unsaved changes. Discard?" with Cancel / Discard buttons.

## States

- **Idle:** as described.
- **Submitting:** Save button shows spinner; inputs are read-only; Cancel still works.
- **Field-level error:** inline below the offending field in `--danger`.
- **Server error:** toast "Could not save changes. Try again."

## Sample data

Use Alice Smith from `_sample-data.md`:
- Username: `alice.smith`
- Display name: Alice Smith
- Legal name: Alice Marion Smith
- Emails: `alice.smith@example.com` (primary), `alice@example.com`

For the rename-warning state, show username field with `alice.smith2` typed and the warning callout visible.

## Edge cases

- **Same value as original:** Save is disabled (no-op).
- **Removing the primary email:** require user to set another as primary first, OR auto-promote the next email in the list. The latter is simpler — auto-promote and show a brief toast on save: "alice@example.com is now the primary email."
- **Empty email list:** allowed. The user has no email — primary email field becomes "—" on the detail page.
- **Username rename collision:** server returns 409. Show inline error on username field: "A person with this username already exists."
- **Username rename for `admin` account:** kanidm may forbid this for built-in admin. If server rejects, show the error message verbatim.

## Mockup elements to render

- Breadcrumb
- Title "Edit Alice Smith"
- Form card with all fields populated with Alice's data
- Email list with star (★) on `alice.smith@example.com`, drag handles visible
- "+ Add email" button below list
- "Save changes" button (primary, enabled)
- Render a variant showing the rename warning: username field changed to `alice.smith2`, warning callout below
