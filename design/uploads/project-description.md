# Kanidm Admin UI — Project Description

This is the universal product brief. It pairs with **one** design-system file (`design-system-linear.md`, `design-system-cloudflare.md`, or `design-system-stripe.md`) and **all** screen briefs in `screens/`. Sample data shared across every screen lives in `_sample-data.md` — use those exact names, UUIDs, and values.

---

## 1. Product

**Kanidm** is a self-hosted identity management server written in Rust. It provides authentication and identity for everything a small organization or self-hoster needs: SSO via OAuth2/OIDC, LDAP, RADIUS, Unix login, SSH key distribution, passkeys/WebAuthn. It is most often used as the single identity source behind services like Grafana, Nextcloud, Gitea, Vaultwarden, and similar.

**Kanidm Admin UI** is a polished web administration interface for kanidm. It is a single-page application that talks to kanidm's existing REST API at `/v1/`. It replaces the kanidm CLI for routine administration: creating people, managing groups and their members, configuring OAuth2 applications, resetting credentials, and other day-to-day identity-management tasks.

It is **not** the regular self-service portal — end users still use the existing kanidm `/ui/` for their own credential management. This UI is for **administrators**.

---

## 2. Audience

**Primary user:** the self-hoster / homelab operator / small-team sysadmin who runs kanidm to provide identity for their stack. They are technically comfortable — they read JSON, they read documentation, they understand what an OAuth2 scope is — but they value fast, polished tools that get out of the way. They are routinely switching between this UI, a terminal, and an IDE.

**Secondary user:** help-desk or junior IT staff at a small organization performing common user-management tasks: onboarding new employees, resetting credentials, adding people to groups, removing people who left.

The UI must serve both well: efficient for the power user (keyboard shortcuts, dense data views, search-first), legible for the help-desk user (clear labels, descriptive empty states, confirmation flows that prevent mistakes).

---

## 3. Core principles

1. **Search is the default.** Every list view shows a prominent search box. A global command palette (Cmd+K / Ctrl+K) jumps to any person, group, OAuth2 app, or section.
2. **Keyboard-driven.** Common actions have shortcuts. Tabbing through forms is logical. Modals trap focus and close on Esc.
3. **Fast list views.** Tables load quickly, support sorting, support filtering. Pagination is preferred over infinite scroll.
4. **Progressive disclosure for advanced features.** Account policy fields, OAuth2 cryptographic key rotation, claim map join strategy — these are tucked into sections that are obvious once you need them but never noisy.
5. **Copy-paste friendly.** UUIDs, SPNs, tokens, secrets, fingerprints — anything that ends up in another tool's config file — has a one-click copy button next to it.
6. **No accidental destructive actions.** Delete, revoke, regenerate, purge — these always confirm. For irreversible operations (deleting a person, regenerating a secret), use type-to-confirm.
7. **Dark mode is the default.** This UI lives next to a terminal. Light mode is supported but not the focus.
8. **Honest empty and error states.** Never a blank screen — always tell the user what to do next.
9. **Tokens shown once.** Generated secrets, reset URLs, and RADIUS shared secrets follow a one-time-reveal pattern (visible after generation, masked thereafter, regeneration confirmed).

---

## 4. Tech context for the designer

- **Single-page app** in a browser tab, served as static assets from its own container alongside kanidm.
- **Talks to a REST API** at `/v1/` on the same origin (Traefik or similar reverse proxy routes both).
- **OAuth2/OIDC is a first-class feature.** This product's users are knee-deep in OAuth2 terminology daily — the UI can use terms like "scope map", "claim map", "PKCE", "RS256", "redirect URI" without defining them, but should always provide a help tooltip / link for the less-common ones.
- **Security/identity domain** — the visual tone should feel trustworthy, technically credible. Avoid playful illustrations or marketing-y flourishes.
- **Runs adjacent to terminals and IDEs** — match the visual register of tools like Linear, Vercel, GitHub, Cloudflare. Dense data is normal. Monospace for IDs/tokens is normal.

---

## 5. Navigation structure

Top-level navigation (sidebar or topbar, designer's call):

1. **Dashboard** — landing/home with counts and quick actions
2. **People** — person management (CRUD, credentials, SSH, RADIUS, sessions, validity)
3. **Groups** — group management (CRUD, members, account policy)
4. **OAuth2 Apps** — OAuth2/OIDC client management (full feature parity with the CLI)

Top-right user menu:
- Current user identity (SPN + display name + avatar)
- **Self** — view your own profile, your own session info
- **Sessions** — your own active sessions
- **Theme toggle** (dark / light / system)
- **Sign out**

Global elements:
- Breadcrumb on every page below the top of the main content area
- Global search (Cmd+K command palette) accessible from anywhere
- Toast/notification region top-right for action feedback
- Re-authentication modal that appears when a privileged action needs an active privilege session

---

## 6. Sample data conventions

Every concrete example, name, UUID, and value in any screen brief MUST match `_sample-data.md`. Examples:

- The example people are `alice.smith`, `bob.jones`, `eve.taylor`, `carol.nguyen`, `admin`. Domain is `idm.example.com`.
- The example groups are `idm_admins`, `developers`, `devops`, `vpn_users`, `on_call`, `system_admins`.
- The example OAuth2 apps are `grafana`, `nextcloud`, `gitea`, `vaultwarden`, `homelab-spa`, `cli-deploy-tool`.

Use these names consistently across all mockups so reviewers can compare across screens without re-orienting.

---

## 7. Feature inventory

### Dashboard (1 screen)
Counts (people / groups / OAuth2), domain info card, current session card, quick action buttons.

### People (10 screens — `screens/20-29-*.md`)
- List with search, filters (status, recently created, expiring), bulk actions
- Create person (name + display name; per CLI limit, other fields edited after)
- Detail view (tabbed: Overview / Credentials / SSH Keys / RADIUS / Sessions / Validity)
- Edit (display name, legal name, mail list)
- Credentials display + admin-generated **intent-token reset link** (TTL picker → URL + QR code, copy-to-share with user)
- SSH key management (list, add with label, delete by label)
- RADIUS secret (one-time show on generation, regenerate, delete)
- Sessions list with per-session destroy
- Validity window (valid-from, expire-at; supports keyword shortcuts: `now`, `never`, `clear`, RFC3339 datetime)
- Delete person (type-SPN-to-confirm)

### Groups (7 screens — `screens/40-46-*.md`)
- List with search and filters
- Create (name + optional entry-managed-by)
- Detail view (tabbed: Overview / Members / Account Policy)
- Edit (rename, description, mail list, entry-managed-by)
- Member management (add by name/uuid, remove, purge-all, set-all/replace with confirm)
- Account policy configuration: credential-type-minimum (any/mfa/passkey/attested), password-minimum-length, auth-session-expiry, privilege-session-expiry, webauthn-attestation-ca-list (JSON upload), limit-search-max-results, limit-search-max-filter-test, allow-primary-cred-fallback
- Delete (type-name-to-confirm)

### OAuth2 Apps (11 screens — `screens/60-6A-*.md`)
- List (cards or table)
- Create (pick type basic/public → name+displayname+origin → review)
- Detail view (tabbed: General / Scope Maps / Claim Maps / Crypto / Image / Advanced)
- General settings: rename (with warning), display name, landing URL, supplementary redirect URLs, toggles (PKCE, strict redirect URL, localhost redirects, consent prompt, prefer-short-username, legacy crypto)
- Basic secret view / reset (one-time show on reset)
- Scope maps: per-group scopes (standard list + supplementary scopes section). Standard scopes: `openid`, `profile`, `email`, `groups`, `groups_uuid`, `groups_name`, `groups_spn`, `ssh_publickeys`, `read`, `supplement`. Custom scopes allowed in supplementary.
- Claim maps: per-claim-per-group values + join strategy (csv / ssv / array)
- Cryptographic keys: list with status (active / rotated / revoked), schedule rotation (now / future datetime), revoke key (heavy confirm)
- Image: preview, upload (png/jpg/svg/webp/gif), remove
- Advanced: refresh token expiry (seconds), device flow toggle (if applicable)
- Delete (type-name-to-confirm)

### Login flow (8 screens — `screens/01-08-*.md`)
- Username entry (with optional instance/domain field if not pre-configured)
- Mechanism choice (when multiple available: password / password+TOTP / password+backup-code / password+security-key / passkey / anonymous)
- Password entry
- TOTP entry (6 digits)
- Backup code entry
- Passkey/WebAuthn prompt (with browser-native authenticator UI overlay)
- Denial screen (with reason)
- Re-authentication modal for privilege escalation

### Self / sessions (2 screens — `80`, `81`)
- Your own profile (read-only): SPN, UUID, mails, groups, current session info
- Your active sessions list with per-session destroy

### Cross-cutting patterns (6 files — `screens/9*.md`)
- Empty states
- Error states (inline, page-level, 401/403/404/500, toast)
- Loading states (skeleton screens, suspense)
- Destructive confirmation pattern
- Copy-and-tokens pattern (masked, reveal, copy, regenerate)
- Search and filter pattern (with command palette)

---

## 8. Not in scope

These exist in the kanidm CLI but the admin UI **does not** include them. Do not invent screens for them.

- **POSIX attributes** — gidnumber, login shell, home directory, unix passwords. Anywhere they appear in the CLI (person posix, group posix), they are intentionally excluded.
- **Service accounts** — separate entity in kanidm, but not covered here.
- **Recycle bin** — restoring deleted entries.
- **System configuration** — domain settings UI, password badlist, denied names, message queue, signing keys at the domain level (only the per-OAuth2-app keys are in scope).
- **IDM Sync** — external identity provider sync configuration.
- **Raw SCIM operations** — direct JSON entry create/update/search via SCIM filter language.
- **Login UI deep customization** — this UI uses the existing kanidm login endpoints; we just re-theme via the chosen design system. There is no settings UI to customize the login appearance.
- **Audit logs / activity feed** — the kanidm API does not currently expose an audit endpoint, so the dashboard does not show "recent activity". A "current session" card replaces what would otherwise be activity.

---

## 9. Reference URLs

For factual ground truth about kanidm's capabilities:

- Kanidm book / docs: https://kanidm.github.io/kanidm/stable/
- Kanidm GitHub: https://github.com/kanidm/kanidm
- CLI reference (book chapters under "Administrators"): https://kanidm.github.io/kanidm/stable/accounts/intro.html
- OAuth2 setup guide: https://kanidm.github.io/kanidm/stable/integrations/oauth2.html

The admin UI surfaces should match the terminology used in those documents exactly. For example: "person" (not "user"), "spn" (not "principal name"), "scope map" (not "scope assignment"), "intent token" (for credential reset), "valid from" / "expire at" (validity window).

---

## 10. How to use this brief

When generating mockups in claude.ai/design:

1. Set up a **Design System** in claude.ai/design using one of `design-system-linear.md`, `design-system-cloudflare.md`, or `design-system-stripe.md`. Fill the company name (`Kanidm`), blurb, GitHub URL, and the palette / typography / motion notes from that file.
2. Paste this `project-description.md` into the project context.
3. Pick a screen brief from `screens/` and ask the designer to render that screen using the design system + sample data.
4. Iterate. Regenerate any screen that doesn't match the brief.
5. Repeat per design system. Compare mockups side-by-side. Pick a winner.
6. Once the visual direction is locked, implementation begins (framework choice, build setup, REST client, etc. decided then).
