# 00 — App Shell (Global Layout)

The persistent chrome of every authenticated page: sidebar, topbar, breadcrumb, command palette, user menu, toasts. This file describes the shell only; specific pages render inside the main content area.

## Purpose

Give the admin a consistent navigation frame they never have to relearn. Make the four core sections (Dashboard, People, Groups, OAuth2 Apps) one click away. Surface global utilities (search, theme, sign out) without crowding. Provide a fast keyboard escape hatch (Cmd+K) for power users.

## Layout regions

```
┌─────────────────────────────────────────────────────────────────────┐
│ Sidebar │ Topbar (breadcrumb · search hint · user menu)            │
│         ├──────────────────────────────────────────────────────────┤
│   Logo  │                                                          │
│  ─────  │                                                          │
│ Dashbrd │           Main content area                              │
│ People  │           (page-specific)                                │
│ Groups  │                                                          │
│ OAuth2  │                                                          │
│         │                                                          │
│         │                                                          │
└─────────┴──────────────────────────────────────────────────────────┘
                                                            Toasts ▲
```

## Sidebar

Vertical, persistent on desktop, collapsible to icon-only or a drawer on mobile/narrow viewports.

**Top:** "Kanidm" wordmark in the system's primary type. On hover, optional version chip showing the kanidm server version (e.g., "v1.6.0") in a small subdued pill.

**Navigation items, in order:**
1. **Dashboard** — Lucide `LayoutDashboard` icon, label "Dashboard"
2. **People** — Lucide `Users` icon, label "People"
3. **Groups** — Lucide `UsersRound` icon, label "Groups"
4. **OAuth2 Apps** — Lucide `Shield` icon, label "OAuth2 Apps"

No section headers between them; only four items. Active item is highlighted per the design system's rules.

**Bottom of sidebar:** an "External docs" link (Lucide `BookOpen` icon) to https://kanidm.github.io/kanidm/stable/ — opens in a new tab. Below it, a theme toggle (sun / moon / system).

**Collapse:** at the bottom, a collapse/expand chevron. Collapsed state shows icons only with hover tooltips.

## Topbar

Horizontal bar above the main content area. Three regions left-to-right.

**Left region (breadcrumb):**
Breadcrumb of the current location. Examples:
- `Dashboard` (top-level)
- `People`
- `People > Alice Smith`
- `OAuth2 Apps > grafana > Scope maps`

Each segment is a link except the last. Truncate middle segments with `…` if the full breadcrumb won't fit.

**Center region (global search hint):**
A subtle "Search anything…" placeholder area with a keyboard shortcut hint badge `⌘K` / `Ctrl K` on the right. Clicking it opens the command palette. Width is moderate (~360px). The actual search happens in the command palette (see screen `95`).

**Right region (user menu):**
- Active session privilege indicator: small dot. Solid filled `--success` dot when privileged session is active; outline-only when only read-write is active. Tooltip on hover: "Privileged session active until 14:32" or "Read-write only — re-authenticate for privileged operations".
- Avatar of the signed-in user (initials in a small circle, e.g., "AS" for `admin`'s "Administrator System").
- Display name + chevron — clicking opens a dropdown:
  - User identity header: `admin` (display name) and `admin@idm.example.com` (SPN) in the dropdown header
  - **View profile** → navigates to `/self` (screen `80`)
  - **My sessions** → navigates to `/sessions` (screen `81`)
  - **Re-authenticate** (only shown when privilege session is not active) → opens reauth modal (screen `08`)
  - Divider
  - **Theme**: submenu (Light / Dark / System) — alternative to sidebar-bottom toggle
  - Divider
  - **Sign out** → calls `GET /v1/logout`, redirects to login

## Main content area

The variable region. Each screen brief in this folder describes what goes inside.

- Max-width: 1400px (centered when wider viewport)
- Padding follows the design system's page-padding spec
- The shell does NOT render the page title; the page itself does

## Toast region

Top-right, anchored to the viewport (not the content area). Behavior described in `screens/91-error-states.md` and the design system's toast spec. Toasts stack downward, newest at top, max 5 visible.

## Command palette (Cmd+K)

Opened by Cmd+K / Ctrl+K from anywhere, or by clicking the topbar search hint. Modal-like overlay. Full description in `screens/95-search-and-filter.md`.

Behavior summary:
- Fuzzy search across all entity types: people, groups, OAuth2 apps, plus shortcut destinations like "Settings", "Sign out"
- Results grouped by entity type
- Arrow keys to navigate, Enter to select, Esc to close
- Mouse hover changes selection too
- Recent / suggested entries shown when query is empty

## Re-authentication modal

When the user attempts a privileged action (mutating data, viewing a secret, etc.) and their privilege session has expired or never existed, a reauth modal appears. Full description in screen `08`.

## Sample data references

- Signed-in user in all shell examples: `admin` with display name "System Administrator", SPN `admin@idm.example.com`, member of `idm_admins` and `system_admins`.
- Privileged session expires "in 38 minutes" (so the badge tooltip can show that).
- Server version chip: "v1.6.0".

## States

- **Loading (first auth on shell):** show a skeleton sidebar and topbar; main area is a centered spinner. Brief — the shell loads fast once the auth token is cached.
- **Network offline:** an inline banner appears at the top of the main content area, before any page content: "You're offline. Reconnecting…" with a refresh icon. The banner is `--warning` toned. When reconnected: banner replaced briefly with success state "Reconnected" that fades out.
- **Re-auth required (privilege check failed mid-action):** see `08`.

## Edge cases

- **Long display name in user menu:** truncate with ellipsis after ~14 characters; full name shown in dropdown header.
- **Multiple instances:** out of scope — this UI is single-instance. (If a future feature adds multi-instance switching, it goes in the topbar near the user menu.)
- **Read-only user (no admin privileges):** People / Groups / OAuth2 items still appear in sidebar, but pages within will render read-only (no edit/delete actions). The shell itself doesn't hide anything; downstream pages handle it.

## Sample mockup elements to render

A wireframe-level mockup of the shell should show:
- The sidebar with all four items, "Dashboard" highlighted as active
- Topbar with breadcrumb "Dashboard", search-hint `⌘K`, user menu showing "admin" with avatar and a small green privilege dot
- Main content area empty (or with a placeholder note "Page content here")
- An example toast pinned top-right: success-toast "Reset link generated for alice.smith"
