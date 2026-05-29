# 62 — OAuth2 Apps: Detail View (Overview + Tabs Shell)

The detail page for a single OAuth2 application. Tabbed layout covering all configurable surfaces.

## Purpose

Single page to administer an OAuth2 application. Tabs split the rich configuration surface into related groups. Overview tab summarizes everything; other tabs are detail screens (63-69).

## Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ OAuth2 Apps > grafana                                               │
│                                                                     │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ [Grafana icon 64px]   Grafana                                │    │
│ │                       grafana                                │    │
│ │                       Confidential                           │    │
│ │                       https://grafana.example.com  [copy]    │    │
│ │                                                              │    │
│ │                       [Edit] [View secret] [⋯ more]          │    │
│ └──────────────────────────────────────────────────────────────┘    │
│                                                                     │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ Overview │ General │ Scope maps │ Claim maps │ Crypto │ Image │ Advanced │ │
│ ├──────────────────────────────────────────────────────────────┤    │
│ │                                                              │    │
│ │  (Tab content)                                               │    │
│ │                                                              │    │
│ └──────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- Breadcrumb: `OAuth2 Apps > grafana` (system name as last segment)

## Identity card (top)

- App image (large, 64-96px) — uploaded image or placeholder
- Display name (large, primary): "Grafana"
- System name (monospace, secondary): "grafana"
- Type badge: "Confidential" or "Public"
- Landing URL (subdued monospace, with copy button): `https://grafana.example.com`
- Right side / below: action buttons:
  - Edit (jumps to General tab)
  - View secret (confidential only; opens Secret card on the detail or jumps to a secret-specific view in screen 64)
  - ⋯ kebab: Rename, Upload image, Delete, Copy UUID

## Tabs

7 tabs (designer can collapse some into kebab if too many):

1. **Overview** (default) — summary view, see below
2. **General** — basic settings + toggles → screen 63
3. **Scope maps** — group → scopes config → screen 65
4. **Claim maps** — group → claim values → screen 66
5. **Crypto** — signing keys → screen 67
6. **Image** — app icon → screen 68
7. **Advanced** — refresh token expiry, device flow → screen 69

The Secret view (screen 64) is accessed via the identity card's "View secret" button rather than as a tab (it's a sensitive action and shouldn't be incidental).

URL preserves tab.

## Overview tab content

```
┌────────────────────────────────────────────────────────────────┐
│ Configuration summary                                          │
│                                                                │
│ Display name      Grafana                                      │
│ System name       grafana                              [copy]  │
│ UUID              3f8a2c1d-…-…                         [copy]  │
│ Type              Confidential                                 │
│ Landing URL       https://grafana.example.com          [copy]  │
│ Supplementary     https://grafana.example.com/login/generic…  │
│ redirect URLs     [1 URL configured]                           │
│                                                                │
│ Toggles                                                        │
│   PKCE                          ●                              │
│   Strict redirect URL           ●                              │
│   Localhost redirects           ○                              │
│   Consent prompt                ●                              │
│   Prefer short username         ○                              │
│   Legacy crypto (HS256)         ○                              │
│                                                                │
│ → Configure on General tab                                     │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ Scope maps                                       → Manage      │
│                                                                │
│ Standard scope maps                                            │
│   idm_admins   openid, profile, email, groups                  │
│   developers   openid, profile, email, groups                  │
│   vpn_users    openid, email                                   │
│                                                                │
│ Supplementary scope maps                                       │
│   idm_admins   grafana_admin                                   │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ Claim maps                                       → Manage      │
│                                                                │
│ No claim maps configured.                                      │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ Signing keys                                     → Manage      │
│                                                                │
│ key-7f3a2c1d  ● Active        Created 2026-01-12              │
│ key-2b8e5d4a  ◐ Rotated       Created 2025-08-04              │
│ key-9c1f3e7b  ⊘ Revoked       Created 2025-02-19              │
└────────────────────────────────────────────────────────────────┘
```

### Configuration summary

A key-value list summarizing the General-tab settings. Read-only:
- Display name, System name, UUID, Type
- Landing URL
- Supplementary redirect URLs (count + first one visible; "+N more" if multiple)
- Toggles: 6 toggle indicators (filled dot = on, outline = off). Use semantic color where relevant (legacy crypto on = warning color).
- "→ Configure on General tab" link

### Scope maps summary

Two sub-sections (standard + supplementary). Show first 3-5 maps; "+N more" if larger.

If no maps: "No scope maps configured. → Add maps on Scope maps tab."

### Claim maps summary

Similar pattern. Show first few; or "No claim maps configured."

### Signing keys summary

List with status indicator (active / rotated / revoked) per key.

## States

- **Loading:** skeleton.
- **App not found:** 404 empty state.
- **Read-only:** action buttons disabled.

## Sample data

Use `grafana` from `_sample-data.md`:
- Display name: Grafana
- System name: grafana
- UUID: `3f8a2c1d-7b4e-4f9a-9c2e-1d8b5e3a7c4f`
- Type: Confidential
- Landing URL: `https://grafana.example.com`
- Supplementary redirect URLs: `https://grafana.example.com/login/generic_oauth` (1 URL)
- Toggles: PKCE ●, Strict redirect ●, Localhost redirects ○, Consent prompt ●, Prefer short ○, Legacy crypto ○
- Standard scope maps: idm_admins (openid, profile, email, groups), developers (openid, profile, email, groups), vpn_users (openid, email)
- Supplementary scope maps: idm_admins (grafana_admin)
- Claim maps: none (use Nextcloud for the claim-map example in a separate mockup)
- Signing keys: 3 keys as in sample data

For a variant with Nextcloud, show:
- Claim maps populated (`nextcloud_quota`, `department` per sample data)

## Edge cases

- **Public client:** "View secret" button hidden. The Secret screen (64) shows "Public clients don't have a basic secret. PKCE is enforced for all auth code flows."
- **Built-in/system OAuth2:** unlikely; kanidm probably doesn't ship built-in clients. If any exist, similar built-in indicator pattern as groups.

## Mockup elements to render

- Breadcrumb
- Identity card for Grafana with image, display name, system name, "Confidential" badge, landing URL with copy, action buttons
- Tab bar with all tabs, Overview active
- Configuration summary card with all toggles and key-values
- Scope maps summary with sample data
- Claim maps summary showing "No claim maps configured." for grafana
- Signing keys summary with all 3 keys and status indicators
- Render a second variant: Nextcloud detail with claim maps populated
- Render a third variant: public client (Homelab SPA) — "View secret" hidden, type badge "Public" warm color, no claim/scope sample
