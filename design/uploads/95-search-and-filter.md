# 95 — Search and Filter (Cross-Cutting Pattern)

The universal pattern for in-list search/filter inputs and the global command palette (Cmd+K).

## Purpose

Make finding any entity fast — both via per-list search and via a global "go anywhere" command palette. Keyboard-first.

## Per-list search input

Every list page (People, Groups, OAuth2 Apps, and any sub-list like Group Members) has a search input prominently placed.

**Appearance:**
- Input with Lucide `Search` icon on the left
- Placeholder: contextual, e.g., "Search people by name, SPN, or email…", "Search groups…", "Search apps…"
- Width: ~280-400px depending on design system density
- Right side: clear button (×) when input has value
- Right side (when empty): keyboard shortcut hint badge (small monospace `/`) if `/` focuses search

**Behavior:**
- Debounced 250-350ms (longer for slower backends)
- Calls the kanidm SCIM search endpoint or filters client-side for already-loaded data
- Updates the list below as results arrive (skeleton briefly if delayed)
- Clear button (×) appears on right when input has value, clicking resets and shows the full list
- `/` keyboard shortcut focuses the input (when input isn't already focused; works on any list page)
- `Esc` clears the input when focused

## Per-list filter dropdowns

Filters complement search: search is for "find this thing", filters are for "narrow down this list".

**Appearance:**
- A dropdown button to the right of the search input
- Label shows current filter state: "Status: All ▾" / "Type: Confidential ▾"
- Opens a dropdown menu with options:
  - Checkbox-style for multi-select (most filters)
  - Radio-style for single-select (sort)
- Selected filters appear as small removable chips below the search row: `Status: Active [×] Type: Confidential [×]`

**Behavior:**
- Selection updates the list immediately
- Chips can be removed individually
- "Clear all filters" link if multiple filters are active

## Global command palette (Cmd+K)

Inspired by Linear / Raycast / GitHub. Opened by `Cmd+K` (macOS) or `Ctrl+K` (others). The single most powerful navigation primitive in the app.

**Appearance:**

```
   ┌───────────────────────────────────────────────────────┐
   │ ┌───────────────────────────────────────────────────┐ │
   │ │ 🔍 Search anything…                       [⌘K]    │ │
   │ └───────────────────────────────────────────────────┘ │
   │                                                       │
   │ Recent                                                │
   │  ⓐ Alice Smith              People                    │
   │  📋 grafana                  OAuth2 Apps              │
   │                                                       │
   │ Quick actions                                         │
   │  ＋ Create person                                     │
   │  ＋ Create group                                      │
   │  ＋ Create OAuth2 application                         │
   │                                                       │
   │ Navigate                                              │
   │  ⌂ Dashboard                                          │
   │  👤 People                                            │
   │  👥 Groups                                            │
   │  🛡 OAuth2 Apps                                       │
   │  ⚙ My sessions                                        │
   │  ⏻ Sign out                                           │
   │                                                       │
   │  ↑↓ navigate · ↵ select · esc close                   │
   └───────────────────────────────────────────────────────┘
```

When the admin types:

```
   ┌───────────────────────────────────────────────────────┐
   │ ┌───────────────────────────────────────────────────┐ │
   │ │ 🔍 graf                                           │ │
   │ └───────────────────────────────────────────────────┘ │
   │                                                       │
   │ OAuth2 Apps                                           │
   │  🛡 Grafana                  grafana@idm.example.com  │
   │                                                       │
   │ Actions                                               │
   │  📋 Create scope map for Grafana                      │
   │  🔑 View basic secret for Grafana                     │
   │                                                       │
   │  ↑↓ navigate · ↵ select · esc close                   │
   └───────────────────────────────────────────────────────┘
```

**Behavior:**
- `Cmd+K` / `Ctrl+K` from anywhere opens the palette as a centered modal overlay
- Backdrop dims the page (subtle)
- Input is auto-focused
- Empty state shows: Recent (recent items the admin viewed), Quick actions (Create person, etc.), Navigate (top-level sections + sign out)
- As the admin types, results filter:
  - Person entities matching name, SPN, displayname, email
  - Group entities matching name, description
  - OAuth2 apps matching name, displayname
  - Quick actions matching action label
  - Navigation destinations matching label
- Results grouped by entity type with section headers
- Selected row: design system's accent color background
- Arrow keys navigate (Up / Down). Wraps top↔bottom.
- Enter selects the highlighted item: navigates to detail or invokes the action
- Mouse hover changes selection
- Esc closes the palette without action
- Closing also removes any modal focus trap

**Fuzzy match:**
- Use a fuzzy matcher (e.g., Levenshtein-distance or character-subset) so typing "grafna" still matches "grafana"
- Match-highlighting: bold the matched characters in the result label

**Recent items:**
- Track which entities the admin has viewed recently (localStorage)
- Show top 5-10 in the "Recent" section when palette is empty
- Recents are user-specific and persist across sessions

## Search vs filter distinction in copy

Make labels clear:
- "Search…" for finding a specific thing
- "Filter:" for narrowing the list

Don't mix: a single dropdown shouldn't say "Filter / Search by status".

## Empty results in palette

When the admin types and nothing matches:
```
   No results for "xyz"
   Try a different search, or [+ Create person] [+ Create group] [+ Create OAuth2 app]
```

The "create" suggestions are quick-action links — useful if the admin was searching for something that doesn't exist yet.

## Sample data references

For mockups, pull from `_sample-data.md`:
- Recent items: Alice Smith (Person), Grafana (OAuth2)
- When user types "graf": only Grafana matches in OAuth2 Apps + "Create scope map for Grafana" action
- When user types "ad": both `admin` (Person) and `idm_admins` (Group) match

## Accessibility

- Palette is a modal with proper focus trapping
- Results list is aria-live region (announces "5 results for 'graf'")
- Each result has role="option"; container is role="listbox"
- Selected state communicated via aria-selected

## Design system variations

- **Linear:** dense palette, ~12px text, sharp transitions, monospace shortcut hints prominent
- **Cloudflare:** comfortable padding, friendly tone, slightly larger result rows
- **Stripe:** roomy palette, ~14px text, soft gradient backdrop, slightly more decorative result row design

## Mockup elements to render

Render 3 distinct search/filter mockups:

1. **List search + filter** — A people list with search input "alic" typed, filter chip "Status: Active" visible, table showing filtered results
2. **Command palette — empty state** — Cmd+K opened, no query, showing Recent (Alice Smith, Grafana) + Quick actions + Navigate sections + keyboard hints footer
3. **Command palette — with results** — Cmd+K with "graf" typed, showing Grafana under OAuth2 Apps + 2 contextual actions, fuzzy match highlighting
