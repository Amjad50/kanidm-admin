# 40 — Groups: List View

The list of all groups. Entry point for group management.

## Purpose

Quickly find any group, see member counts and policy status at a glance, jump into a group's detail page, or create a new group.

## Layout

Inside the app shell. Main content area:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Groups                                              [+ Create group]│
│ 18 groups                                                           │
│                                                                     │
│ ┌──────────────────────┐ ┌──────────────────────┐                   │
│ │ 🔍 Search groups…    │ │ Filter: All         ▾│                   │
│ └──────────────────────┘ └──────────────────────┘                   │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ ☐ │ Group              │ Description       │ Members │ Policy │ ⋮│ │
│ │───┼────────────────────┼───────────────────┼─────────┼────────┼──│ │
│ │ ☐ │ idm_admins         │ Identity manag…   │ 2       │ ●      │ ⋮│ │
│ │ ☐ │ developers         │ Software develo…  │ 24      │ ●      │ ⋮│ │
│ │ ☐ │ devops             │ Infrastructure…   │ 6       │ ●      │ ⋮│ │
│ │ ☐ │ vpn_users          │ Granted WireGu…   │ 31      │ ○      │ ⋮│ │
│ │ ☐ │ on_call            │ Engineers curr…   │ 4       │ ●      │ ⋮│ │
│ │ ☐ │ system_admins      │ Root-level kan…   │ 1       │ ●      │ ⋮│ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ Showing 1–6 of 18                                  ‹ 1 2 3 ›        │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- Title: "Groups"
- Subtitle: "{N} groups"
- Right: "+ Create group" primary button → screen 41

## Controls row

- Search input: "Search by group name…" debounced 300ms
- Filter dropdown:
  - All (default)
  - Has account policy (only groups with policy enabled)
  - No account policy
  - Has members
  - Empty (no members)
  - Built-in (kanidm-system groups like `idm_admins`)
  - User-created

Multi-select with chip display.

## Table columns

1. **Checkbox** (40px)
2. **Group** (25% width) — group name in primary/monospace, optionally with a small "Built-in" badge if kanidm-managed (e.g., `idm_admins`)
3. **Description** (30% width) — truncated to ~60 chars with ellipsis; full description in tooltip on hover. "—" subdued if not set.
4. **Members** (10% width) — count with people icon
5. **Account policy** (10% width) — filled dot (●) if enabled, outline (○) if not. Color: accent if enabled, subdued if not. Tooltip: "Account policy enabled" / "No account policy"
6. **Actions** (40px) — kebab menu:
   - View details
   - Edit
   - Manage members
   - Manage account policy
   - Divider
   - Delete (red)

Row click navigates to group detail (screen 42).

## Bulk actions

When ≥1 row selected, bulk action bar appears with:
- "{N} selected" + Clear
- "Delete selected" (danger) — opens confirm with all SPNs/names listed

## States

- **Loading:** skeleton table.
- **Empty (no groups in system — extremely rare since kanidm ships built-in groups):** illustration + "No groups yet" + "+ Create group" CTA.
- **Empty after search:** "No groups match '{query}'" + Clear search.
- **Error:** "Could not load groups." + Retry.

## Pagination

Bottom: "Showing 1–25 of 18" (since 18 < 25, no page navigator needed; show only the count). For larger instances: "Showing 1–25 of 84" with page nav.

Page size selector: 25 / 50 / 100.

## Sample data

Use all six sample groups from `_sample-data.md`:

| Group | Description | Members | Policy |
|---|---|---|---|
| `idm_admins` | Identity management administrators with full system access | 2 | ● enabled |
| `developers` | Software development team — code repository and dev OAuth2 access | 24 | ● enabled |
| `devops` | Infrastructure and platform operations | 6 | ● enabled |
| `vpn_users` | Granted WireGuard / OpenVPN remote access | 31 | ○ disabled |
| `on_call` | Engineers currently in the on-call rotation | 4 | ● enabled |
| `system_admins` | Root-level kanidm administrators | 1 | ● enabled |

Subtitle: "18 groups"
Pagination: "Showing 1–6 of 18"

## Edge cases

- **Group with no description:** show "—" in subdued color.
- **Group with 0 members:** still show "0" in the count, possibly with a subtle warning dot if it looks like an orphaned group (member count is 0 AND last-modified is >90 days ago).
- **Built-in groups (kanidm system groups):** badge "Built-in" next to name. These typically can't be deleted; delete option in kebab is disabled with tooltip "Built-in groups can't be deleted."
- **Very long description:** truncate at ~60 chars with ellipsis. Tooltip on hover shows the full description.

## Keyboard

- `/` focuses search
- `↑ ↓` navigates rows
- `Enter` opens detail
- `Space` toggles checkbox
- `Cmd+A` selects all on page

## Mockup elements to render

- Page header with title + subtitle + Create group button
- Search + filter row
- Table with all 6 sample groups
- Built-in badge on `idm_admins` and `system_admins`
- Policy column with filled dots on policy-enabled groups, outline dot on `vpn_users`
- One row hovered to show hover state
- Pagination
- Render a second variant with 2 rows selected and the bulk action bar at the bottom
