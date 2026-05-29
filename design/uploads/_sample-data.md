# Sample Data — Single Source of Truth

Every screen brief uses these exact names, UUIDs, and values. Do not invent new examples. If you need an additional sample, add it here first, then reference it.

The domain name is **`idm.example.com`** everywhere.

---

## People

| Name (login) | Display name | Legal name | Primary mail | Secondary mail | UUID | Groups |
|---|---|---|---|---|---|---|
| `alice.smith` | Alice Smith | Alice Marion Smith | `alice.smith@example.com` | `alice@example.com` | `7c3a8b4e-2f1d-4c5e-9a8b-1f2e3d4c5b6a` | developers, idm_admins, vpn_users |
| `bob.jones` | Bob Jones | Robert Jones | `bob.jones@example.com` | — | `9d4b7c2a-3e8f-4d1b-8a6c-5e9f7d2b4c1e` | developers, vpn_users |
| `eve.taylor` | Eve Taylor | Evelyn Taylor | `eve.taylor@example.com` | — | `2a8c5e1b-6f3d-4a9c-b7e2-8d1c3a5b7f9e` | developers, on_call |
| `carol.nguyen` | Carol Nguyen | Carol Linh Nguyen | `carol.nguyen@example.com` | — | `5b1f9d3a-4c8e-4b7d-a2f6-9c1e4b8d2a5f` | devops, on_call, vpn_users |
| `admin` | System Administrator | — | `admin@example.com` | — | `00000000-0000-0000-0000-000000000000` | idm_admins, system_admins |

SPNs (used in lists, breadcrumbs, headers): `alice.smith@idm.example.com`, etc.

## Pending invitations / expired accounts (for state examples)
- `dave.locked@idm.example.com` — account expired 3 days ago
- `frank.future@idm.example.com` — valid-from is 2 weeks in the future
- `jane.doe@idm.example.com` — newly created, no credentials yet

## Extra sample people (for picker / autocomplete examples only)

These are non-members in any group sample — used for "Add members" pickers and similar typeahead scenarios:
- `jane.doe@idm.example.com` — Jane Doe
- `paul.kim@idm.example.com` — Paul Kim
- `rita.shah@idm.example.com` — Rita Shah
- `sam.lopez@idm.example.com` — Sam Lopez
- `thomas.li@idm.example.com` — Thomas Li

---

## Groups

| Name | Description | Member count | Entry-managed-by | Mail | Account policy enabled? |
|---|---|---|---|---|---|
| `idm_admins` | Identity management administrators with full system access | 2 | `system_admins` | `admins@example.com` | yes |
| `developers` | Software development team — code repository and dev OAuth2 access | 24 | `idm_admins` | `dev@example.com` | yes |
| `devops` | Infrastructure and platform operations | 6 | `idm_admins` | `devops@example.com` | yes |
| `vpn_users` | Granted WireGuard / OpenVPN remote access | 31 | `devops` | — | no |
| `on_call` | Engineers currently in the on-call rotation | 4 | `devops` | `oncall@example.com` | yes |
| `system_admins` | Root-level kanidm administrators (used as entry-managed-by for other admin groups) | 1 | `system_admins` | — | yes |

### Sample account policy values (for `idm_admins`)
- credential-type-minimum: `passkey`
- password-minimum-length: `16`
- auth-session-expiry: `3600` seconds (1 hour)
- privilege-session-expiry: `900` seconds (15 minutes)
- webauthn-attestation-ca-list: configured (uploaded `fido_metadata.json`)
- limit-search-max-results: `1000`
- limit-search-max-filter-test: `1500`
- allow-primary-cred-fallback: `false`

### Sample account policy values (for `developers`)
- credential-type-minimum: `mfa`
- password-minimum-length: `12`
- auth-session-expiry: `28800` (8 hours)
- privilege-session-expiry: `1800` (30 minutes)
- webauthn-attestation-ca-list: not set
- limit-search-max-results: default (1000)
- limit-search-max-filter-test: default (1500)
- allow-primary-cred-fallback: `true`

---

## OAuth2 / OIDC Applications

| Name (system ID) | Display name | Type | Landing URL | Supplementary redirects | UUID | Has image? |
|---|---|---|---|---|---|---|
| `grafana` | Grafana | basic (confidential) | `https://grafana.example.com` | `https://grafana.example.com/login/generic_oauth` | `3f8a2c1d-7b4e-4f9a-9c2e-1d8b5e3a7c4f` | yes (uploaded `grafana.svg`) |
| `nextcloud` | Nextcloud | basic | `https://cloud.example.com` | `https://cloud.example.com/apps/user_oidc/code` | `8d2e6a4b-9f3c-4d7e-a1b5-6c8f4d2a9e3b` | yes (uploaded `nextcloud.png`) |
| `gitea` | Gitea | basic | `https://git.example.com` | `https://git.example.com/user/oauth2/kanidm/callback` | `1a7c3b8d-5e9f-4a2c-b6d4-7e3f1a8c5b9d` | yes |
| `vaultwarden` | Vaultwarden | basic | `https://vault.example.com` | `https://vault.example.com/identity/connect/oidc-signin` | `6e2a9c4f-1b8d-4e7a-c3f5-9d2b6e4a8c1f` | no |
| `homelab-spa` | Homelab Dashboard (SPA) | public | `https://dash.example.com` | — | `4b9f7a2e-3c8d-4b1f-a6e9-2c4d7b8f1a3e` | no |
| `cli-deploy-tool` | Deploy CLI | public | `https://deploy.example.com/auth/callback` | — | `9c5b1d8a-7e3f-4c9b-a2d6-8f1e5c3b7a4d` | no |

### Sample scope map (for `grafana`)
| Group | Scopes |
|---|---|
| `idm_admins` | `openid`, `profile`, `email`, `groups` |
| `developers` | `openid`, `profile`, `email`, `groups` |
| `vpn_users` | `openid`, `email` |

Supplementary scope map (for `grafana`):
| Group | Scopes |
|---|---|
| `idm_admins` | `grafana_admin` (custom) |

### Sample claim map (for `nextcloud`)
| Claim name | Group | Values | Join |
|---|---|---|---|
| `nextcloud_quota` | `developers` | `50GB` | `array` |
| `nextcloud_quota` | `idm_admins` | `unlimited` | `array` |
| `department` | `developers` | `Engineering`, `Product` | `csv` |
| `department` | `devops` | `Engineering`, `Infrastructure` | `csv` |

### Sample cryptographic key state (for `grafana`)
- `key-7f3a2c1d` — created 2026-01-12, status: **Active** (current)
- `key-2b8e5d4a` — created 2025-08-04, status: **Rotated** (still validates old tokens)
- `key-9c1f3e7b` — created 2025-02-19, status: **Revoked** (manual revocation)

### Sample OAuth2 toggle states (for `grafana`)
- PKCE: enabled
- Legacy crypto: disabled
- Prefer-short-username: disabled (uses SPN)
- Strict redirect URL: enabled
- Localhost redirects: disabled
- Consent prompt: enabled

### Sample refresh token expiry
- `grafana`: `2592000` seconds (30 days)
- `vaultwarden`: default (blank)

---

## Sample SSH keys (for `alice.smith`)

| Label | Public key (excerpt) | Fingerprint (SHA256) | Added |
|---|---|---|---|
| `laptop_ed25519` | `ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIK7...` | `SHA256:4FZJYr...` | 2025-11-03 |
| `workstation_rsa` | `ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAB...` | `SHA256:bxQ8mF...` | 2025-06-21 |
| `yubikey_5c` | `ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoY...` | `SHA256:Vk2P9N...` | 2025-03-15 |

---

## Sample sessions (for `alice.smith`)

| Session ID | Issued | Expires | Purpose | Source |
|---|---|---|---|---|
| `a4c2e8f1-...` | 2026-05-14 09:22 | 2026-05-14 17:22 | read-write | this device |
| `b8d3f9a2-...` | 2026-05-13 14:08 | 2026-05-13 22:08 | read-write | another device |
| `c1e4a7b8-...` | 2026-05-10 11:15 | never | read-only | api token (CI) |

---

## Sample credential status

For `alice.smith`:
- **Primary:** Password + TOTP (TOTP labeled "Personal Phone")
- **Passkeys:** 2 registered (`MacBook Pro Touch ID`, `YubiKey 5C NFC`)
- **Attested passkeys:** none
- **Unix password:** not set
- **SSH keys:** 3 (see above)
- **Backup codes:** 6 remaining (of 8 generated)

For `bob.jones`:
- **Primary:** Password only
- **Passkeys:** 0
- **SSH keys:** 1 (`laptop_main`)

For `jane.doe` (new):
- **Primary:** not set
- **Passkeys:** 0
- **SSH keys:** 0
- Status banner: "No credentials configured yet"

---

## Sample reset link (intent token result)

```
URL:    https://idm.example.com/ui/reset?token=eyJhbGciOiJFUzI1NiIs...
Expires: 2026-05-14 17:22:00 UTC (in 1 hour)
```

Display in UI: masked until reveal, QR code rendering of the full URL, copy-to-clipboard button.

---

## Sample RADIUS secret (for `bob.jones`)

```
xK8mP2qF9vN4jH7tR1yC6wA3eL5sB0gD
```

(Always shown once on generation, then masked.)

---

## Sample domain info

- **Domain name:** `idm.example.com`
- **Display name:** `Example Organization IDM`
- **UUID:** `00000000-0000-0000-0000-ffff00000000`
- **Functional level:** `8`
- **LDAP base DN:** `dc=idm,dc=example,dc=com`

---

## Sample dashboard counts

- Persons: **127**
- Groups: **18**
- OAuth2 applications: **6**
- (Current session) Signed in as `admin@idm.example.com`, expires in 38 minutes

---

## Date / time format conventions

- Absolute display: `2026-05-14 14:08 UTC`
- Relative display in lists: `2 hours ago`, `3 days ago`, `in 1 hour`
- ISO RFC3339 used in form inputs and tokens: `2026-05-14T17:22:00+00:00`
- Validity keywords (per CLI): `now`, `never`, `clear`, `any`, `epoch`

---

## Tone notes for copy

- Use "person" not "user" in user-facing copy (kanidm's preferred term).
- Use "application" or "client" interchangeably for OAuth2 — pick one per screen and be consistent.
- "Sign in" not "Log in"; "Sign out" not "Log out".
- Destructive verbs are direct: "Delete", "Revoke", "Destroy session" (not "Remove" or "End").
