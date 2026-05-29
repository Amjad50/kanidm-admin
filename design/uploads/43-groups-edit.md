# 43 — Groups: Edit Group

Edit a group's attributes: rename, description, mail list, entry-managed-by.

## Purpose

Allow editing of mutable group attributes. Warn on rename. Provide list-based editing for mail (add / reorder / remove / set primary, same pattern as person email editing).

## Layout

Inline within group detail (replacing Overview tab) OR a separate page at `/groups/developers/edit`. Designer's call per design system.

```
┌─────────────────────────────────────────────────────────────────────┐
│ Groups > developers > Edit                                          │
│                                                                     │
│ Edit developers                                                     │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Group name                                                      │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ developers                                                  │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ ⚠ Renaming changes the SPN. Any OAuth2 scope maps and external │ │
│ │   references using this group name must be updated.            │ │
│ │                                                                 │ │
│ │ Description                                                     │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Software development team — code repository and dev OAuth2 │ │ │
│ │ │ access                                                     │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │                                                                 │ │
│ │ Mail addresses                                                  │ │
│ │ ┌────────────────────────────────────────────────────────────┐  │ │
│ │ │ ★ dev@example.com                                  ⋯  ✕    │  │ │
│ │ │   developers-list@example.com                      ★  ✕    │  │ │
│ │ └────────────────────────────────────────────────────────────┘  │ │
│ │ [+ Add mail]                                                    │ │
│ │ The starred mail is primary. Drag to reorder.                   │ │
│ │                                                                 │ │
│ │ Entry managed by                                                │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ idm_admins                                          [Change]│ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ The group that can manage this group's attributes and          │ │
│ │ membership.                                                     │ │
│ │                                                                 │ │
│ │              [Cancel]          [Save changes]                   │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- Breadcrumb: `Groups > developers > Edit`
- Title: "Edit developers"

## Form fields

### Group name

- Label "Group name"
- Same validation as create (lowercase letters, digits, underscore)
- **Warning callout below** (only shown if user modified the field): "⚠ Renaming changes the SPN. Any OAuth2 scope maps and external references using this group name must be updated." in `--warning` tone.

### Description

- Label "Description"
- Optional, multi-line textarea (3-4 rows)
- "Clear" button to remove

### Mail addresses

A reorderable list with primary indicator (same pattern as person email editing in screen 23):
- Drag handle (left)
- Star icon: filled for primary, outline for others. Click to set primary.
- Email address
- "×" to remove

Below: "+ Add mail" button. Empty state: placeholder row "No mail addresses set."

Helper: "The starred mail is primary. Drag to reorder."

### Entry managed by

- Label "Entry managed by"
- Display the current value (e.g., `idm_admins`) with a "Change" button on the right
- Clicking "Change" opens a group picker popover (typeahead) where the admin can search and select another group
- "Clear" / "Reset to default" option in the picker (defaults to `idm_admins`)

### Footer

- Cancel — discards changes, navigates back. If unsaved changes, confirm modal.
- Save changes — primary, disabled until any field changes. On click:
  - PATCH `/v1/group/{name}` with the changeset
  - On success: toast "Changes saved" + navigate back to Overview tab
  - On rename collision: inline error on name field
  - On error: toast

## Unsaved changes warning

Same as person edit (screen 23): browser navigation prompts confirm if changes are unsaved.

## States

- **Idle:** as described.
- **Submitting:** Save spinner.
- **Field-level error:** inline.
- **Server error:** toast.

## Sample data

Use `developers` from `_sample-data.md`:
- Name: `developers`
- Description: "Software development team — code repository and dev OAuth2 access"
- Mail: `dev@example.com` (primary), `developers-list@example.com` (secondary, fabricated for the second-mail example)
- Entry managed by: `idm_admins`

## Edge cases

- **Rename built-in group:** kanidm likely forbids renaming `idm_admins` and other system groups. If server rejects, show error inline.
- **Entry-managed-by points to self:** unusual but permitted in kanidm. Allow but show warning.
- **Empty group name:** disabled save until valid.
- **Clearing entry-managed-by:** "Reset to default" sets it back to `idm_admins` (or whatever kanidm defaults to for this group's class).

## Mockup elements to render

- Breadcrumb
- Title "Edit developers"
- Form card with all fields populated from `developers`'s sample data
- Mail list with primary star, drag handles
- Entry-managed-by field showing `idm_admins` + Change button
- Cancel + Save changes buttons
- Render a second variant showing the rename warning state: group name changed to `developers_new`, warning callout below
- Render the entry-managed-by picker popover state as a third mockup variant
