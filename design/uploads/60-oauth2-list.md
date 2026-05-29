# 60 — OAuth2 Apps: List View

The list of all OAuth2/OIDC clients (applications) configured in kanidm. Entry point for OAuth2 administration.

## Purpose

Browse all configured OAuth2 applications. See which are confidential (basic) vs public, their origin URLs, their app icons. Navigate to detail to configure scope maps, claim maps, secrets, etc. Or create a new OAuth2 application.

## Layout

Inside the app shell. Main content area. Two presentation modes:

### Mode A — Card grid (recommended for Stripe/Cloudflare variants, where the larger icons/whitespace shine)

```
┌─────────────────────────────────────────────────────────────────────┐
│ OAuth2 Apps                            [+ Create OAuth2 application]│
│ 6 applications                                                      │
│                                                                     │
│ ┌──────────────────────┐ ┌──────────────────────┐                   │
│ │ 🔍 Search apps…      │ │ Type: All            ▾│                   │
│ └──────────────────────┘ └──────────────────────┘                   │
│                                                                     │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                  │
│ │   [GR icon]  │ │   [NC icon]  │ │   [G  icon]  │                  │
│ │ Grafana      │ │ Nextcloud    │ │ Gitea        │                  │
│ │ confidential │ │ confidential │ │ confidential │                  │
│ │ grafana.exa… │ │ cloud.examp… │ │ git.example… │                  │
│ └──────────────┘ └──────────────┘ └──────────────┘                  │
│                                                                     │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                  │
│ │   [—]        │ │   [—]        │ │   [—]        │                  │
│ │ Vaultwarden  │ │ Homelab SPA  │ │ Deploy CLI   │                  │
│ │ confidential │ │ public       │ │ public       │                  │
│ │ vault.examp… │ │ dash.examp…  │ │ deploy.exam… │                  │
│ └──────────────┘ └──────────────┘ └──────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

Each card: ~200-240px wide, ~160px tall:
- Top: app image (48-64px) — uploaded image or placeholder with first 1-2 chars of display name
- Display name (primary text, font-medium)
- Type badge: "confidential" or "public" with semantic color (e.g., confidential = info, public = warning to highlight PKCE-only)
- Landing URL (subdued, truncated, monospace)
- Hover: card lifts slightly, cursor pointer
- Click: navigate to detail (screen 62)
- Right-corner kebab menu: View, Edit, Delete

### Mode B — Table (recommended for Linear variant, denser)

```
┌─────────────────────────────────────────────────────────────────────┐
│ OAuth2 Apps                            [+ Create OAuth2 application]│
│ 6 applications                                                      │
│                                                                     │
│ ┌──────────────────────┐ ┌──────────────────────┐                   │
│ │ 🔍 Search apps…      │ │ Type: All            ▾│                   │
│ └──────────────────────┘ └──────────────────────┘                   │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ App                       │ Type        │ Landing URL          │ │
│ │───────────────────────────┼─────────────┼──────────────────────│ │
│ │ [Gr] Grafana (grafana)    │ confidential│ grafana.example.com  │ │
│ │ [Nc] Nextcloud (nextcloud)│ confidential│ cloud.example.com    │ │
│ │ [G ] Gitea (gitea)        │ confidential│ git.example.com      │ │
│ │ [—]  Vaultwarden          │ confidential│ vault.example.com    │ │
│ │ [—]  Homelab SPA          │ public      │ dash.example.com     │ │
│ │ [—]  Deploy CLI           │ public      │ deploy.example.com   │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

Designer picks based on the design system. Both are acceptable; the brief is mode-agnostic and just specifies the data and behavior.

## API data shape — critical notes

See `../api-reality.md` for full detail. Brief summary for this screen:

- **Type (basic/public)** is determined by the `class` array on the entry: `oauth2_resource_server_basic` vs `oauth2_resource_server_public`. There is no separate `type` attr.
- **Image** is a SHA256 hash (`image: ["1d10...233c"]`) — NOT a URL or path. The UI fetches the image via a separate endpoint like `/v1/oauth2/{name}/_image`.
- **Landing URL** is `oauth2_rs_origin_landing` (single value); supplementary redirect URLs are in `oauth2_rs_origin` (array, may be absent).
- The fields `oauth2_rs_scope_map`, `oauth2_rs_claim_map`, `key_internal_data` are **encoded strings**, not structured — see detail screens for parsing notes.
- All booleans (PKCE, strict redirect, etc.) are encoded as `"true"` / `"false"` STRINGS.
- Default-valued fields are OMITTED from the entry — the UI cannot infer "off" from absence.

## Page header

- Title: "OAuth2 Apps"
- Subtitle: "{N} applications"
- Right: "+ Create OAuth2 application" primary button → screen 61

## Controls row

- Search input: "Search by name or URL…" debounced
- Filter dropdown:
  - All (default)
  - Confidential
  - Public
  - Has image
  - No image
  - With legacy crypto (HS256)
  - With device flow enabled
- Sort dropdown: Name (default) / Created / Modified / Type

## Card / row content

For each OAuth2 app:
- **Image** — if `image` attribute is set, render uploaded image (48-64px in cards, 32px in table). Otherwise, placeholder with first 1-2 chars of display name in a colored circle.
- **Display name** (primary): "Grafana"
- **System name** (subdued): "grafana" — shown in parens or on second line
- **Type badge:** "confidential" or "public"
- **Landing URL** (truncated, monospace, copy-friendly)
- **Actions menu / kebab:**
  - View details (default click)
  - Edit general
  - Manage scope maps
  - View basic secret (only for confidential)
  - Delete (red)

## States

- **Loading:** skeleton cards/rows.
- **Empty:** illustration + "No OAuth2 applications yet" + "Create your first OAuth2 application to enable SSO for one of your services." + primary CTA.
- **Search no match:** "No applications match '{query}'" + Clear.
- **Error:** "Could not load OAuth2 applications." + Retry.

## Pagination

Same pattern as other lists. For 6 apps, just "Showing 1–6 of 6" with no page nav. For larger instances, paginate.

## Sample data

Use all 6 OAuth2 apps from `_sample-data.md`:

| Name | Display | Type | Landing | Image |
|---|---|---|---|---|
| grafana | Grafana | confidential | https://grafana.example.com | yes (grafana.svg) |
| nextcloud | Nextcloud | confidential | https://cloud.example.com | yes (nextcloud.png) |
| gitea | Gitea | confidential | https://git.example.com | yes |
| vaultwarden | Vaultwarden | confidential | https://vault.example.com | no |
| homelab-spa | Homelab Dashboard (SPA) | public | https://dash.example.com | no |
| cli-deploy-tool | Deploy CLI | public | https://deploy.example.com/auth/callback | no |

Subtitle: "6 applications"

## Edge cases

- **Image upload failure / broken URL:** fall back to placeholder gracefully.
- **Public client with non-HTTPS landing URL:** highlight with warning indicator (security). Tooltip: "Public clients should use HTTPS in production."
- **Confidential client with default secret never rotated:** no UI indicator (the secret is hidden — we don't know). But the detail page should flag "secret last rotated: never" if applicable.
- **App name with unusual characters (kanidm allows underscores, dashes, dots):** render as-is in monospace.

## Mockup elements to render

- Page header with title + count + Create button
- Search + filter row
- Choose ONE display mode (card grid OR table) per design system variant; render that
- Show all 6 sample apps with their images (or placeholders) and metadata
- Hover state on one card / row
- Render the empty state as a second variant (no apps)
