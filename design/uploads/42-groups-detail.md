# 42 — Groups: Detail View

The detail page for a single group. Tabbed layout: Overview, Members, Account Policy.

## Purpose

Show everything administered about a group: identity, members, account policy. Provide entry points for editing each.

## Layout

Inside the app shell. Main content area:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Groups > developers                                                 │
│                                                                     │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ developers                                                   │    │
│ │ developers@idm.example.com                                   │    │
│ │ Software development team — code repository and dev OAuth2 …│    │
│ │                                                              │    │
│ │   24 members · Account policy ●                              │    │
│ │                                                              │    │
│ │   [Edit] [Manage members] [⋯]                                │    │
│ └──────────────────────────────────────────────────────────────┘    │
│                                                                     │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ Overview │ Members (24) │ Account policy                     │    │
│ ├──────────────────────────────────────────────────────────────┤    │
│ │                                                              │    │
│ │  (Tab content here, e.g., Overview)                          │    │
│ │                                                              │    │
│ └──────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- Breadcrumb: `Groups > developers` (last segment is the group name)

## Identity card (top)

- Group icon (Lucide `UsersRound` or similar) in a circular soft-background container
- Group name (large, primary, optional monospace): "developers"
- SPN (monospace, secondary): "developers@idm.example.com"
- Description (subdued, italic-allowed): "Software development team — code repository and dev OAuth2 access"
- Quick stats line: "24 members · Account policy ●" (filled dot if enabled, outline if not, color per design)
- Action buttons:
  - Edit (primary or secondary) → screen 43
  - Manage members (navigates to Members tab) → screen 44
  - ⋯ kebab: Account policy (jumps to that tab), Copy UUID, Copy SPN, Delete (red)

## Tabs

1. **Overview** (default)
2. **Members ({count})** — count in label
3. **Account policy** — see screen 45

URL preserves tab: `/groups/developers?tab=members`. Browser back/forward navigates between tabs.

## Overview tab content

```
┌────────────────────────────────────────────────────────────────┐
│ Identity                                                       │
│                                                                │
│ UUID            <uuid>                              [copy]     │
│ Group name      developers                                     │
│ SPN             developers@idm.example.com         [copy]      │
│ Description     Software development team — code repository … │
│ Mail            dev@example.com                                │
│ Entry managed   idm_admins                                     │
│                 by                                             │
│                                                                │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ Members (24)                            → Manage members tab   │
│                                                                │
│ Showing a preview of the first 8 members:                      │
│                                                                │
│  ⓐ alice.smith    ⓑ bob.jones    ⓒ carol.nguyen                │
│  ⓔ eve.taylor     ⓕ frank.future ⓓ dave.locked                 │
│  +18 more                                                      │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ Account policy                                                 │
│                                                                │
│ ● Enabled                                                      │
│ Credential type minimum  Passkey                              │
│ Password minimum length  16                                    │
│ Session expiry           3600 seconds (1 hour)                 │
│ Privilege expiry         900 seconds (15 minutes)              │
│                                                                │
│ → Configure on Account policy tab                              │
└────────────────────────────────────────────────────────────────┘
```

### Identity section

Key-value list:
- UUID (monospace, copy button)
- Group name
- SPN (monospace, copy)
- Description (clamped to ~2 lines with expand link if longer)
- Mail (the group's mail attribute; "—" subdued if not set)
- Entry managed by — link to that group's detail page (e.g., `idm_admins`)

### Members preview section

A small grid showing the first 8-12 members with their avatars + usernames. If there are more, show "+N more" link → Members tab.

If no members: "No members yet. → Add members on the Members tab."

### Account policy summary section

If account policy is enabled:
- ● Enabled (success-colored dot)
- A small key-value list with the configured policy values:
  - Credential type minimum
  - Password minimum length
  - Session expiry (in seconds + human-readable conversion)
  - Privilege expiry
  - (Other configured values, if any)
- Link "→ Configure on Account policy tab"

If account policy is NOT enabled:
- ○ Disabled (subdued dot)
- "No account policy is configured for this group."
- Link "→ Enable on Account policy tab"

## States

- **Loading:** skeleton.
- **Group not found:** 404-style empty state.
- **Read-only viewer (no edit privileges):** action buttons disabled.
- **Built-in group (e.g., `idm_admins`):** delete option in kebab is disabled with tooltip "Built-in groups can't be deleted." Detected via `class` containing `"builtin"` or `"system"`.
- **Dynamic group (e.g., `idm_all_persons`):** detected via `class` containing `"dyngroup"`. The Members tab is read-only — members are computed by kanidm via a filter expression, not assigned manually. Show a banner on the Members tab: "This is a dynamic group. Members are computed automatically based on kanidm's internal filter and cannot be edited here." Disable add/remove/purge actions.

## Sample data

Use `developers` from `_sample-data.md`:
- UUID (placeholder — kanidm assigns; use a sample): `f7e2a8d4-3c1b-4e9f-a6c2-5d8b1f3a7e9c`
- Group name: `developers`
- SPN: `developers@idm.example.com`
- Description: "Software development team — code repository and dev OAuth2 access"
- Mail: `dev@example.com`
- Entry managed by: `idm_admins`
- Member count: 24
- Member preview names: `alice.smith`, `bob.jones`, `carol.nguyen`, `eve.taylor`, `frank.future`, `dave.locked` + "+18 more"
- Account policy: enabled with developers' values (credential-type-minimum=mfa, password-minimum-length=12, auth-session-expiry=28800, privilege-session-expiry=1800)

For an alternate variant, use `vpn_users` (no account policy enabled):
- Group name: `vpn_users`
- 31 members
- Account policy: ○ Disabled
- "No account policy is configured for this group."

## Edge cases

- **No description:** show "—" or "No description set" in subdued color.
- **No mail:** show "—" or "No mail address set" in subdued.
- **Entry-managed-by is the group itself (self-managed, edge case):** show "(self-managed)" badge.
- **Member count mismatch (cache drift):** the count in the badge might briefly differ from the listed members count after a recent add/remove. Refresh on tab switch.
- **Group has 0 members:** Members preview section shows "No members yet" + link to add on Members tab.

## Mockup elements to render

- Breadcrumb
- Identity card for `developers` with all fields populated (24 members, account policy ●, action buttons)
- Tab bar with Overview, Members (24), Account policy — Overview active
- Identity section with all key-values
- Members preview with 6 avatar chips + "+18 more"
- Account policy summary section showing enabled state with key policy values
- Render a second variant: `vpn_users` (account policy disabled state)
