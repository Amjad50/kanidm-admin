#!/usr/bin/env bash
# One-shot setup for the kanidm-admin-ui docker-compose stack.
#
# What this does:
#   1. (init) Currently a no-op stub. Master doesn't have OAuth2 consent
#      yet, so no cookie signing key needs to be generated. When OAuth2
#      consent lands, this will generate a key into .env that
#      docker-compose reads automatically.
#   2. (passwords) Recovers the admin and idm_admin passwords by running
#      `kanidmd recover-account` inside the kanidm container and prints
#      them.
#
# Usage:
#   ./bootstrap.sh init        — currently no-op (kept for forward compat)
#   ./bootstrap.sh passwords   — recover admin passwords (run AFTER stack is up)
#   ./bootstrap.sh             — runs init if no .env, otherwise passwords

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd -P)"
cd "$SCRIPT_DIR"

ENV_FILE="$SCRIPT_DIR/.env"

generate_signing_key() {
    # Cookie signing key is not yet needed in master (it arrives with the
    # OAuth2 consent feature in a later port). This is a no-op stub kept
    # so the `init` subcommand remains a useful future hook.
    echo "init: nothing to set up yet — cookie signing key will be wired up when OAuth2 consent lands."
}

recover_passwords() {
    if ! docker compose ps --status running kanidm | grep -q kanidm; then
        echo "ERROR: kanidm container is not running. Start the stack first:" >&2
        echo "       docker compose up -d" >&2
        exit 1
    fi

    echo
    echo "→ Recovering admin password (the kanidm system admin)…"
    local admin_pw
    admin_pw="$(docker compose exec -T kanidm \
        kanidmd recover-account -c /data/server.toml admin 2>/dev/null \
        | awk -F'"' '/new_password/ {print $2; exit}')"

    if [ -z "$admin_pw" ]; then
        echo "ERROR: could not parse admin password from kanidmd output. Try:" >&2
        echo "       docker compose exec kanidm kanidmd recover-account -c /data/server.toml admin" >&2
        exit 1
    fi

    echo "→ Recovering idm_admin password (the kanidm IDM administrator — use this one to log into the admin UI)…"
    local idm_admin_pw
    idm_admin_pw="$(docker compose exec -T kanidm \
        kanidmd recover-account -c /data/server.toml idm_admin 2>/dev/null \
        | awk -F'"' '/new_password/ {print $2; exit}')"

    if [ -z "$idm_admin_pw" ]; then
        echo "ERROR: could not parse idm_admin password from kanidmd output." >&2
        exit 1
    fi

    cat <<EOF

═══════════════════════════════════════════════════════════════════════════════
  Initial admin passwords — copy these somewhere safe; this is the ONLY time
  bootstrap.sh prints them. You can always re-run \`bootstrap.sh passwords\`
  to regenerate them (which will INVALIDATE the previous ones).
═══════════════════════════════════════════════════════════════════════════════

  admin        : $admin_pw
       └── System-level. Use the kanidm CLI ('kanidm system …') with this.
            Almost never needed through the web UI.

  idm_admin    : $idm_admin_pw
       └── IDM administrator. ★ Use this one to log into the web UI at
            https://idm.localhost/

═══════════════════════════════════════════════════════════════════════════════

  Next steps:

  1) Trust Caddy's internal CA so your browser doesn't warn:
        docker compose exec caddy cat /data/caddy/pki/authorities/local/root.crt \\
            | sudo tee /usr/local/share/ca-certificates/caddy-local-root.crt
        sudo update-ca-certificates
     (or visit https://idm.localhost/ once, click through the warning, then
      'Always trust this certificate' in your browser's cert dialog.)

  2) Open https://idm.localhost/ in a browser. Log in as 'idm_admin' with the
     password above.

  3) Create yourself a real user account from the admin panel (or via CLI:
        docker compose exec kanidm kanidm person create <your_username> "<Display Name>" \\
            --name idm_admin
     and then add it to the idm_admins group if you want admin powers).

EOF
}

# ── Dispatch ──────────────────────────────────────────────────────────────

cmd="${1:-auto}"

case "$cmd" in
    init)
        generate_signing_key
        echo
        echo "Next: run \`docker compose up -d\`, then \`./bootstrap.sh passwords\`."
        ;;
    passwords)
        recover_passwords
        ;;
    auto)
        if [ ! -f "$ENV_FILE" ] || ! grep -q '^COOKIE_SIGNING_KEY=' "$ENV_FILE"; then
            generate_signing_key
            echo
            echo "Now bring the stack up:"
            echo "    docker compose up -d"
            echo "Then re-run this script:"
            echo "    ./bootstrap.sh passwords"
        else
            recover_passwords
        fi
        ;;
    *)
        echo "Usage: $0 [init|passwords]" >&2
        exit 1
        ;;
esac
