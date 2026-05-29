# Project guide

Self-hosted operator panel for [kanidm](https://github.com/kanidm/kanidm).
Server-rendered Rust (Axum + Askama) with HTMX, Tailwind v4, and a few
Preact islands. Talks to kanidm exclusively over the public `/v1/` REST
API. Deployable as a Docker container behind a reverse proxy on the same
domain as kanidm.

## Stack

| Layer | Choice | Why |
|---|---|---|
| HTTP server | **Axum 0.8** | Modern, ergonomic, great extractors |
| Templating | **Askama 0.16** + `askama_web` (axum-0.8) | Compile-checked HTML, no runtime template loader |
| Styling | **Tailwind v4** with shadcn/tweakcn-shaped tokens in `@theme` | Tokens become native Tailwind utilities; tweakcn can edit them online |
| Client interactivity | **HTMX 2** | 80% of interactions are partial swaps |
| Client islands | **Preact 10 + TypeScript**, bundled by **Bun** | Only for genuinely stateful widgets (Cmd+K, dropdowns, toasts, pagination) |
| Behaviors | tiny `data-*` registry under `islands/behaviors/` | Delegated DOM enhancement for things that aren't worth an island |
| Kanidm client | **`kanidm_client` + `kanidm_proto` 1.10** | Typed Rust client wraps `/v1/` REST API |
| Config | **figment** (TOML + `KANIDM_ADMIN_*` env) | Both file and env, env wins |

**Hard preference: never edit `Cargo.toml` manually. Always use `cargo add <crate> [--features ...]`.**

**Non-goals (deliberately out of scope):** POSIX attributes, service
accounts, recycle bin, system config, IDM sync, raw SCIM.

## Code style

- No comments unless explaining a non-obvious "why" (a hidden constraint, a workaround, an invariant). Identifiers should be self-documenting.
- No backwards-compat shims, no dead code, no half-finished implementations.
- No defensive checks for things internal code already guarantees.

## Bun for the frontend toolchain

The frontend bundle (CSS + JS) is built with Bun. The backend is Rust — it
is **not** a Bun.serve app, so the Bun.serve / bun:sqlite / Bun.redis
boilerplate does not apply here.

- `bun install` for dependencies, `bun run build` (CSS + JS bundles), `bun run dev` (watch).
- `bunx <package>` instead of `npx`.

## Authentication model

Started as session-cookie reuse (kanidm's `bearer` cookie sent to us on
the shared domain). That path is gone: we ship our own homegrown login
that talks to kanidm's `/v1/auth/*` endpoints. Reasons we own login:

- Single design system (no jarring jump to kanidm's login screen).
- Control over the WebAuthn challenge flow (passkey / security key auto-fire after HTMX swap).
- Reauth modal on session expiry — re-prompts in place without losing the page.

The `AdminUser` extractor checks admin-group membership against the
configured `admin_group` (defaults to `idm_admins`) before any `/admin/*`
route runs. `KanidmClientFactory::for_token(token)` builds a fresh
client per request with the user's token already set, so per-user
permissions and audit trails are preserved.

## Critical gotchas in the kanidm API

Surprises that hurt earlier attempts. Read these before touching the
data layer.

1. **Entries are `attrs: BTreeMap<String, Vec<String>>`.** Booleans encoded as strings (`"true"`/`"false"`), integers as strings, multivalued for everything. Use the helpers in `src/kanidm/entry.rs` (`attr_first`, `attr_all`, `attr_bool`, `attr_int`, `in_class`).
2. **`oauth2_allow_insecure_client_disable_pkce` is INVERTED.** `"true"` means PKCE is DISABLED. Render UI as "PKCE enabled" with the bit flipped.
3. **Account policy password attr is `auth_password_minimum_length`** (note the `auth_` prefix), not `password_minimum_length`.
4. **Pre-formatted scope map strings:** `oauth2_rs_scope_map` is `["groupname@spn: {\"scope1\", \"scope2\"}"]`. Parser in `src/kanidm/scope_map.rs`.
5. **Pre-formatted claim map strings:** `oauth2_rs_claim_map` is `["claim:group:joinchar:value1,value2"]`. Parser in `src/kanidm/claim_map.rs`. Join-char: space → ssv, semicolon → array. Strict — anything else is rejected.
6. **OAuth2 image fetch URL** is `/ui/images/oauth2/{client_name}`, keyed by name not hash. We proxy it.
7. **No "disable account policy" endpoint** — only per-field reset: `DELETE /v1/group/{id}/_attr/<attr>`.
8. **Several endpoints return structured proto types**, not flat attrs: `CredentialStatus`, `UatStatus`, `CUIntentToken`, `Option<String>` for basic secret. Prefer these over scraping entry attrs.
9. **`UatStatus` time fields** are Unix timestamps (integers), not RFC3339.
10. **Reset-token format** is short (`xkdzk-fr2p7-...`), not a JWT — display verbatim.
11. **Group membership identifiers** accept both SPN and UUID; backend normalizes.
12. **kanidm 409 responses** are often opaque blobs. We surface attribute-aware friendly messages (mail already in use, name taken, …) via `friendly_client_error` in `src/handlers/common.rs`, with the full original error logged at trace level.

## Cross-cutting patterns

- **Tab-as-URL.** Detail pages (person, group, oauth2) keep the current tab in the URL path (`/admin/people/:id/credentials`), not query string. HTMX swaps the tab body, the URL updates, refresh-safe.
- **HTMX OOB swaps** for tab nav: `TabsNavFragment` re-renders the nav strip alongside the main swap so the active-state stays correct.
- **`render_actions_cell`** collapses 0/1/many row actions into the right control (nothing / direct icon button / kebab+menu). Three matching partials own the markup; the Preact dropdown island handles the menu.
- **Macros vs struct-backed partials.** Pure-template fragments live in `templates/macros/{ui,forms,page}.html`. Anything with a real invariant (escaping, format branching) is a Rust struct under `src/views/partials.rs` with a matching `templates/partials/_*.html`. The rule of thumb for extracting something new: 3+ identical-shape markup chunks across templates.
- **`data-behavior` registry.** Delegated DOM handlers live in `islands/behaviors/`, keyed by data attribute. See `islands/behaviors/README.md` for the contract.
- **Friendly errors + rollback.** Multi-step kanidm operations (create person with mail + legalname) rollback on partial failure; errors preserve user input so the form re-renders with what they typed.

## Hard styling rule (Tailwind v4 + shadcn/tweakcn tokens)

All visual properties come from shadcn-shaped design tokens defined in
`styles/app.css` (so the theme can be tweaked verbatim with tweakcn online).
Light theme is `:root`; dark theme is activated by the `.dark` class on `<html>`.
Use only:

Baseline shadcn:

- `bg-background`, `text-foreground` (page chrome / body)
- `bg-card`, `bg-card-foreground` (surfaces / cards)
- `bg-popover`, `bg-popover-foreground` (elevated surfaces, menus, modals)
- `bg-accent`, `text-accent-foreground` (subtle hover/active surface — NOT the orange action color)
- `bg-primary`, `text-primary`, `text-primary-foreground` (the orange action color and its variants `bg-primary/90`, `bg-primary/80`)
- `bg-secondary`, `text-secondary-foreground` (secondary surface)
- `bg-muted`, `text-muted-foreground` (muted surface / muted text — use `text-muted-foreground/60` for disabled)
- `bg-destructive`, `text-destructive`, `border-destructive`, `text-destructive-foreground`
- `border-border`, `border-input` (single + slightly stronger border)
- `ring-ring`, `outline-ring/50` (focus rings)

Extensions (kept on top of shadcn baseline):

- `bg-primary-soft` (low-alpha primary background)
- `text-link`, `bg-link-soft`
- `bg-success`, `text-success`, `border-success`, `bg-success-soft`, `text-success-foreground`
- `bg-warning`, `text-warning`, `border-warning`, `bg-warning-soft`, `text-warning-foreground`
- `bg-destructive-soft`
- `bg-info`, `text-info`, `border-info`, `bg-info-soft`, `text-info-foreground`
- `bg-code-bg`, `bg-token-bg`, `text-mono-chip`
- `accent-primary` (CSS `accent-color` utility, for native form controls)
- `shadow-primary-ring` (subtle ring around primary-color dots)

Radius (driven by `--radius`):

- `rounded-sm` (`--radius` − 4px)
- `rounded-md` (`--radius` − 2px)
- `rounded-lg` (`--radius`)
- `rounded-xl` (`--radius` + 4px)
- `rounded-pill` (999px — extension)

Shadows: `shadow-sm`, `shadow`, `shadow-md`, `shadow-lg`, `shadow-xl`, `shadow-2xl`.

Fonts: `font-sans`, `font-mono`.

NEVER use:

- Raw Tailwind palette colors: `bg-zinc-900`, `text-gray-500`, etc.
- Raw hex values in `style="..."` attributes for colors
- `bg-(--var)` arbitrary-value escape hatches
- Inline color declarations
- The OLD vocabulary (`bg-canvas`, `bg-surface`, `bg-elevated`, `bg-hover`,
  `bg-active`, `text-tertiary`, `text-disabled`, `border-subtle`,
  `border-default`, `border-strong`, `bg-danger`, `shadow-card`, etc.) —
  all migrated, do not reintroduce.

If a template needs a color or radius not in app.css, ADD IT to app.css
as a new var on both `:root` and `.dark` and mirror it in `@theme inline` —
do not work around it.
