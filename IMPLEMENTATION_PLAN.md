# kanidm-admin-ui — Implementation Plan

A polished, server-rendered admin panel for [kanidm](https://github.com/kanidm/kanidm) that lives in its own repo and ships as a Docker container behind Traefik on the same domain as kanidm. Zero fork maintenance: it talks to kanidm exclusively over the public `/v1/` REST API using the user's existing bearer cookie.

This document is the single source of truth for the implementation. It is written to be handed to any agent (or human) to pick up a phase and complete it without further context. Each phase has explicit deliverables, file paths, and a verification recipe.

---

## Table of contents

- [Background](#background)
- [Stack & conventions](#stack--conventions)
- [Authentication model](#authentication-model)
- [Repository layout](#repository-layout)
- [Critical gotchas in the kanidm API](#critical-gotchas-in-the-kanidm-api)
- [Phase 0 — Foundation](#phase-0--foundation)
- [Phase 1 — Dashboard with real data](#phase-1--dashboard-with-real-data)
- [Phase 2 — People CRUD](#phase-2--people-crud)
- [Phase 3 — Groups + account policy](#phase-3--groups--account-policy)
- [Phase 4 — OAuth2](#phase-4--oauth2)
- [Phase 5 — Self, sessions, polish](#phase-5--self-sessions-polish)
- [Phase 6 — Login flow (optional, deferred)](#phase-6--login-flow-optional-deferred)
- [Cross-cutting patterns](#cross-cutting-patterns)
- [Verification across phases](#verification-across-phases)

---

## Background

kanidm has no real admin web UI — only a minimal HTMX panel at `/ui/admin` (read-only persons, basic groups) and a comprehensive CLI. Routine admin (creating OAuth2 clients, managing group members, resetting credentials) is tedious via CLI. This project ships a polished, fast, server-rendered admin panel as a **separate** service that uses kanidm's public REST API and reuses kanidm's session cookie for auth.

**Goals**
- Feel as fast and snappy as kanidm itself (server-render + HTMX, no SPA framework).
- Cover the full feature surface that's painful in CLI: People CRUD + creds reset + SSH + sessions + validity; Groups CRUD + members + account policy; OAuth2 full parity (scope maps, claim maps, crypto, image).
- Deployable standalone (Docker + Traefik on same domain → cookie is shared).
- **Zero fork maintenance** if upstream rejects.

**Non-goals (out of scope)**
- POSIX attributes (anywhere).
- Service accounts.
- Recycle bin / system config / IDM sync / raw SCIM.
- Login UI replacement — deferred to Phase 6.

**Source of truth for API shapes:** `design-briefs/api-reality.md` in the parent kanidm repo. Read it before touching the data layer.

---

## Stack & conventions

| Layer | Choice | Why |
|---|---|---|
| HTTP server | **Axum 0.8** | Modern, ergonomic, great extractors |
| Templating | **Askama 0.16** + `askama_web` (axum-0.8) | Compile-checked HTML, no runtime template loader |
| Styling | **Tailwind v4** with design tokens declared in `@theme` | Tokens become native Tailwind utilities — change one file, whole design changes |
| Client interactivity | **HTMX 2.0.9** (from CDN) | 80% of interactions are partial swaps |
| Client islands | **Preact 10 + TypeScript**, bundled by **Bun** | Only for genuinely stateful widgets (Cmd+K, datetime picker, multi-selects) |
| Kanidm client | **`kanidm_client` + `kanidm_proto` 1.10** crates | Typed Rust client wraps `/v1/` REST API |
| Config | **figment** (TOML + `KANIDM_ADMIN_*` env) | Both file and env, env wins |

**Hard preference: never edit `Cargo.toml` manually. Always use `cargo add <crate> [--features ...]`.**

**Code style**
- No comments unless explaining a non-obvious "why" (a hidden constraint, a workaround, an invariant). Identifiers should be self-documenting.
- No backwards-compat shims, no dead code, no half-finished implementations.
- No defensive checks for things internal code already guarantees.

---

## Authentication model

**Model B — reuse kanidm's session cookie.**

- kanidm sets a cookie named `bearer` (see `kanidm_proto::internal::COOKIE_BEARER_TOKEN`) on successful login.
- We deploy on the same Traefik domain as kanidm so the browser sends that cookie to us.
- The `AdminUser` extractor (in [src/auth.rs](src/auth.rs)) reads the cookie, calls `auth_valid()` + `whoami()`, and checks `memberof` for the admin group from config (`admin_group`, defaults to `idm_admins`). If absent → `Forbidden`. If cookie missing → `Unauthenticated` (redirect to kanidm login). Once we ship Phase 6, this becomes our own login.
- Every handler that needs an authenticated admin takes `AdminUser` in its signature.

**Per-request client.** `KanidmClientFactory::for_token(token)` builds a fresh `KanidmClient` with the user's token already set, so per-user permissions and audit trails are preserved.

---

## Repository layout

```
kanidm-admin-ui/
├── Cargo.toml                  # managed only via `cargo add`
├── package.json                # bun scripts for bundling
├── tsconfig.json
├── kanidm-admin-ui.example.toml
├── kanidm-admin-ui.toml        # gitignored; per-deployment
├── IMPLEMENTATION_PLAN.md      # this file
├── README.md
├── src/
│   ├── main.rs                 # axum server entry, AppState, router wiring
│   ├── config.rs               # figment loader
│   ├── auth.rs                 # KanidmClientFactory + AdminUser extractor
│   ├── error.rs                # AppError + IntoResponse
│   ├── handlers/
│   │   ├── mod.rs              # router(): mounts all sub-routers
│   │   ├── dashboard.rs
│   │   ├── people/
│   │   │   ├── mod.rs          # router for /people/*
│   │   │   ├── list.rs
│   │   │   ├── detail.rs
│   │   │   ├── create.rs
│   │   │   ├── edit.rs
│   │   │   ├── delete.rs
│   │   │   ├── credentials.rs
│   │   │   ├── ssh.rs
│   │   │   ├── radius.rs
│   │   │   ├── sessions.rs
│   │   │   └── validity.rs
│   │   ├── groups/
│   │   │   ├── mod.rs
│   │   │   ├── list.rs
│   │   │   ├── detail.rs
│   │   │   ├── members.rs
│   │   │   ├── policy.rs
│   │   │   └── { create, edit, delete }.rs
│   │   ├── oauth2/
│   │   │   ├── mod.rs
│   │   │   ├── list.rs
│   │   │   ├── detail.rs
│   │   │   ├── create.rs
│   │   │   ├── general.rs
│   │   │   ├── secret.rs
│   │   │   ├── scope_maps.rs
│   │   │   ├── claim_maps.rs
│   │   │   ├── crypto.rs
│   │   │   ├── image.rs
│   │   │   ├── advanced.rs
│   │   │   └── delete.rs
│   │   ├── self_/
│   │   │   ├── mod.rs
│   │   │   ├── profile.rs
│   │   │   └── sessions.rs
│   │   └── health.rs
│   ├── kanidm/                 # kanidm-specific helpers
│   │   ├── mod.rs
│   │   ├── entry.rs            # attr_first, attr_bool, attr_int, in_class
│   │   ├── scope_map.rs        # parse "group@spn: {\"a\", \"b\"}" lines
│   │   ├── claim_map.rs        # parse "claim:group:joinchar:value" lines
│   │   ├── key_state.rs        # parse "id: status alg counter" lines
│   │   └── policy.rs           # account policy field metadata + defaults
│   └── views/                  # template structs that don't fit in handlers/
│       └── mod.rs
├── templates/
│   ├── base.html               # app shell (sidebar, topbar, overlay slot)
│   ├── login_redirect.html     # tiny page telling user to log in via kanidm
│   ├── dashboard.html
│   ├── people/
│   │   ├── list.html
│   │   ├── detail.html
│   │   ├── _row.html           # partials for HTMX swaps (prefix _)
│   │   ├── _ssh_key.html
│   │   ├── ...
│   ├── groups/...
│   ├── oauth2/...
│   ├── self_/...
│   └── partials/
│       ├── _modal.html         # modal frame; takes block { body, footer }
│       ├── _table_pagination.html
│       ├── _toast.html
│       └── _empty.html
├── styles/
│   └── app.css                 # @import "tailwindcss"; @theme {...}; design tokens
├── islands/
│   ├── entry.ts                # mounts every island conditionally
│   ├── command_palette.tsx
│   ├── datetime_keyword.tsx
│   ├── scope_map_editor.tsx
│   └── claim_map_editor.tsx
├── design/                     # gitignored exception: vendored design HTML
│   ├── assets/tokens.css       # design system CSS variables
│   ├── 10-dashboard.html       # …39 screen files…
│   └── …
└── static/
    ├── app.css                 # built (gitignored)
    ├── app.js                  # built (gitignored)
    ├── favicon.svg             # from design/assets
    └── logo-square.svg
```

---

## Critical gotchas in the kanidm API

These are the surprises that hurt the previous attempt. Read [`design-briefs/api-reality.md`](../kanidm/design-briefs/api-reality.md) for the full treatment; this section summarizes the load-bearing ones.

1. **Entries are `attrs: BTreeMap<String, Vec<String>>`.** Booleans encoded as strings (`"true"`/`"false"`), integers as strings, multivalued for everything.
2. **`oauth2_allow_insecure_client_disable_pkce` is INVERTED.** `"true"` means PKCE is DISABLED. Render UI as "PKCE enabled" with the bit flipped.
3. **Account policy password attr is `auth_password_minimum_length`** (note the `auth_` prefix), not `password_minimum_length`.
4. **Pre-formatted scope map strings:** `oauth2_rs_scope_map` is `["groupname@spn: {\"scope1\", \"scope2\"}"]` — write a parser in `src/kanidm/scope_map.rs`.
5. **Pre-formatted claim map strings:** `oauth2_rs_claim_map` is `["claim:group:joinchar:value1,value2"]` — see `src/kanidm/claim_map.rs`.
6. **Image fetch URL** is `/ui/images/oauth2/{client_name}`, keyed by name not hash.
7. **No "disable account policy" endpoint** — only per-field reset: `DELETE /v1/group/{id}/_attr/<attr>`.
8. **Several endpoints return structured proto types**, not flat attrs: `CredentialStatus`, `UatStatus`, `CUIntentToken`, `Option<String>` for basic secret. Prefer these over scraping entry attrs.
9. **`UatStatus` time fields** are Unix timestamps (integers), not RFC3339.
10. **Reset-token format** is short (`xkdzk-fr2p7-...`), not a JWT — display verbatim.
11. **Group membership identifiers** accept both SPN (`alice@idm.example.com`) and UUID; backend normalizes.

---

## Phase 0 — Foundation

**Goal:** replace the throwaway base.html with the real design shell so every subsequent screen ports trivially. After this, you should be able to navigate to `/`, see the design's sidebar + topbar, and see the dashboard placeholder rendering inside the shell with real-looking styling.

### Tasks

#### 0.1 Set up design tokens as Tailwind utilities (shadcn-style)

**Hard rule for the whole codebase: no raw colors, no raw hex values, no `bg-zinc-900`, no `bg-(--name)`, no inline `style="color: #..."`. Every visual property goes through a named token rendered as a Tailwind utility (`bg-surface`, `text-primary`, `border-subtle`, `rounded-md`, `shadow-card`). Changing one file (`styles/tokens.css`) re-themes the whole app.**

The design HTML files already use this pattern with raw CSS variables (`var(--accent-default)`). We translate that into Tailwind v4 `@theme` blocks so the tokens become first-class utilities.

**File: `styles/tokens.css`** (new, single source of truth)

Declare semantic tokens scoped to a `@theme` block. Tailwind v4 auto-generates utilities from any `--color-*`, `--spacing-*`, `--radius-*`, `--shadow-*`, `--font-*` variable.

```css
@import "tailwindcss";

@theme {
  /* Surface — bg-canvas, bg-surface, bg-elevated, etc. */
  --color-canvas: #0f0f12;
  --color-surface: #1a1a1f;
  --color-elevated: #232328;
  --color-hover: #2a2a30;
  --color-active: #34343c;

  /* Borders — border-subtle, border-default, border-strong */
  --color-subtle: #2d2d33;
  --color-default: #3a3a42;
  --color-strong: #4d4d57;

  /* Text — text-primary, text-secondary, text-tertiary, text-disabled */
  --color-primary: #f4f4f7;
  --color-secondary: #b8b8c3;
  --color-tertiary: #8a8a98;
  --color-disabled: #54545d;

  /* Accent — bg-accent, text-accent, border-accent + bg-accent-soft etc. */
  --color-accent: #f6821f;
  --color-accent-hover: #fb9438;
  --color-accent-pressed: #d96e15;
  --color-accent-soft: rgb(246 130 31 / 0.12);
  --color-on-accent: #1a1a1f;

  /* Link */
  --color-link: #5fa9ff;
  --color-link-hover: #82bcff;
  --color-link-soft: rgb(95 169 255 / 0.12);

  /* Semantic — bg-success, text-warning, border-danger, etc. */
  --color-success: #4ec97a;
  --color-success-soft: rgb(78 201 122 / 0.14);
  --color-warning: #f4b740;
  --color-warning-soft: rgb(244 183 64 / 0.14);
  --color-danger: #ec5b59;
  --color-danger-soft: rgb(236 91 89 / 0.14);
  --color-info: #5fa9ff;
  --color-info-soft: rgb(95 169 255 / 0.14);

  /* Special surfaces */
  --color-code-bg: #232328;
  --color-token-bg: #15151a;

  /* Radii — rounded-sm, rounded, rounded-md, rounded-lg, rounded-pill */
  --radius-sm: 4px;
  --radius: 6px;
  --radius-md: 8px;
  --radius-lg: 10px;
  --radius-pill: 999px;

  /* Shadows — shadow-card, shadow-elevated, shadow-modal */
  --shadow-card: 0 1px 3px rgb(0 0 0 / 0.30), 0 1px 2px rgb(0 0 0 / 0.18);
  --shadow-elevated: 0 8px 24px rgb(0 0 0 / 0.40), 0 2px 6px rgb(0 0 0 / 0.25);
  --shadow-modal: 0 20px 40px rgb(0 0 0 / 0.55), 0 8px 16px rgb(0 0 0 / 0.35);

  /* Fonts — font-sans, font-mono */
  --font-sans: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --font-mono: "JetBrains Mono", "SF Mono", Menlo, Consolas, monospace;

  /* Layout */
  --spacing-sidebar: 240px;
  --spacing-topbar: 56px;
}

/* Light-mode override: same token names, different values. */
@layer base {
  [data-theme="light"] {
    --color-canvas: #f7f7fa;
    --color-surface: #ffffff;
    --color-elevated: #ffffff;
    --color-hover: #f0f0f4;
    --color-active: #e6e6ec;
    --color-subtle: #e5e5ea;
    --color-default: #d4d4dc;
    --color-strong: #b0b0bb;
    --color-primary: #1a1a1f;
    --color-secondary: #525260;
    --color-tertiary: #7a7a86;
    --color-disabled: #bcbcc4;
    --color-accent-soft: rgb(246 130 31 / 0.10);
    --color-on-accent: #1a1a1f;
    --color-code-bg: #f0f0f4;
    --color-token-bg: #f7f7fa;
  }
}

@source "../templates/**/*.html";
@source "../islands/**/*.{ts,tsx}";
```

**File: `styles/app.css`** (entry point — imports tokens, no styles of its own)

```css
@import "./tokens.css";

html, body {
  background: var(--color-canvas);
  color: var(--color-primary);
  font-family: var(--font-sans);
  -webkit-font-smoothing: antialiased;
}
```

Bun's build pipeline (`bun run build:css`) compiles this through Tailwind v4. Every `--color-*` becomes a utility:

| Token | Tailwind utilities generated |
|---|---|
| `--color-canvas` | `bg-canvas`, `text-canvas`, `border-canvas`, etc. |
| `--color-accent` | `bg-accent`, `text-accent`, `border-accent`, etc. |
| `--color-accent-soft` | `bg-accent-soft`, `text-accent-soft`, etc. |
| `--color-danger` | `bg-danger`, `text-danger`, `border-danger`, etc. |
| `--radius-md` | `rounded-md` |
| `--shadow-card` | `shadow-card` |

**In templates, write only these utilities:**

```html
<!-- ✅ correct -->
<div class="bg-surface border border-subtle rounded-md shadow-card p-6">
  <h2 class="text-primary text-lg font-semibold">Title</h2>
  <p class="text-secondary mt-1">Subtitle</p>
  <button class="bg-accent text-on-accent rounded h-9 px-4 hover:bg-accent-hover">
    Save
  </button>
</div>

<!-- ❌ never do this -->
<div class="bg-zinc-900 border-zinc-800 rounded-md" style="color: #f4f4f7">
```

**Porting the design HTML.** The design files use class names like `.card`, `.btn-primary`, `.badge-success`. We do **not** keep those — every porting step rewrites them as Tailwind utilities referencing our tokens. The design's CSS is a visual reference, not vendored code.

If a pattern repeats across 5+ screens (e.g., the modal frame), extract it into an Askama partial in `templates/partials/` so the utility soup stays in one place. Do **not** create CSS component classes — that defeats the "change one file" goal.

**Verification:** `bun run build` produces `static/app.css`. Inspect: the file contains `.bg-accent { background-color: var(--color-accent); }` and `:root { --color-accent: #f6821f; }`. Edit `--color-accent` in `tokens.css` to `#5e6ad2` (Linear violet), rebuild — every accent-colored element in the running app turns violet without touching any template.

**Theme switching.** Toggle is a single attribute swap: `document.documentElement.setAttribute('data-theme', 'light')`. The user-menu dropdown's theme toggle (Phase 0.5) does this and persists to `localStorage`.

#### 0.2 Copy design assets
- Copy `design/assets/{favicon.svg,logo-square.svg,kani-waving.svg}` into `static/`.

#### 0.3 Port the app shell into `templates/base.html`
The current `templates/base.html` uses raw Tailwind utilities (`bg-zinc-950` etc.) — **throw it away and start from `design/index.html`** (or any non-login screen — they all share the `.app` grid).

The shell must define these Askama blocks:

```
{% block title %}Kanidm Admin{% endblock %}
{% block crumb %}{% endblock %}            -- breadcrumb in topbar
{% block page_actions %}{% endblock %}     -- right side of topbar (optional)
{% block content %}{% endblock %}          -- main page content
{% block overlay %}{% endblock %}          -- never rendered server-side;
                                              kept for documentation only —
                                              HTMX swaps into #overlay-slot
```

Hard requirements:
- Element with `id="overlay-slot"` somewhere outside `.main`, used by HTMX as `hx-target="#overlay-slot" hx-swap="innerHTML"` for modals. Modals render their own `.modal-backdrop` so the slot stays empty by default.
- Element with `id="cmd-palette-island"` for the Preact Cmd+K mount.
- Sidebar nav items take a `current_section: &str` field passed from every handler's template struct. Render the `.active` class when `current_section == "people"` for the People link, etc. Add a `BaseFields` struct in `src/views/mod.rs` with `current_section`, `user_displayname`, `user_spn`, `domain_name` to avoid copy-paste.
- HTMX 2.0.9 and `/static/app.js` loaded via `<script defer>` in `<head>` — defer is fine because HTMX binds on `DOMContentLoaded`.

#### 0.4 Build the base layout struct
Create `src/views/mod.rs`:

```rust
pub struct BaseFields {
    pub current_section: &'static str,   // "dashboard" | "people" | "groups" | "oauth2" | "self"
    pub user_displayname: String,
    pub user_spn: String,
    pub domain_name: String,             // for sidebar header subtitle
    pub privileged: bool,                // for topbar privilege dot
}
```

Every page template embeds a `BaseFields` field named `base` and references it in the shell. Handlers build it in one line via a `BaseFields::for(&user, &state, "people")` constructor.

**The domain name should be cached.** Looking it up on every request is wasteful. Add an `Arc<RwLock<Option<DomainInfo>>>` to `AppState`, populate on first request to dashboard, refresh on TTL of 60s. Don't over-engineer this — a single mutex around a small struct is fine.

#### 0.5 Convert `AdminUser` to populate session metadata
Extend `AdminUser` with:
- `signed_in_at: Option<OffsetDateTime>` (Unix timestamp from UAT → `time::OffsetDateTime::from_unix_timestamp`)
- `session_expires_at: Option<OffsetDateTime>`
- `privileged: bool` (read `auth_type` or check expiry of `limit_search_max_results`-style policy from UAT — see api-reality §UAT)

These come from `kanidm_proto::internal::token::UserAuthToken`, decoded from the bearer cookie value. The cookie value is a base64-encoded JWS; use `kanidm_client::KanidmClient::get_token()` or decode directly. **If decoding the JWS in the extractor is painful, call `whoami()` and read structured fields from the `Entry`.** Don't reinvent JWT verification.

#### 0.6 Wire the sidebar nav to real routes
Routes that must exist (most return 404 with a friendly "coming in Phase N" message for now):

```
GET  /                            → dashboard
GET  /people                      → people list (Phase 2)
GET  /groups                      → groups list (Phase 3)
GET  /oauth2                      → oauth2 list (Phase 4)
GET  /me                          → self profile (Phase 5)
GET  /me/sessions                 → self sessions (Phase 5)
POST /logout                      → clears cookie, redirects to /
GET  /healthz                     → 200 OK
```

Use Axum's `Router::nest` to keep `/people/*`, `/groups/*`, `/oauth2/*`, `/me/*` in their own sub-routers under `src/handlers/{people,groups,oauth2,self_}/mod.rs::router()`. The top-level `handlers::router()` merges all of them.

#### 0.7 Build the 404 / 403 / 401 pages
- 401 unauthenticated → render a small page (`templates/login_redirect.html`) saying "Please log in to {kanidm_url}", with a link. After Phase 6 this becomes our login form.
- 403 forbidden → render a small page saying "You are not in the `{admin_group}` group. Contact an administrator." (Don't reveal members.)
- 404 → friendly empty state.

Wire these via `AppError::IntoResponse` returning the right template.

### Phase 0 deliverables
- `templates/base.html` reproduces `design/index.html`'s `.app` grid pixel-close (allowing for content area differences).
- `styles/app.css` includes design tokens; build produces working CSS.
- `static/{favicon.svg,logo-square.svg}` in place.
- `src/views/mod.rs::BaseFields` exists and is used by `DashboardView`.
- All sidebar nav links route somewhere (real handler or "Phase N coming soon" placeholder).
- `cargo run` succeeds; visiting `localhost:3000` (after copying the `bearer` cookie) shows the design shell with placeholder dashboard.

### Phase 0 verification
1. Open `design/10-dashboard.html` and our live `/` side by side. Sidebar, topbar, page padding, font, and accent color should match within 1–2 px tolerance.
2. Inspect: `--accent-default` resolves to `#f6821f` in our page.
3. Sidebar links: clicking People navigates to `/people` (placeholder), back arrow works, active state highlights correctly.
4. Forbidden flow: temporarily change `admin_group` to a group you're not in; refresh → see Forbidden page.

---

## Phase 1 — Dashboard with real data

**Goal:** the dashboard at `/` shows your actual kanidm instance's counts, domain info, and your live session details, styled exactly like `design/10-dashboard.html`.

### Tasks

#### 1.1 Port `design/10-dashboard.html` → `templates/dashboard.html`
- Keep the design's HTML structure verbatim where it's data-free (icons, layout, classes).
- Replace hardcoded counts (`127`, `18`, `6`) with `{{ person_count }}`, etc.
- Replace `idm.example.com` and `Example Organization IDM` with `{{ domain_name }}` / `{{ domain_display_name }}`.
- Replace the session card's hardcoded admin info with `{{ base.user_displayname }}` / `{{ base.user_spn }}` / signed-in / expires-at / privilege dot.
- Each metric card is a link (`<a class="metric" href="/people">`) — wire them.
- Quick action tiles link to `/people/new`, `/groups/new`, `/oauth2/new`, `/me/credentials` (4th is placeholder).

#### 1.2 Handle "count failed" gracefully
The current `DashboardView` already has `Option<usize>` counts and tolerates per-card failure. In the template, render `{% match person_count %}{% when Some with (n) %}{{ n }}{% when None %}—{% endmatch %}` and add a small "Failed to load" muted text below the number when `None`.

#### 1.3 Refresh button
The "Refresh" button in the page header should be `hx-get="/" hx-target="body" hx-swap="outerHTML"` — full page reload, but via HTMX so it feels snappy. Or just a plain `<a href="/">` — both fine.

#### 1.4 Format times
- "Signed in 38 minutes ago" — use `time` crate to compute relative duration. Add a tiny helper in `src/views/mod.rs::format_relative_past(OffsetDateTime) -> String`.
- "Session expires in 6 hours 22 minutes" — similar helper `format_relative_future`.
- "Active — 22 minutes remaining" — privileged session expiry; needs UAT decoding.

### Phase 1 deliverables
- `templates/dashboard.html` matches `design/10-dashboard.html` with real data substituted.
- Dashboard loads in <300ms against a local kanidm instance (`tokio::join!` runs all 4 fetches in parallel).
- All four quick-action tiles route to (placeholder) pages.

### Phase 1 verification
1. Counts on dashboard match `kanidm person list -o json | jq length` etc.
2. Domain name matches `kanidm system domain show`.
3. Session card shows your real username, signed-in time matches when you logged into kanidm, expires-at is in the future.
4. Kill kanidm temporarily → dashboard still renders with "—" for counts and "Failed to load" notes per card.

---

## Phase 2 — People CRUD

**Goal:** complete CRUD for persons. This phase pins the patterns we reuse for groups and OAuth2: list with search, detail with tabs, modal forms via HTMX, optimistic table updates.

### Routes

```
GET  /people                              → list, with ?q=, ?page=, ?valid=
GET  /people/new                          → create form
POST /people                              → create, redirect to detail
GET  /people/{id}                         → detail, defaults to overview tab
GET  /people/{id}/overview                → overview tab
GET  /people/{id}/credentials             → credentials tab
GET  /people/{id}/ssh                     → SSH keys tab
GET  /people/{id}/radius                  → RADIUS tab
GET  /people/{id}/sessions                → sessions tab
GET  /people/{id}/validity                → validity tab
GET  /people/{id}/edit                    → edit form
POST /people/{id}                         → save edits, redirect
GET  /people/{id}/delete                  → delete confirm (modal fragment)
POST /people/{id}/delete                  → delete, redirect to list

POST /people/{id}/credentials/reset       → generate intent token
POST /people/{id}/ssh                     → add SSH key
POST /people/{id}/ssh/{tag}/delete        → remove SSH key
POST /people/{id}/radius/regenerate       → regenerate RADIUS secret
POST /people/{id}/radius/delete           → remove RADIUS secret
POST /people/{id}/sessions/{sid}/destroy  → destroy session
POST /people/{id}/validity                → set valid-from / expires-at
POST /people/{id}/unlock                  → softlock-reset (api-reality §unlock)
```

`{id}` accepts SPN or UUID; the backend normalizes. Use `axum::extract::Path<String>` and pass through.

### Tasks

#### 2.1 List page — `templates/people/list.html`
Port `design/20-people-list.html`. Key elements:

- Page header: title "People", subtitle "{n} accounts", "Create person" CTA → `/people/new`.
- Toolbar: search input (`hx-get="/people"`, `hx-trigger="input changed delay:200ms, search"`, `hx-target="#people-table"`, `hx-include="this"`, `hx-push-url="true"`). The endpoint returns the **full page** if non-HTMX, or just the `<tbody>` partial (`templates/people/_rows.html`) if `HX-Request: true` is set. Same handler, branch on the header. Use the `axum_htmx::HxRequest` extractor (add via `cargo add axum-htmx`).
- Filter chips: validity (active / expired / future) wired as HTMX query toggles.
- Table: columns Avatar, Name, SPN, Primary email, Validity, Groups count, Last login (if we expose it), Actions. Each row is `<tr hx-get="/people/{id}" hx-target="body" hx-push-url="true">` so the whole row is clickable. Action cell has a `.kebab` button → HTMX dropdown.
- Pagination: server-side, `?page=` + `?per=`. Don't load all 127 rows on first paint; default `per=50`.

**Search implementation.** `kanidm_client::idm_person_account_list()` returns all persons. For small instances (<1000) we can filter server-side in Rust. For larger, use `idm_person_account_search(&q)` if it exists in the client — or fall through to filter in Rust. Keep it simple for now: fetch all, filter in Rust by SPN/displayname/mail substring (case-insensitive).

#### 2.2 Detail page — `templates/people/detail.html`
Port `design/22-people-detail.html` for the Overview tab. The detail page is split into:

- **Header card** (`.identity-card`): avatar, displayname, SPN, mail, group chips, top-right actions (Edit, Delete, kebab).
- **Tabs** (`.tabs > .tab`): Overview / Credentials / SSH Keys / RADIUS / Sessions / Validity. Each tab is a separate URL — clicking a tab is `<a href="/people/{id}/credentials" hx-get="..." hx-target="#tab-content" hx-push-url="true">`. The handler returns either the full page (with shell + header + tabs + active tab content) or just the `#tab-content` fragment.
- **Sidebar info card** (right column on wide screens): SPN, UUID, created, last modified, groups list.

**Important:** tabs use HTMX for partial-swap snappiness, but every tab URL also works as a full page load (for bookmarking and back button). The handler reads `HX-Request` and chooses fragment vs. full template.

Sub-tabs map to the screen files:
- Overview: groups, mail addresses, legal name, identity attrs.
- Credentials (`24-people-credentials.html`): show `CredentialStatus` from `idm_person_account_get_credential_status()`. "Generate reset link" button → modal with TTL picker (1h / 8h / 24h / custom) → POST to `/people/{id}/credentials/reset` → response renders the URL + QR + copy button.
- SSH (`25-people-ssh-keys.html`): list keys (`idm_person_account_list_ssh_pubkeys`); add form (tag + key); each row has remove button.
- RADIUS (`26-people-radius.html`): show status, "Regenerate" → one-time-show pattern (copy + dismiss); "Delete" with confirm.
- Sessions (`27-people-sessions.html`): `idm_account_list_user_auth_tokens` → list `UatStatus[]`; each row has destroy.
- Validity (`28-people-validity.html`): two datetime inputs with keyword shortcuts (now / never / clear) — Preact island.

#### 2.3 Create form — `templates/people/create.html`
Port `21-people-create.html`. Two fields: SPN (username) and display name. POST → `idm_person_account_create` → redirect to `/people/{spn}`.

#### 2.4 Edit form — `templates/people/edit.html`
Port `23-people-edit.html`. Editable: displayname, legal name, mail addresses (add/remove rows). POST → update via `idm_person_account_set_displayname`, etc. — one client call per changed field, or use `idm_person_account_update` if a unified setter exists. Show success toast on save (HTMX OOB swap into `#toast-stack`).

#### 2.5 Delete modal — `templates/people/_delete_modal.html`
HTMX pattern:
1. User clicks Delete → `hx-get="/people/{id}/delete" hx-target="#overlay-slot"`.
2. Server returns the `.modal-backdrop` containing the modal with a type-SPN-to-confirm input and a POST form.
3. User types matching SPN, submits → `hx-post="/people/{id}/delete"`. On success, server returns `HX-Redirect: /people` header.
4. Cancel button does `hx-get="/empty"` → returns empty string → clears `#overlay-slot`.

Add a tiny `GET /empty` route that returns `200 ""` for the cancel pattern.

#### 2.6 Credentials reset (24-people-credentials.html)
- Show `CredentialStatus` summary (which factor types exist, last used).
- "Generate reset link" button → modal with TTL radio buttons.
- On submit: POST `/people/{id}/credentials/reset` → kanidm returns a `CUIntentToken` (the short alphanumeric `xkdzk-fr2p7-fb5wd-5e2hf`). Server renders a "URL ready" view with:
  - The full URL: `{kanidm_url}/ui/reset?token={token}`.
  - A QR code (inline SVG generated server-side via `qrcode` crate).
  - Copy button (`navigator.clipboard.writeText` via small `<script>` block — or HTMX-style copy with `data-clipboard-text` and a Preact handler).
  - TTL warning.

Add `cargo add qrcode`.

### Phase 2 deliverables
- All routes above work end-to-end against a real kanidm instance.
- List filter/search has <300ms perceived latency.
- Modal pattern locked down (delete + credential reset use it).
- One-time-show pattern locked down (RADIUS, intent token).
- Tab-as-URL pattern locked down (used in detail page).

### Phase 2 verification
1. Create alice via UI → confirm via `kanidm person get alice`.
2. Edit displayname → confirm via CLI.
3. Add SSH key → confirm via `kanidm person ssh list-publickeys alice`.
4. Generate reset link → open URL in incognito → kanidm's reset flow accepts the token.
5. Delete alice → list no longer shows her.
6. Search "ali" → only matching rows shown; back button works (history pushed).

---

## Phase 3 — Groups + account policy

**Goal:** complete CRUD for groups including the messy account-policy fields.

### Routes

```
GET  /groups
GET  /groups/new
POST /groups
GET  /groups/{id}
GET  /groups/{id}/overview
GET  /groups/{id}/members
GET  /groups/{id}/policy
GET  /groups/{id}/edit
POST /groups/{id}
GET  /groups/{id}/delete
POST /groups/{id}/delete

POST /groups/{id}/members/add            → { name: "alice@..." }
POST /groups/{id}/members/{mid}/remove
POST /groups/{id}/members/purge          → confirm modal first

POST /groups/{id}/policy/{field}          → set
POST /groups/{id}/policy/{field}/reset    → DELETE attr (defaults restored)
```

### Tasks

#### 3.1 List + create + edit + delete
Pattern-identical to People. Port `40-groups-list.html`, `41-groups-create.html`, `43-groups-edit.html`, `46-groups-delete.html`.

#### 3.2 Members (`44-groups-members.html`)
- Member list as `.member-chip`s.
- Add via a single search input with HTMX `hx-post="/groups/{id}/members/add"` → on success, swap the member chip list (`#members-list`) with the updated partial. Use a `<datalist>` populated from `idm_person_account_list` for typeahead (no JS needed).
- Remove: each chip's `×` button is `hx-post="/groups/{id}/members/{mid}/remove" hx-target="closest .member-chip" hx-swap="outerHTML swap:200ms"`.
- Purge: confirm modal (type group name), then POST.

#### 3.3 Account policy (`45-groups-account-policy.html`)
This is the gnarly screen. Policy fields per api-reality:

| Field | Attr | Type | Default |
|---|---|---|---|
| Credential type minimum | `credential_type_minimum` | enum (any/mfa/passkey/attested) | `any` |
| Password minimum length | `auth_password_minimum_length` | int | 10 |
| Auth session expiry | `authsession_expiry` | int (seconds) | 3600 |
| Privilege session expiry | `privilege_expiry` | int (seconds) | 600 |
| WebAuthn attestation CAs | `webauthn_attestation_ca_list` | JSON | empty |
| Limit search max results | `limit_search_max_results` | int | 1024 |
| Limit search max filter test | `limit_search_max_filter_test` | int | 2048 |
| Allow primary cred fallback | `allow_primary_cred_fallback` | bool | false |

Each field renders as its own small card with:
- Current value (or "Default — *value*" muted) read from the entry attrs.
- Edit form (one field, one Save button), `hx-post`s to `/groups/{id}/policy/{field}`.
- Reset button → `hx-post`s to `/groups/{id}/policy/{field}/reset` → DELETE attr.

**There is no "enable account policy" toggle in kanidm.** It's enabled the moment any attr exists. The page header says "Account policy — {n} customizations" and the design's "Enable" toggle from the brief is REMOVED. Don't ship it; the api-reality.md notes this was wrong in the original brief.

Field metadata (label, helper, default, attr name, type) lives in `src/kanidm/policy.rs` as a static slice. The template iterates it. Adding new policy fields = one entry in that slice.

### Phase 3 deliverables
- Group CRUD pattern-identical to People.
- Member add/remove with HTMX swap-in-place, datalist typeahead.
- Policy page with per-field set/reset; no global toggle.

### Phase 3 verification
1. Create group → confirm CLI.
2. Add member, remove member → confirm CLI.
3. Set `auth_password_minimum_length=14` → confirm via `kanidm group account-policy show developers`.
4. Reset that field → it disappears from CLI output.

---

## Phase 4 — OAuth2

**Goal:** full feature parity with the OAuth2 CLI. This is the deepest screen surface — six sub-tabs and several encoded-string formats.

### Routes

```
GET  /oauth2
GET  /oauth2/new                          → wizard step 1 (type select)
POST /oauth2                              → create, redirect to detail
GET  /oauth2/{id}
GET  /oauth2/{id}/general
GET  /oauth2/{id}/secret                  → basic auth only
GET  /oauth2/{id}/scope-maps
GET  /oauth2/{id}/claim-maps
GET  /oauth2/{id}/crypto
GET  /oauth2/{id}/image
GET  /oauth2/{id}/advanced
GET  /oauth2/{id}/delete
POST /oauth2/{id}/delete

POST /oauth2/{id}/general                 → name, displayname, landing, toggles
POST /oauth2/{id}/redirect/add
POST /oauth2/{id}/redirect/{idx}/remove

POST /oauth2/{id}/secret/reset            → returns new secret (one-time-show)

POST /oauth2/{id}/scope-map/standard      → { group, scopes: [..] }
POST /oauth2/{id}/scope-map/standard/{g}/delete
POST /oauth2/{id}/scope-map/supplementary
POST /oauth2/{id}/scope-map/supplementary/{g}/delete

POST /oauth2/{id}/claim-map               → { claim, group, join, values: [..] }
POST /oauth2/{id}/claim-map/{c}/{g}/delete

POST /oauth2/{id}/crypto/rotate           → { at: "now" | datetime }
POST /oauth2/{id}/crypto/revoke           → { key_id }

POST /oauth2/{id}/image                   → multipart upload
POST /oauth2/{id}/image/delete

POST /oauth2/{id}/advanced                → refresh expiry, etc.
```

### Tasks

#### 4.1 List (`60-oauth2-list.html`)
Card grid by default; toggle to table. Each card shows image (or initial), name, type badge (basic/public), landing URL, kebab menu. Image fetched from `{kanidm_url}/ui/images/oauth2/{name}`.

#### 4.2 Create wizard (`61-oauth2-create.html`)
Two steps:
1. Pick type: Basic (confidential) or Public (PKCE-only, mobile/SPA).
2. Name + display name + landing URL → review → create.

POST → `idm_oauth2_rs_basic_create` or `idm_oauth2_rs_public_create`. Redirect to `/oauth2/{name}/general`.

#### 4.3 Detail tabs (`62`–`69`)
Same tab-as-URL pattern as People.

**General (`63`).** Editable name (with rename warning callout), displayname, landing URL, supplementary redirect URLs (`.list-row` with add/remove), toggles for:
- PKCE — **invert** `oauth2_allow_insecure_client_disable_pkce`
- Strict redirect URL — `oauth2_strict_redirect_uri`
- Localhost redirects — `oauth2_allow_localhost_redirect`
- Consent prompt — `oauth2_consent_prompt`
- Prefer short username — `oauth2_prefer_short_username`
- Legacy crypto — `oauth2_jwt_legacy_crypto_enable`

Each toggle is a tiny form: `<form hx-post="/oauth2/{id}/general"><input type="hidden" name="field" value="pkce"><input type="checkbox" name="enabled" ...></form>` with `hx-trigger="change"`.

**Secret (`64`).** Basic auth only. Display masked secret with reveal toggle; Regenerate button → confirm modal → POST → response renders the new secret in one-time-show mode (copy + dismiss). Public clients show a callout "Public clients have no secret; PKCE is required."

**Scope maps (`65`).** Two sections: Standard and Supplementary. Each is a list of `(group, scopes[])` rows.

- Parse `oauth2_rs_scope_map` and `oauth2_rs_sup_scope_map` entries with `src/kanidm/scope_map.rs`. The format is `groupname@spn: {\"openid\", \"profile\"}` — strip prefix until `:`, then JSON-parse the brace-set after replacing braces with brackets, or write a tiny parser.
- Add row: group multi-select (from group list) + scope multi-select with standard scopes (openid, profile, email, groups, ssh_publickeys, etc.) + custom scope text input. **Preact island** because of the multi-select interaction.
- Edit row: open inline editor (HTMX `hx-get` returns the editor partial in place).
- Delete row.

**Claim maps (`66`).** Similar to scope maps but with a `join` strategy picker (csv/ssv/array) per row. Parse format: `claim:group:joinchar:value1,value2`. Use `Oauth2ClaimMapJoin` enum from `kanidm_proto::oauth2`.

**Crypto (`67`).** List current keys (parse `key_internal_data` strings: `id: status alg counter`). Actions: Rotate (schedule with datetime or "now") and Revoke (heavy confirm: type key ID).

**Image (`68`).** Show current image (or "no image" placeholder). Upload form: file input with accept=`image/png,image/jpeg,image/svg+xml,image/webp,image/gif`. Multipart upload to `POST /v1/oauth2/{id}/_image`. Remove button → DELETE attr.

**Advanced (`69`).** Refresh token expiry (seconds, blank = default). Device flow toggle if available.

**Delete (`6A`).** Same modal pattern as people — type name to confirm.

### Phase 4 deliverables
- All six sub-tabs working.
- Parsers for scope maps, claim maps, and key data all unit-tested with real CLI fixtures.
- PKCE attr inversion correct (test: PKCE toggle ON in UI ↔ `oauth2_allow_insecure_client_disable_pkce: false` in entry).
- Image upload accepts all five formats; image served from kanidm's URL not ours.

### Phase 4 verification
1. Create a "grafana" basic client via UI → confirm via `kanidm system oauth2 get grafana`.
2. Toggle PKCE off → confirm `oauth2_allow_insecure_client_disable_pkce=true` in CLI output.
3. Add scope map (developers → openid, profile, email, groups) → confirm via CLI.
4. Add claim map (groups → developers → csv → admin,user) → confirm.
5. Rotate keys → CLI shows new key with status `valid`, old one `retired`.
6. Upload image → image appears in card grid on list page.

---

## Phase 5 — Self, sessions, polish

### Routes

```
GET  /me
GET  /me/sessions
POST /me/sessions/{sid}/destroy
POST /logout
```

### Tasks

#### 5.1 Self profile (`80-self.html`)
Read-only view of current admin's own entry — same shape as person detail Overview but with "Logout" as the primary action. Mostly a copy of `templates/people/detail.html` with editing stripped.

#### 5.2 Self sessions (`81-sessions.html`)
Similar to person sessions tab but for the current user.

#### 5.3 Reauth modal (`08-reauth-modal.html`)
Triggered when a privileged write returns 401 priv-required. The flow:
1. Backend returns `HX-Trigger: kanidm-reauth` header on the failed response.
2. A tiny script in `islands/entry.ts` (or a `body hx-on::trigger`) reacts to that event by firing `hx-get="/reauth"` into `#overlay-slot`.
3. The reauth modal posts to kanidm's `auth_step` and on success closes the overlay and replays the original mutation via `hx-trigger` after a swap.

Defer if complex — kanidm's privilege expiry is normally 10min so most operations won't hit this in a session.

#### 5.4 Cmd+K palette (Preact island)
Already mounted in `templates/base.html`. Wire it to:
- Search across people, groups, oauth2 apps (3 parallel `fetch` calls to `/people?q=&format=json`, etc.). Add a JSON variant to list endpoints behind `Accept: application/json` for this.
- Keyboard nav (arrow keys + enter).
- Recent items in localStorage.

#### 5.5 Toast system
A small Preact island bound to a global event. Server returns `HX-Trigger: {"toast": {"title": "...", "kind": "success"}}` on mutations; client renders into `#toast-stack`.

### Phase 5 deliverables
- `/me` and `/me/sessions` complete.
- Cmd+K opens with ⌘K, fuzzy-searches across all three resource types, enter navigates.
- Toast notifications on every mutation.
- Reauth modal (best effort).

---

## Phase 6 — Real authentication (OIDC against kanidm)

**Problem this solves.** Today the admin UI reads kanidm's `bearer` session cookie directly. To log in, a user must log into kanidm somewhere, open devtools, copy the cookie, and paste it into a cookie on the admin-ui origin. Fragile, manual, no refresh, and blocks ever offering this upstream. We replace it with the standard pattern every other kanidm client (Grafana, Nextcloud, Gitea) already uses: **OIDC authorization-code flow with PKCE against kanidm itself**.

**Approach.** Register the admin UI as a **public** OAuth2 client in kanidm (PKCE-only, no client secret). On a cold request the UI redirects to kanidm's own `/ui/oauth2/authorise?...`. The user authenticates on kanidm's existing login pages (passwords, passkeys, TOTP — all of kanidm's existing UX, unchanged). Kanidm redirects back to `/auth/callback?code=...`. We exchange the code at `/oauth2/token`, get an access token (a JWT), drop it into a server-set `HttpOnly` `Secure` `SameSite=Lax` cookie scoped to the admin-ui origin, and let the existing `AdminUser` extractor read it. Refresh tokens handled in the background.

**Why not other options.** A popup/iframe + `postMessage` would require modifying kanidm's login pages — defeats the "zero-fork" goal. Resource Owner Password Credentials (user types password into our form) bypasses MFA/passkeys and is an anti-pattern for federated identity.

**Why this slots here, not earlier.** Phase 4 ships the OAuth2 admin screens. With those in place, the very first thing we do in Phase 6 is register `kanidm-admin-ui` *using the admin UI itself* — nice dogfooding loop. Doing it before Phase 4 would mean writing the registration via raw CLI, which works but loses the loop.

### Routes

```
GET  /auth/login                 — generate PKCE verifier+challenge + state nonce, 302 to kanidm /ui/oauth2/authorise
GET  /auth/callback              — verify state, POST /oauth2/token, set admin_session cookie, 302 to /
POST /auth/logout                — delete cookie, optionally revoke at kanidm
```

### Tasks

#### 6.1 OAuth2 client registration (one-time, documented)

In the README, document the bootstrap steps using kanidm CLI (or the admin UI itself once it works):

```bash
kanidm system oauth2 create-public kanidm-admin-ui "Kanidm Admin" https://admin.idm.example.com
kanidm system oauth2 update-scope-map kanidm-admin-ui idm_admins openid profile email groups
# PKCE is on by default for public clients
```

The `groups` scope gives us the user's group memberships in the ID token claim, which lets us pre-check the `idm_admins` membership before hitting `/v1/whoami`. We still call `whoami` as the canonical source — the claim is just a fast-path gate.

#### 6.2 Config keys

Add to `Config`:
- `oauth2_client_id: String` (e.g. `kanidm-admin-ui`)
- `oauth2_redirect_url: String` (the admin UI's `/auth/callback` URL)
- `cookie_signing_key_path: PathBuf` — 32-byte key for signing the short-lived PKCE state cookie. Auto-generate and persist on first start if missing.
- Rename `kanidm_session_cookie` to `admin_session_cookie` (default: `admin_session`) — different name from kanidm's `bearer` so a shared parent domain doesn't collide.

#### 6.3 New handler: `src/handlers/auth.rs`

- `GET /auth/login`:
  1. Generate PKCE `verifier` (random 64 bytes, base64url) and `challenge` (SHA256 of verifier, base64url).
  2. Generate `state` nonce.
  3. Store `{verifier, state, return_to}` in a signed, HttpOnly, 5-minute cookie (`auth_pending`).
  4. 302 to `{kanidm_url}/ui/oauth2/authorise?response_type=code&client_id={id}&redirect_uri={cb}&scope=openid+profile+email+groups&state={state}&code_challenge={challenge}&code_challenge_method=S256`.

- `GET /auth/callback?code=...&state=...`:
  1. Read `auth_pending`, verify state matches, delete it.
  2. POST `{kanidm_url}/oauth2/token` with `grant_type=authorization_code, code, redirect_uri, client_id, code_verifier`.
  3. Parse response: `access_token`, optional `refresh_token`, `expires_in`.
  4. Set `admin_session` cookie (HttpOnly, Secure, SameSite=Lax, expires=access expiry).
  5. If refresh_token present, set `admin_refresh` cookie (HttpOnly, Secure, SameSite=Strict, longer expiry).
  6. 302 to `return_to` (from `auth_pending`, default `/`).

- `POST /auth/logout`:
  1. Delete both cookies.
  2. Optionally POST to `{kanidm_url}/oauth2/token/revoke` if access token exists.
  3. 302 to `/auth/login` (which will immediately bounce them back to kanidm to log in again, which is fine).

#### 6.4 `AdminUser` extractor changes

- Change `cookie_name` from `bearer` to `admin_session_cookie` from config.
- On `auth_valid()` failure: try refresh token if present, retry, then succeed. If refresh fails, fall through to `Unauthenticated`.
- `AppError::Unauthenticated` rendering changes: instead of the current "paste your cookie" page, return 302 to `/auth/login?return_to={current_path}` for non-HTMX requests, or `HX-Redirect` header for HTMX requests.

#### 6.5 Background refresh

The token's `exp` is in the UAT payload (already decoded by `parse_uat_payload`). When `exp - now < 60s` and a refresh token is present, refresh inline at the start of `from_request_parts`. This keeps long-running browser sessions alive without the user noticing.

#### 6.6 Smoke test against homelab

1. Register the client in kanidm (CLI for bootstrap, or via the admin UI's own OAuth2 screen if available).
2. Set the three config keys.
3. Visit `/` in a cold browser. Confirm redirect to kanidm login.
4. Log in with password+TOTP, then passkey.
5. Confirm landing on `/`, all admin functions work.
6. Wait for token expiry (or force-expire by setting `expires_in` low at kanidm); confirm silent refresh.
7. Click logout, confirm cookie cleared and redirect works.

### Phase 6 deliverables

- README has a one-paragraph "First-time setup" section explaining the kanidm CLI commands.
- `/auth/login`, `/auth/callback`, `/auth/logout` all work.
- Cookie paste is no longer required for ANY user-facing flow.
- Background refresh keeps sessions alive past initial access token expiry.
- Logout from admin UI clears local cookie (kanidm session may persist; document that).

### Phase 6 verification

- Cold incognito → `/` → kanidm login → back to admin UI: works end-to-end.
- Same flow with passkey-only: works.
- Same flow with a non-idm_admins user: bounces with the existing `Forbidden` template, not a confusing "unauthenticated".
- After 1 hour idle, refresh on a page: works (refresh token kicks in).
- After revoking the OAuth2 client in kanidm: next request redirects to login.

### Risks / things to document

- **Bootstrap chicken-and-egg.** Registering the OAuth2 client requires already being authenticated to kanidm. The very first admin still uses CLI or upstream `/ui/admin`. Same as Grafana/Nextcloud setup.
- **Logout asymmetry.** Logging out of admin UI doesn't log you out of kanidm. Matches every other OIDC client's behavior; document it.
- **Cookie scope.** If admin UI and kanidm share a parent domain, kanidm's `bearer` cookie may also arrive on admin UI requests. Harmless because we don't read it, but worth knowing for debugging.
- **PKCE state cookie key rotation.** If the signing key changes, in-flight logins fail (state cookie won't verify). Acceptable; users just retry.

---

## Phase 7 — Login flow (optional, deferred)

If we decide to replace kanidm's login UI entirely (NOT what Phase 6 does — Phase 6 *uses* kanidm's login; this phase would *replace* it), screens 01–07 slot in:

- `01-login-username.html` → `/login` (username + passkey shortcut)
- `02-login-choose-mech.html` → `/login/mech` (pick mechanism)
- `03-login-password.html` → `/login/password`
- `04-login-totp.html` → `/login/totp`
- `05-login-backup-code.html` → `/login/backup-code`
- `06-login-passkey.html` → `/login/passkey`
- `07-login-denied.html` → `/login/denied`

Backend: walk kanidm's `auth_init`/`auth_step` state machine, set the `admin_session` cookie on success. This is a separate project — don't start until Phases 0–6 are stable. Realistically, **Phase 6 makes this unnecessary** for most use cases. Only do this if you also want to theme/customize the login UX itself.

---

## Cross-cutting patterns

### HTMX usage

- **Forms:** `hx-post="/...", hx-target="..."`. Server returns the updated fragment.
- **Modals:** `hx-get` into `#overlay-slot`, modal renders its own `.modal-backdrop`. Cancel does `hx-get="/empty"` to clear.
- **Toasts:** server returns `HX-Trigger: {"toast": {...}}`; client island renders.
- **Redirects:** server returns `HX-Redirect: /path` for "go elsewhere" responses.
- **Out-of-band swaps:** when a mutation should update two areas (e.g., row count badge + the row itself), use `hx-swap-oob="true"` on the secondary fragment.
- **History:** `hx-push-url="true"` on every navigation-style request (tab switches, list filters) so back button works.

### When to use a Preact island vs. HTMX

| Need | Approach |
|---|---|
| Form submit + show result | HTMX |
| Modal open/close | HTMX |
| Tab switching | HTMX with `hx-push-url` |
| Inline edit | HTMX |
| Toggle switches | HTMX with `hx-trigger="change"` |
| Optimistic delete with undo toast | HTMX + OOB swap |
| Multi-select with search inside | **Preact island** |
| Datetime picker with keyword shortcuts (now/never) | **Preact island** |
| Cmd+K palette | **Preact island** |
| Live-validating fields as you type | HTMX `hx-trigger="input changed delay:300ms"` for backend validation; Preact only if needed client-side |

Default to HTMX. Only reach for Preact when you genuinely need persistent client state.

### Template partials

- Partials live in the same folder as their parent and start with `_` (e.g., `templates/people/_row.html`).
- A partial is a self-contained Askama template with its own struct in the handler module.
- For HTMX swaps, the handler picks partial vs. full based on `HX-Request` header.

### Error handling

`AppError` variants and their HTTP responses:

| Variant | Status | Body |
|---|---|---|
| `Unauthenticated` | 401 | `templates/login_redirect.html` (or `HX-Redirect: /login` for HTMX requests after Phase 6) |
| `Forbidden` | 403 | `templates/forbidden.html` |
| `NotFound` | 404 | `templates/not_found.html` |
| `Kanidm(String)` | 502 | Friendly page + toast for HTMX |
| `Template(askama::Error)` | 500 | Friendly page |
| `Other(anyhow::Error)` | 500 | Friendly page (don't leak details) |

### Logging

`tracing` configured in `main.rs` with `EnvFilter`. Every handler emits `tracing::info!` on entry with the SPN. Errors emit `tracing::warn!` with context. Avoid `error!` for user-facing errors (forbidden, not found) — they're not server errors.

---

## Verification across phases

After each phase, run the verification recipe and check off the deliverables. Don't move to the next phase with broken acceptance criteria.

**Per-phase smoke test against your homelab kanidm:**
1. Stop bun dev / Rust dev.
2. `bun run build && cargo run`.
3. Authenticate:
   - **Pre-Phase-6:** copy your `bearer` cookie from the kanidm domain to `localhost:3000` in DevTools.
   - **Phase 6+:** visit `localhost:3000`, redirect bounces you to kanidm, log in normally.
4. Walk through the phase's deliverables one-by-one.
5. Compare side-by-side with the matching `design/*.html` file — pixel-close acceptance.

**End-of-project:** Phase 5 verification + run for a week against the homelab without falling back to CLI for any in-scope operation.
