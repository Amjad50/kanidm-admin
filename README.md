# kanidm-admin-ui

A polished, self-hosted **admin panel for [kanidm](https://github.com/kanidm/kanidm)**. Designed to fill the gap kanidm's built-in `/ui/admin` leaves: full CRUD for people, groups, OAuth2 clients, account policy, sessions, and a clean homegrown login flow.

Server-rendered (no SPA), feels as snappy as kanidm itself. Talks to kanidm exclusively over its public `/v1/` REST API — **zero fork maintenance**, drop it next to your existing instance.

## What you get at `/admin`

- **People** — list / create / edit / delete, credentials & reset-link, SSH keys, RADIUS secret, sessions, validity windows.
- **Groups** — list / create / edit / delete, members, account policy.
- **OAuth2 clients** — list / create / delete, plus per-client tabs for general settings, secret (reveal / regenerate), scope maps, claim maps, signing keys (rotate / revoke), client image, advanced settings (refresh-token TTL, device flow).
- **Self** — `/admin/me` and `/admin/me/sessions` for any logged-in user.
- **Homegrown login** at `/admin/login` (password / TOTP / passkey / security-key), plus a reauth modal that re-prompts in place when your session expires.
- **Cmd+K palette** to jump anywhere fast.

The whole app lives under `/admin/*` — pages, login, /me, healthz, static assets, everything. Anyone in your `idm_admins` group (configurable) can reach the operator pages; everyone else can still use `/admin/login`, `/admin/me`, and `/admin/me/sessions`.

Visiting `/` is harmless: the app responds with a permanent redirect to `/admin`, so a dedicated subdomain (`admin.idm.example.com/`) lands users on the dashboard automatically.

## Install

Pre-built multi-arch images (amd64 + arm64) are published to GHCR.

| Tag | What it points at | When to use |
|---|---|---|
| `:latest` | The most recent semver release | Default for stable installs |
| `:v0.1.0`, `:0.1`, `:0` | A specific release / minor / major | Pin to a version for predictable upgrades |
| `:edge` | Tip of `master` after CI passes | Bleeding-edge / preview |
| `:master-<short-sha>` | Exact reproducible build | Rollback or debugging |

Full image: `ghcr.io/amjad50/kanidm-admin-ui:<tag>`.

Two install paths — pick the one that fits.

### Path A: drop in next to your existing kanidm

You already run kanidm behind a reverse proxy. Add this admin UI as a second backend.

**1. Write a minimal `kanidm-admin-ui.toml`** next to your compose file:

```toml
bind_addr = "0.0.0.0:3000"

# Where to reach your kanidm server. Must be HTTPS.
kanidm_url = "https://idm.example.com"

# Group whose members can access /admin/*.
admin_group = "idm_admins"
```

See [`kanidm-admin-ui.example.toml`](kanidm-admin-ui.example.toml) for all options (CA cert, accept-invalid-certs for dev, session cookie name, etc.).

**2. Add the admin UI as a compose service.** Same definition either way:

```yaml
services:
  admin-ui:
    image: ghcr.io/amjad50/kanidm-admin-ui:latest
    restart: unless-stopped
    volumes:
      - ./kanidm-admin-ui.toml:/app/kanidm-admin-ui.toml:ro
    networks:
      - kanidm-net

networks:
  kanidm-net:
    external: true
```

**3. Route to it.** Two styles — pick one:

- **Subdomain** (`admin.idm.example.com`) — clean separation, one rule per backend, easy to lock down independently.
- **Path-split** (`idm.example.com/admin`) — keep one hostname; admin UI owns `/admin/*` and kanidm owns the rest.

Snippets for both, in the three common reverse proxies:

#### Traefik

Subdomain — add to the `admin-ui` service's `labels:`
```yaml
- "traefik.enable=true"
- "traefik.http.routers.admin-ui.rule=Host(`admin.idm.example.com`)"
- "traefik.http.routers.admin-ui.entrypoints=websecure"
- "traefik.http.routers.admin-ui.tls.certresolver=letsencrypt"
- "traefik.http.services.admin-ui.loadbalancer.server.port=3000"
```

Path-split — same but match on `/admin*` instead of a hostname:
```yaml
- "traefik.enable=true"
- "traefik.http.routers.admin-ui.rule=Host(`idm.example.com`) && PathPrefix(`/admin`)"
- "traefik.http.routers.admin-ui.entrypoints=websecure"
- "traefik.http.routers.admin-ui.tls.certresolver=letsencrypt"
- "traefik.http.routers.admin-ui.priority=100"
- "traefik.http.services.admin-ui.loadbalancer.server.port=3000"
```

Priority 100 ensures the path match beats any kanidm catch-all router on the same host.

#### Caddy

Subdomain:
```caddyfile
admin.idm.example.com {
    reverse_proxy admin-ui:3000
}
```

Path-split — add to your existing kanidm host block:
```caddyfile
idm.example.com {
    handle /admin* {
        reverse_proxy admin-ui:3000
    }
    handle {
        reverse_proxy https://kanidm:8443 {
            transport http { tls }
        }
    }
}
```

#### Nginx

Subdomain — a new `server` block:
```nginx
server {
    listen 443 ssl http2;
    server_name admin.idm.example.com;
    # ... your TLS config ...

    location / {
        proxy_pass http://admin-ui:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Path-split — add inside your existing `server` block for `idm.example.com`:
```nginx
location /admin {
    proxy_pass http://admin-ui:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

**4. Bring it up and open it.**

```bash
docker compose up -d admin-ui
```

Then open `https://admin.idm.example.com/` (subdomain) or `https://idm.example.com/admin` (path-split). Log in with any `idm_admins` account.

### Path B: full stack with docker-compose (fresh setup)

For a clean local install — kanidm + this admin UI + Caddy in front, all wired up. Useful for kicking the tires or as a starting point.

```bash
git clone https://github.com/Amjad50/kanidm-admin.git
cd kanidm-admin/deploy
docker compose up -d
./bootstrap.sh passwords   # prints the initial idm_admin password
```

Then trust Caddy's internal CA and browse to **https://idm.localhost/admin**. The [`deploy/SETUP.md`](deploy/SETUP.md) walkthrough has the cert-trust details and a production checklist.

## Configuration

All keys can be set in `kanidm-admin-ui.toml` or via environment variables prefixed with `KANIDM_ADMIN_` (env wins).

| Key | Type | Default | What it does |
|---|---|---|---|
| `bind_addr` | string | `127.0.0.1:3000` | TCP address to listen on |
| `kanidm_url` | string | *required* | URL of your kanidm server (HTTPS) |
| `kanidm_ca_path` | string | unset | Path to a CA cert if kanidm uses a private CA |
| `kanidm_accept_invalid_certs` | bool | `false` | **Dev only.** Skip cert verification for kanidm |
| `admin_group` | string | `idm_admins` | Group whose members can reach `/admin/*` |
| `session_cookie_name` | string | `kanidm_admin_session` | Cookie name; distinct from kanidm's own `bearer` so they don't collide on a shared parent domain |
| `static_dir` | string | `static` | Where the bundled CSS/JS live (set to `/app/static` inside the official image — it's already the container default) |
| `dev_insecure_cookies` | bool | `false` | **Dev only.** Drop `Secure` flag for http:// localhost |

Env example (in your `docker-compose.yml`, in lieu of the TOML mount):

```yaml
services:
  admin-ui:
    image: ghcr.io/amjad50/kanidm-admin-ui:latest
    environment:
      KANIDM_ADMIN_KANIDM_URL: https://idm.example.com
      KANIDM_ADMIN_ADMIN_GROUP: admin_panel_users
```

## Updating

```bash
docker compose pull admin-ui
docker compose up -d admin-ui
```

Pin to a version tag (e.g. `image: ghcr.io/amjad50/kanidm-admin-ui:v0.1.0`) if you want predictable upgrades.

## Health check

`GET /admin/healthz` returns `200 OK` with body `ok`. Use it for container/orchestrator health probes.

## Compatibility

| This | Pairs with |
|---|---|
| `v0.1.x` | `kanidm/server:1.10.0` |

The `kanidm_client` Rust crate is pinned to kanidm `1.10.x`. Mixing major versions will fail at startup with a version-mismatch error.

## Development

Contributors:

```bash
git clone https://github.com/Amjad50/kanidm-admin.git
cd kanidm-admin
cp kanidm-admin-ui.example.toml kanidm-admin-ui.toml   # point at your kanidm
bun install
bun run dev          # CSS + JS watch
cargo watch -x run   # in another terminal (cargo install cargo-watch)
```

Browser at http://localhost:3000/admin.

Architectural choices, kanidm-API gotchas, cross-cutting patterns, and styling rules are all in [CLAUDE.md](CLAUDE.md) — it's the single doc that explains *why* the code looks the way it does.

CI runs `cargo fmt`, `clippy`, and `cargo test` on every PR. The Docker image is built and pushed to GHCR after CI passes on `master` (as `edge`) and on every `v*` tag (as `vX.Y.Z` / `latest`).

## License

MPL-2.0
