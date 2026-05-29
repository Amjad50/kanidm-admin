# 20 — People: List View

The list of all person accounts. The primary entry point for most admin tasks involving users: searching, filtering, opening details, performing bulk actions.

## Purpose

Let the admin find any person quickly via search or filter, see their status at a glance, and either jump into a person's detail page or perform a bulk action.

## Layout

Inside the app shell. Main content area:

```
┌─────────────────────────────────────────────────────────────────────┐
│ People                                              [+ Create person]│
│ 127 people                                                          │
│                                                                     │
│ ┌──────────────────────┐ ┌─────────────┐ ┌──────────────┐           │
│ │ 🔍 Search people…    │ │ Status: All │ │ Sort: Name ▾ │           │
│ └──────────────────────┘ └─────────────┘ └──────────────┘           │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ ☐ │ Person                  │ Email             │ Status  │ ⋮  │ │
│ │───┼─────────────────────────┼───────────────────┼─────────┼────│ │
│ │ ☐ │ ⓐ alice.smith           │ alice@example.com │ Active  │ ⋮  │ │
│ │   │   Alice Smith           │                   │         │    │ │
│ │ ☐ │ ⓑ bob.jones             │ bob@example.com   │ Active  │ ⋮  │ │
│ │   │   Bob Jones             │                   │         │    │ │
│ │ ☐ │ ⓒ carol.nguyen          │ carol@example.com │ Active  │ ⋮  │ │
│ │   │   Carol Nguyen          │                   │         │    │ │
│ │ ☐ │ ⓓ dave.locked           │ dave@example.com  │ Expired │ ⋮  │ │
│ │   │   Dave Locked           │                   │         │    │ │
│ │ ☐ │ ⓔ eve.taylor            │ eve@example.com   │ Active  │ ⋮  │ │
│ │   │   Eve Taylor            │                   │         │    │ │
│ │ ☐ │ ⓕ frank.future          │ frank@example.com │ Not yet │ ⋮  │ │
│ │   │   Frank Future          │                   │         │    │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ Showing 1–6 of 127                            ‹ 1 2 3 ··· 22 ›      │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- Title: "People"
- Subtitle: "{N} people" where N is the total count (refreshes on filter)
- Right side: primary button "+ Create person" — opens screen 21

## Controls row

Three controls:

1. **Search input:** Placeholder "Search by name, SPN, or email…". Debounced (300ms). Searches across name, SPN, displayname, mail. Calls kanidm's SCIM search endpoint. Clear button (×) appears when input has value. Keyboard shortcut `/` focuses search.

2. **Status filter dropdown:** Options:
   - All (default)
   - Active
   - Expired (`expire_at` in the past)
   - Not yet active (`valid_from` in the future)
   - Without credentials

   Multi-select chip-style — admin can pick multiple. Selected filters appear as removable chips below the row.

3. **Sort dropdown:** Options:
   - Name (default)
   - Display name
   - Created (newest first)
   - Modified (most recent first)
   - SPN

   Affects table sort order.

## Table

Columns left-to-right:

1. **Checkbox** (40px) — bulk selection. Header has a "select all on this page" checkbox.
2. **Person** (40% width) — avatar (32-36px circle with initials) + stack of (display name in primary text + SPN in secondary text monospace). Or single-line layout per design system density.
3. **Email** (25% width) — primary email address. Show subdued "—" if not set.
4. **Status** (12% width) — badge with semantic color:
   - "Active" → success
   - "Expired" → danger
   - "Not yet active" → warning
   - "No credentials" → warning
   - "Suspended" → danger (kanidm has soft-locked accounts)
5. **Actions menu** (40px) — kebab (vertical dots) button. Opens dropdown with row actions.

### Row actions (kebab menu)

- View details (default action; also triggered by clicking anywhere on the row except checkbox/email)
- Edit
- Generate reset link
- Manage SSH keys
- Set validity
- Destroy session(s)
- Divider
- Delete (red, destructive — opens confirm modal from screen 29)

### Bulk actions bar (appears when ≥1 row is selected)

Fixed at the bottom of the content area, slides up. Shows:
- "{N} selected" + "Clear selection"
- Action buttons: "Generate reset links", "Set validity (bulk)", "Delete selected"

For bulk delete, the confirm modal lists all selected SPNs (with "and N more…" if many) and uses type-to-confirm pattern from screen 93.

## States

- **Loading:** skeleton table with 6-8 skeleton rows. Headers visible. Search and filters are interactive.
- **Empty (no people in system):** centered illustration (per `90-empty-states.md`) + heading "No people yet" + body "Create your first person to get started." + primary button "+ Create person".
- **Empty after search:** "No people match '{query}'" with a "Clear search" button. Subdued tone.
- **Empty after filter:** "No people match the current filters" with "Clear filters" button.
- **Error:** "Could not load people. Retry." with a retry button. Inline at the top of the table.
- **Slow connection:** if a search takes >2s, show a small spinner in the search input.

## Pagination

Bottom of table:
- Left: "Showing 1–25 of 127" (the page range and total)
- Right: page navigator — `‹ 1 2 3 … 22 ›` with current page highlighted
- Page size selector (small): "25 per page ▾" with options 10 / 25 / 50 / 100

Default page size: 25.

## Sample data

Use exactly these names from `_sample-data.md`:
- `alice.smith` (Alice Smith) — alice@example.com — Active
- `bob.jones` (Bob Jones) — bob@example.com — Active
- `carol.nguyen` (Carol Nguyen) — carol@example.com — Active
- `dave.locked` (Dave Locked) — dave@example.com — Expired
- `eve.taylor` (Eve Taylor) — eve@example.com — Active
- `frank.future` (Frank Future) — frank@example.com — Not yet active

Subtitle count: "127 people"
Pagination shows: "Showing 1–6 of 127" with pages "1 2 3 … 22"

## Edge cases

- **Person with no mail:** show "—" in the Email column (subdued text).
- **Person with multiple emails:** show only the primary (first in list) with a small "+2" badge indicating additional addresses (hover tooltip lists them all).
- **Very long display name:** truncate with ellipsis after ~30 characters.
- **Person with empty display name:** show SPN as primary text and "No display name set" as secondary in subdued style.
- **Filter combinations that yield zero results:** show "No people match the current filters" message.

## Keyboard

- `/` focuses the search input
- `↑ / ↓` navigates rows (when search is not focused)
- `Enter` on a focused row opens detail view
- `Space` toggles row checkbox
- `Cmd+A` / `Ctrl+A` selects all on page (only when search is not focused — escape to release search first)

## Mockup elements to render

- Page title "People" + count subtitle "127 people"
- "+ Create person" primary button top-right
- Search input + status filter + sort dropdown row
- Table with all 6 sample people from `_sample-data.md`
- Row with `dave.locked` showing "Expired" badge (in danger color)
- Row with `frank.future` showing "Not yet active" badge (in warning color)
- One row hovered to show hover state
- Pagination at bottom: "Showing 1–6 of 127" + page navigator
- Render a second variant with one row selected (checkbox checked) to show the bulk actions bar appearing at the bottom
- Render the empty state separately (no people)
