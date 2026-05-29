# kanidm-admin-ui

Self-hosted admin panel for [kanidm](https://github.com/kanidm/kanidm). Server-rendered (Axum + Askama), Tailwind v4 styling, HTMX for interactivity, Preact islands for the few genuinely client-side bits (Cmd+K palette, etc.).

## Status

Phase 1 scaffold: dashboard renders against a real kanidm instance, auth via existing kanidm session cookie.

## Stack

- **Backend:** Axum 0.8 + Askama 0.16 + tower-http
- **Auth:** reuses kanidm session cookie; admin-group gate
- **API:** [`kanidm_client`](https://docs.rs/kanidm_client) crate (no manual REST)
- **Styling:** Tailwind v4 (CSS-first config)
- **Interactivity:** HTMX 2 + Preact 10 islands
- **Bundler:** Bun

## Prerequisites

- Rust 1.95+
- [Bun](https://bun.com) 1.3+
- A running kanidm instance you can hit

## First-time setup

```bash
cp kanidm-admin-ui.example.toml kanidm-admin-ui.toml
# edit kanidm-admin-ui.toml to point at your kanidm instance
bun install
```

## Dev loop

Two terminals:

1. **CSS + JS rebuild on file change:**
   ```bash
   bun run dev
   ```

2. **Rust server with auto-reload** (one-time: `cargo install cargo-watch`):
   ```bash
   cargo watch -x run
   ```

Browser: http://localhost:3000

You'll get an auth error if no kanidm session cookie is present. The simplest way to get one in dev: sign in to your kanidm web UI in the same browser, then come back. (For real deploys, run behind Traefik on the same domain — see deployment notes below.)

## Directory layout

```
src/                    Rust source
├── main.rs             entry + Axum router
├── config.rs           config (TOML + env)
├── auth.rs             KanidmClientFactory + AdminUser extractor
├── error.rs            AppError → HTTP response
└── handlers/           one file per route group
    ├── mod.rs          router composition
    ├── health.rs       /healthz
    └── dashboard.rs    / (dashboard)

templates/              Askama templates (HTML)
├── base.html           shell (sidebar + island mounts)
└── dashboard.html

islands/                Preact/TS islands for client-side bits
├── entry.ts            mounts islands onto their DOM nodes
└── command_palette.tsx Cmd+K palette

styles/
└── app.css             Tailwind v4 entry → /static/app.css

static/                 build output, .gitignored

design/                 raw HTML mockups from claude.ai/design
                        (drop them here, port into templates/ over time)
```

## Where to drop the design HTML

Put the HTML files from claude.ai/design into [`design/`](design/). They're visual reference — we extract structure + Tailwind classes into Askama templates one by one.

## Phase 1 scope (current)

- [x] Project scaffold
- [x] Kanidm client + admin-group auth gate
- [x] Dashboard route with real data (person/group/oauth2 counts + domain info)
- [x] Preact island mounting (Cmd+K stub)
- [ ] People list (next)
- [ ] People detail + tabs
- [ ] OAuth2 list + detail
- [ ] Groups list + detail
- [ ] Account policy editor

## Phase 2 (later, optional)

Replace kanidm's user-facing UI (login, apps portal, credential reset) under the same design system.

## License

MPL-2.0
