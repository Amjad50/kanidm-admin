# 10 — Dashboard / Overview

The landing page after sign-in. An at-a-glance summary of the kanidm instance with quick navigation to common tasks.

## Purpose

Give the admin instant orientation: what's in this instance (counts), who they're signed in as, where to go next. Avoid noise; this is a launchpad, not a dwelling space.

Because kanidm does not expose an audit/activity log API, the dashboard **does not** show "recent activity". Instead, it shows static counts, instance info, and the current session's status.

## Layout

The page lives inside the app shell (screen 00). Main content area:

```
┌─────────────────────────────────────────────────────────────────┐
│ Dashboard                                                       │
│                                                                 │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐                          │
│ │ 👥        │ │ 🛡️        │ │ 🔐        │                          │
│ │ 127       │ │ 18       │ │ 6        │                          │
│ │ People    │ │ Groups   │ │ OAuth2   │                          │
│ │ → View    │ │ → View   │ │ → View   │                          │
│ └──────────┘ └──────────┘ └──────────┘                          │
│                                                                 │
│ ┌──────────────────────┐ ┌─────────────────────────┐            │
│ │ Instance             │ │ Your session            │            │
│ │                      │ │                         │            │
│ │ idm.example.com      │ │ admin                   │            │
│ │ Example Org IDM      │ │ admin@idm.example.com   │            │
│ │ Domain level 8       │ │                         │            │
│ │ Server v1.6.0        │ │ Signed in 38 min ago    │            │
│ │                      │ │ Privileged session      │            │
│ │ uuid: 00000000-…     │ │ active for 22 min       │            │
│ └──────────────────────┘ │                         │            │
│                          │ → My sessions           │            │
│                          └─────────────────────────┘            │
│                                                                 │
│ Quick actions                                                   │
│ ┌────────────────────┐ ┌────────────────────┐                   │
│ │ + Create person    │ │ + Create group     │                   │
│ └────────────────────┘ └────────────────────┘                   │
│ ┌────────────────────┐ ┌────────────────────┐                   │
│ │ + Create OAuth2 app│ │ + Generate reset   │                   │
│ │                    │ │   link             │                   │
│ └────────────────────┘ └────────────────────┘                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Page header

- Title: "Dashboard"
- No breadcrumb (this is the root)
- Optional subtitle: "Welcome back, System Administrator." (Use the user's display name. If `displayname` is empty, fall back to SPN.)
- Optional "Refresh" icon-button top-right that re-fetches counts.

## Three metric cards (top row)

Each card represents one of the major sections. Click anywhere on the card to navigate to that section's list page.

| Card | Icon (Lucide) | Count source | Click destination |
|---|---|---|---|
| People | `Users` | SCIM filter `class eq "person"` count | `/people` |
| Groups | `UsersRound` | SCIM filter `class eq "group"` count | `/groups` |
| OAuth2 Apps | `Shield` | SCIM filter `class eq "oauth2_resource_server"` count | `/oauth2` |

Card content:
- Top-left: icon in a circular soft-background container (e.g., `--accent-soft` background, accent icon — design system dictates color)
- Big number: large bold (per design system's `--font-2xl` / dashboard hero number style)
- Label below number: "People", "Groups", "OAuth2 Apps" (lowercase or sentence case per design)
- Hover affordance: card lifts/highlights per design system; cursor pointer
- Click affordance: implicit "→ View {N} people" link bottom-right of the card

For the **Stripe variant**, the hero numbers can use the gradient text treatment for visual emphasis. For **Linear** and **Cloudflare**, plain `--text-primary` is fine.

The cards may be a 3-column grid on desktop, stacking to 2 columns on tablets and 1 column on mobile.

## Instance + session cards (middle row)

Two larger cards in a 2-column layout.

### Instance card

Shows static info about this kanidm deployment. Source: `GET /v1/domain` which returns `Vec<Entry>` — the UI takes the first element. The Entry is the standard `{"attrs": {key: [vals]}}` shape.

Relevant attrs:
- `name` — domain name like `idm.example.com`
- `domain_display_name` — human-readable name
- `uuid`
- `domain_level` — functional level as string-encoded integer
- `domain_ldap_basedn` — LDAP base DN

Fields rendered:
- **Domain name** (large): `idm.example.com` (from `name`)
- **Display name** (secondary): `Example Organization IDM` (from `domain_display_name`)
- **Domain functional level:** `8` (from `domain_level`, parseInt)
- **LDAP base DN:** `dc=idm,dc=example,dc=com` (from `domain_ldap_basedn`)
- **UUID** (small, monospace, copy-button): `00000000-0000-0000-0000-ffff00000000`

There is NO server-version endpoint. Don't fabricate one — omit. If the response headers happen to include a `Server: kanidm/X.Y.Z` field, the UI can surface that, but kanidm doesn't standardize this.

The endpoint may return `ClientError::EmptyResponse` if the privilege session is expired or the user lacks read access. Handle gracefully: show "Could not load instance information. Re-authenticate?" with a retry button that opens the reauth modal.

### Session card

Shows info about the signed-in admin's current session.

Fields:
- **Display name** (large): "System Administrator"
- **SPN** (secondary, monospace): `admin@idm.example.com`
- **Signed in:** "38 minutes ago" (relative time, from session's `issued_at`)
- **Session expires:** "in 6 hours 22 minutes" (relative, from `expiry`)
- **Privileged session status:**
  - If active: green dot + "Privileged session active — 22 minutes remaining" 
  - If not active: gray dot + "Privileged session expired" + small "Re-authenticate" link (opens reauth modal)
- Link at bottom: "→ My sessions" navigates to screen 81

## Quick actions section

A header "Quick actions" and a grid of 4 large action buttons / cards.

Each is a card-button (~200px wide × 100px tall) with:
- Icon top-left
- Label (font-medium, primary text color)
- Optional description below in subdued text

The 4 quick actions:
1. **Create person** — Lucide `UserPlus` icon, navigates to `/people/new`
2. **Create group** — Lucide `UsersRound` / `PlusCircle` icon, navigates to `/groups/new`
3. **Create OAuth2 application** — Lucide `Shield` / `Plus` icon, navigates to `/oauth2/new`
4. **Generate reset link** — Lucide `KeyRound` icon, opens a small flow: pick a person → generate intent token (this is essentially a shortcut into the credentials reset flow described in screen 24)

## States

- **Loading (initial):** show three skeleton metric cards, two skeleton info cards, and the quick action grid. Counts appear as they fetch. The page is interactive while data loads.
- **Counts failed:** if any count fetch fails, show "—" in that card with a small retry icon button.
- **Domain info failed:** instance card shows "Could not load instance information. Retry" — single retry button.
- **Empty instance (brand new install):** counts are 0. Cards still render, with "0" big numbers. Quick actions become extra-relevant. Optionally add a banner at the top: "Welcome to your new kanidm instance. Start by creating a person."

## Sample data

Pull from `_sample-data.md`:
- People count: **127**
- Groups count: **18**
- OAuth2 count: **6**
- Domain: `idm.example.com` / "Example Organization IDM" / level 8 / uuid `00000000-0000-0000-0000-ffff00000000`
- Current user: `admin` / "System Administrator" / `admin@idm.example.com`
- Signed in 38 minutes ago, session expires in 6h 22m, privileged for 22 minutes more

## Edge cases

- **Very large instance (10,000+ people):** counts come from SCIM filter responses which fetch full entries. For very large instances, this is slow. The dashboard should fetch counts in parallel and show each card's number as it arrives (don't block). Long-term, kanidm could add a count-only endpoint; not in scope here.
- **Server version unavailable:** omit field; don't fabricate.
- **No `displayname` on the current user:** show only SPN.
- **Read-only admin (no write privileges):** the quick actions section is hidden, OR each button is disabled with a tooltip "Your account does not have write access." Designer's call — hidden is cleaner; disabled is more discoverable.

## Tone

Brief, factual. The dashboard is not a marketing surface — it's a control panel. Avoid encouraging copy ("Great work today!"); avoid emojis in production copy (icons only).

## Mockup elements to render

For a generated mockup, render:
- Page title "Dashboard"
- Three metric cards in a row: 127 People, 18 Groups, 6 OAuth2 Apps
- Instance card with all fields from sample data
- Session card with all fields
- Privileged session indicator visible (green dot + "Privileged session active — 22 minutes remaining")
- Quick actions section with all 4 buttons
- Use sample data values exactly as specified
