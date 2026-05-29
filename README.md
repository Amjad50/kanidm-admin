# kanidm-admin-ui

Self-hosted admin panel for [kanidm](https://github.com/kanidm/kanidm). Server-rendered (Axum + Askama), Tailwind v4 styling, HTMX for interactivity, Preact islands for the few genuinely client-side bits (Cmd+K palette, dropdowns, toasts).

## Status

Operator panel is feature-complete: people, groups, OAuth2 clients, account policy, self profile + sessions, and a homegrown login flow (password / TOTP / passkey / security key). Ships as a Docker image; the [`deploy/`](deploy/) directory has a docker-compose stack with Caddy in front of both kanidm and the admin UI.

## Stack

- **Backend:** Axum 0.8 + Askama 0.16 + tower-http
- **Auth:** homegrown login (talks to kanidm `/v1/auth`); admin-group gate on `/admin/*`
- **API:** [`kanidm_client`](https://docs.rs/kanidm_client) crate (no manual REST)
- **Styling:** Tailwind v4 (CSS-first config, shadcn/tweakcn-shaped tokens)
- **Interactivity:** HTMX 2 + Preact 10 islands + a tiny `data-behavior` registry
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
bun run build   # builds CSS + JS bundles into static/
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

Browser: http://localhost:3000 (login page) → http://localhost:3000/admin (dashboard).

## Routes

- `/login`, `/login/*` — homegrown login flow
- `/logout`
- `/me`, `/me/sessions` — self profile + session management
- `/admin` — dashboard
- `/admin/people`, `/admin/people/:id/*` — list, detail (overview / credentials / SSH / RADIUS / sessions / groups / validity), create, edit, delete
- `/admin/groups`, `/admin/groups/:id/*` — list, detail (overview / members / account policy), create, edit, delete
- `/admin/oauth2`, `/admin/oauth2/:id/*` — list, detail (overview / secret / scope maps / claim maps / crypto / image / advanced), create, delete
- `/healthz` — liveness probe

## Directory layout

```
src/                       Rust source
├── main.rs                Axum entry + router composition
├── config.rs              TOML + env config (figment)
├── error.rs               AppError → HTTP response
├── auth/                  KanidmClientFactory, AdminUser extractor, pending-auth stash
├── kanidm/                kanidm-specific helpers (entry, ssh, policy, scope/claim map, key state)
├── handlers/              one module per route group
│   ├── mod.rs             router composition
│   ├── common.rs          shared error/lookup helpers
│   ├── dashboard.rs       /admin
│   ├── login/             /login, /login/password, /login/totp, /login/passkey, /login/securitykey
│   ├── session.rs         /logout
│   ├── self_user/         /me, /me/sessions
│   ├── people/            /admin/people/*
│   ├── groups/            /admin/groups/*
│   ├── oauth2/            /admin/oauth2/*
│   ├── empty.rs           /empty (HTMX overlay clear)
│   └── health.rs          /healthz
└── views/                 reusable view structs (pagination, dropdown, toast, icons, partials, time)

templates/                 Askama templates
├── base.html              app shell (sidebar + topbar + island mounts)
├── dashboard.html
├── error_*.html           404 / 403 / 500
├── macros/                ui::, forms::, page:: macro libraries
├── partials/              struct-backed shared partials (modal, pagination, …)
├── login/, self_user/, sessions/
└── people/, groups/, oauth2/

islands/                   Preact/TS islands + behaviors
├── entry.ts               mounts islands + behaviors
├── command_palette.tsx    Cmd+K
├── dropdown.tsx           data-dropdown JSON contract
├── pagination.tsx
├── toast.tsx
├── lib/                   shared client utilities (base64url, etc.)
└── behaviors/             delegated DOM enhancements (see behaviors/README.md)
                           — includes copy, row-href, reveal-secret, webauthn-login, …

styles/app.css             Tailwind v4 entry → /static/app.css

static/                    build output (app.css/app.js gitignored; logos + favicon tracked)

deploy/                    docker-compose stack (Caddy + kanidm + admin-ui)
```

Architectural choices, kanidm-API gotchas, cross-cutting patterns, and styling rules are all in [CLAUDE.md](CLAUDE.md).

## Deployment

See [`deploy/SETUP.md`](deploy/SETUP.md) for the docker-compose stack. The [`Dockerfile`](Dockerfile) is multi-stage (Bun → cargo → distroless-style runtime).

## License

MPL-2.0
