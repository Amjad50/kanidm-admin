# 44 — Groups: Members Tab

The Members tab on the group detail page. Lists current members, allows adding/removing members, and supports bulk operations.

## Purpose

Manage group membership: add, remove, replace all (set), purge all. The most common day-to-day group admin operation.

## Layout

Tab content inside the group detail page:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Members                                                             │
│                                                                     │
│ 24 members in developers                          [+ Add members]   │
│                                                                     │
│ ┌──────────────────────┐ ┌────────────────────────────────────────┐ │
│ │ 🔍 Filter members…   │ │ [Replace all members] [Purge all]      │ │
│ └──────────────────────┘ └────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ ☐ │ Member                                          Actions     │ │
│ │───┼─────────────────────────────────────────────────────────────│ │
│ │ ☐ │ ⓐ alice.smith     Alice Smith                    Remove    │ │
│ │ ☐ │ ⓑ bob.jones       Bob Jones                      Remove    │ │
│ │ ☐ │ ⓒ carol.nguyen    Carol Nguyen                   Remove    │ │
│ │ ☐ │ ⓔ eve.taylor      Eve Taylor                     Remove    │ │
│ │ ☐ │ … (20 more)                                                 │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ Showing 1–4 of 24                                  ‹ 1 2 3 4 5 6 ›  │
└─────────────────────────────────────────────────────────────────────┘
```

## Tab content

### Header row

- "{N} members in {group_name}" subtitle
- Right: "+ Add members" primary button → opens add-members modal/flow (see below)

### Controls row

- Filter input: "Filter members…" — filters the current list (client-side; the list could be large for groups like `vpn_users` so server-side filter is also acceptable). Lucide `Search` icon.
- Right: two danger-secondary buttons:
  - "Replace all members" — opens a flow to wholesale-replace membership (uses `kanidm group set-members` semantically)
  - "Purge all" — danger; opens confirm to remove all members at once

### Members table

Columns:

1. **Checkbox** (40px) — for bulk operations
2. **Member** — avatar (24-32px) + stack of name + display name (e.g., "alice.smith" / "Alice Smith"). Also indicates type (person vs service-account) with a small icon if relevant — but since service accounts are out of scope for the UI, all members shown are people (kanidm allows group members to be people, service accounts, or other groups for nested membership; for the UI scope here, assume mostly people; render nested groups with a `Users` icon and a "Group" label if present).
3. **Actions** — "Remove" button. On click: small confirm: "Remove alice.smith from developers?" with Cancel / Remove.

Row click navigates to the member's detail page.

### Bulk remove

When ≥1 row selected, bulk action bar appears: "{N} selected · Remove selected".

### Add members modal/flow

Opens from "+ Add members" button:

```
   ┌────────────────────────────────────────────────────┐
   │  Add members to developers                  [×]    │
   ├────────────────────────────────────────────────────┤
   │                                                    │
   │  Search                                            │
   │  ┌────────────────────────────────────────────┐    │
   │  │ 🔍 Search people…                          │    │
   │  └────────────────────────────────────────────┘    │
   │                                                    │
   │  Available (showing 10 of 100+):                  │
   │  ┌────────────────────────────────────────────┐    │
   │  │ ☐ ⓙ jane.doe     Jane Doe                  │    │
   │  │ ☐ ⓟ paul.kim     Paul Kim                  │    │
   │  │ ☑ ⓡ rita.shah    Rita Shah                 │    │
   │  │ ☑ ⓢ sam.lopez    Sam Lopez                 │    │
   │  │ ☐ ⓣ thomas.li    Thomas Li                 │    │
   │  └────────────────────────────────────────────┘    │
   │                                                    │
   │  2 selected                                        │
   │                                                    │
   ├────────────────────────────────────────────────────┤
   │              [Cancel]       [Add 2 members]        │
   └────────────────────────────────────────────────────┘
```

Modal contents:
- Search input at top — typeahead querying people (and groups, if nested membership is allowed)
- Available list — shows non-members. Checkbox to select. As-you-type filter.
- Selected count at the bottom
- Footer: Cancel + "Add {N} member{s}" (singular/plural)
- On submit: `POST /v1/group/{name}/_attr/member` for each selected. Show success toast: "Added 2 members to developers."

### Replace all members flow

Opens a confirm modal first:

```
   ┌────────────────────────────────────────────────────┐
   │  Replace all members                         [×]   │
   ├────────────────────────────────────────────────────┤
   │                                                    │
   │  ⚠ This will replace the current 24 members with   │
   │  a new list. The current members will be removed   │
   │  unless they're in the new list.                   │
   │                                                    │
   │  Pick the new members below. Use the regular Add   │
   │  flow if you want to add without replacing.        │
   │                                                    │
   │  [ Continue to picker ]                            │
   ├────────────────────────────────────────────────────┤
   │              [Cancel]                              │
   └────────────────────────────────────────────────────┘
```

Continuing opens the same picker UI as Add, but with all current members pre-selected. Saving calls `PUT /v1/group/{name}/_attr/member` to wholesale-replace.

### Purge all flow

Confirm modal:

```
   ┌────────────────────────────────────────────────────┐
   │  Purge all members                           [×]   │
   ├────────────────────────────────────────────────────┤
   │                                                    │
   │  ⚠ Remove all 24 members from developers?          │
   │                                                    │
   │  The group will be empty afterwards. Members are   │
   │  not deleted from kanidm — only removed from this  │
   │  group.                                            │
   │                                                    │
   │  Type the group name to confirm:                   │
   │  developers                                        │
   │  ┌────────────────────────────────────────────┐    │
   │  │                                            │    │
   │  └────────────────────────────────────────────┘    │
   │                                                    │
   ├────────────────────────────────────────────────────┤
   │              [Cancel]       [Purge all members]    │
   └────────────────────────────────────────────────────┘
```

Type-to-confirm pattern. On confirm: `PUT /v1/group/{name}/_attr/member` with empty list (or DELETE).

## States

- **Loading:** skeleton table.
- **Empty group:** "No members yet" + "+ Add members" primary CTA.
- **Filter no matches:** "No members match '{query}'" + Clear filter.
- **Adding/removing:** row shows loading state, then updates.
- **Privilege required:** add/remove actions require privilege session — reauth modal triggered.
- **Dynamic group (`class: dyngroup`):** members are read-only. Source is the `dynmember` attr (computed by kanidm), not the `member` attr. Show a banner: "This is a dynamic group. Membership is computed automatically." Hide add/remove/purge buttons. The table still displays the computed members for visibility.

## Sample data

For `developers` (24 members) — show first 4 + indicator for remaining:
- `alice.smith` / Alice Smith
- `bob.jones` / Bob Jones
- `carol.nguyen` / Carol Nguyen
- `eve.taylor` / Eve Taylor

For the Add members modal, show non-member sample names (fabricated for variety):
- `jane.doe` / Jane Doe
- `paul.kim` / Paul Kim
- `rita.shah` / Rita Shah
- `sam.lopez` / Sam Lopez
- `thomas.li` / Thomas Li

## Edge cases

- **Adding someone already in the group:** the picker hides existing members. If kanidm rejects (race condition), show toast "alice.smith is already a member."
- **Removing the only member of `idm_admins`:** warning callout: "⚠ Removing all members of idm_admins will leave kanidm with no administrators. Continue only if you have another admin path configured."
- **Nested groups:** kanidm supports nested membership (group A is a member of group B). Show these with a `Users` icon and the label "(group)" — clicking opens that group's detail.
- **Service account members:** out of scope for this UI, but if present in the data, render them with a `Bot` icon and a "(service account)" label.
- **Very large groups (1000+ members):** paginate the table. Filter does server-side query.

## Mockup elements to render

- Tab content with "Members" heading
- Subtitle "24 members in developers" + Add members button
- Filter input + Replace all / Purge all buttons
- Members table with first 4 sample members
- One row hovered to show Remove button highlighted
- Pagination
- Render the Add members modal as a separate mockup with sample non-member candidates, 2 selected (Rita Shah, Sam Lopez), "Add 2 members" button
- Render the Purge all confirm modal as a third variant
