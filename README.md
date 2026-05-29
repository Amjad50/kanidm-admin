# kanidm-admin-ui

A polished, self-hosted **admin panel for [kanidm](https://github.com/kanidm/kanidm)**. Designed to fill the gap kanidm's built-in `/ui/admin` leaves: full CRUD for people, groups, OAuth2 clients, account policy, sessions, and a clean homegrown login flow.

Server-rendered (no SPA), feels as snappy as kanidm itself. Talks to kanidm exclusively over its public `/v1/` REST API — **zero fork maintenance**, drop it next to your existing instance.

## What you get at `/admin`

- **People** — list / create / edit / delete, credentials & reset-link, SSH keys, RADIUS secret, sessions, validity windows.
- **Groups** — list / create / edit / delete, members, account policy.
- **OAuth2 clients** — list / create / delete, plus per-client tabs for general settings, secret (reveal / regenerate), scope maps, claim maps, signing keys (rotate / revoke), client image, advanced settings (refresh-token TTL, device flow).
- **Self** — `/me` and `/me/sessions` for any logged-in user.
- **Homegrown login** at `/login` with password / TOTP / passkey / security-key support, plus a reauth modal that re-prompts in place when your session expires.
- **Cmd+K palette** to jump anywhere fast.

Anyone in your `idm_admins` group (configurable) can reach `/admin/*`. Everyone else can still use `/login`, `/me`, and `/me/sessions`.

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

You already run kanidm behind a reverse proxy. Add this admin UI as a second backend and route the admin paths to it.

**1. Write a minimal `kanidm-admin-ui.toml`** next to your compose file:

```toml
bind_addr = "0.0.0.0:3000"

# Where to reach your kanidm server. Must be HTTPS.
kanidm_url = "https://idm.example.com"

# Group whose members can access /admin/*.
admin_group = "idm_admins"
```

See [`kanidm-admin-ui.example.toml`](kanidm-admin-ui.example.toml) for all options (CA cert, accept-invalid-certs for dev, session cookie name, etc.).

**2. Add the admin UI as a service** in the same `docker-compose.yml` as your kanidm + reverse proxy. The UI takes ownership of `/admin/*`, `/login*`, `/logout`, `/me*`, `/healthz`, `/empty`, and `/static/*` — everything else stays on kanidm.

#### Traefik

```yaml
services:
  admin-ui:
    image: ghcr.io/amjad50/kanidm-admin-ui:latest
    restart: unless-stopped
    volumes:
      - ./kanidm-admin-ui.toml:/app/kanidm-admin-ui.toml:ro
    labels:
      - "traefik.enable=true"
      # Routes the admin UI owns. Priority must beat the kanidm router
      # so these specific paths win over a kanidm catch-all on the same host.
      - "traefik.http.routers.admin-ui.rule=Host(`idm.example.com`) && (PathPrefix(`/admin`) || PathPrefix(`/login`) || Path(`/logout`) || PathPrefix(`/me`) || Path(`/healthz`) || Path(`/empty`) || PathPrefix(`/static`))"
      - "traefik.http.routers.admin-ui.entrypoints=websecure"
      - "traefik.http.routers.admin-ui.tls=true"
      - "traefik.http.routers.admin-ui.tls.certresolver=letsencrypt"
      - "traefik.http.routers.admin-ui.priority=100"
      - "traefik.http.services.admin-ui.loadbalancer.server.port=3000"
    networks:
      - kanidm-net

networks:
  kanidm-net:
    external: true
```

Your existing kanidm router stays as-is — Traefik's path-priority routing sends `/admin/*` and friends to this service and everything else to kanidm.

#### Caddy

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

Then in your `Caddyfile`:

```caddyfile
idm.example.com {
    @ui_paths {
        path /admin /admin/* /login /login/* /logout /me /me/* /healthz /empty /static/*
    }
    handle @ui_paths {
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

In your nginx server block for `idm.example.com`:

```nginx
location ~ ^/(admin|login|logout|me|healthz|empty|static)(/|$) {
    proxy_pass http://admin-ui:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

**3. Bring it up.**

```bash
docker compose up -d admin-ui
```

**4. Open `https://idm.example.com/admin`** and log in with an `idm_admins` account.

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

`GET /healthz` returns `200 OK` with body `ok`. Use it for container/orchestrator health probes.

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
