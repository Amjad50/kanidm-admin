# 45 — Groups: Account Policy Tab

The Account Policy tab on the group detail page. Enables a per-group authentication policy and configures its settings. All policy fields from kanidm CLI's `group account-policy` subcommands.

## Purpose

Configure the authentication policy that applies to members of this group: minimum credential type, password length, session timeouts, attestation list, search limits, fallback behavior. Per kanidm: each setting can be enabled per-group, and members of multiple policy-enabled groups get the most restrictive combination.

## Layout

Tab content inside the group detail page:

### State A — Policy disabled

```
┌─────────────────────────────────────────────────────────────────────┐
│ Account policy                                                      │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │  📋  Account policy is not enabled for this group.              │ │
│ │                                                                 │ │
│ │  When enabled, you can configure session expiry, credential     │ │
│ │  requirements, and other authentication rules that apply to     │ │
│ │  members of this group.                                         │ │
│ │                                                                 │ │
│ │  [ Enable account policy ]                                      │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### State B — Policy enabled (configured)

```
┌─────────────────────────────────────────────────────────────────────┐
│ Account policy                                                      │
│                                                                     │
│ ● Enabled                                          [Disable policy] │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Authentication requirements                                     │ │
│ │                                                                 │ │
│ │ Credential type minimum                                         │ │
│ │ ( ) Any (no requirement)                                        │ │
│ │ (•) MFA (password + second factor)                              │ │
│ │ ( ) Passkey                                                     │ │
│ │ ( ) Attested passkey                                            │ │
│ │ Used: All members must satisfy this credential strength.        │ │
│ │ [Reset to default]                                              │ │
│ │                                                                 │ │
│ │ Password minimum length                                         │ │
│ │ ┌────┐                                                          │ │
│ │ │ 12 │ characters                                               │ │
│ │ └────┘                                                          │ │
│ │ Minimum: 8. Default: 10.                                        │ │
│ │ [Reset to default]                                              │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Sessions                                                        │ │
│ │                                                                 │ │
│ │ Session expiry        ┌──────┐ seconds   ≈ 8 hours              │ │
│ │                       │ 28800│                                  │ │
│ │                       └──────┘                                  │ │
│ │ Max duration of a sign-in session.                              │ │
│ │ [Reset to default]                                              │ │
│ │                                                                 │ │
│ │ Privileged session   ┌──────┐ seconds   ≈ 30 minutes           │ │
│ │ expiry               │ 1800 │                                  │ │
│ │                      └──────┘                                  │ │
│ │ Duration of an elevated/privileged session before re-auth.     │ │
│ │ [Reset to default]                                              │ │
│ │                                                                 │ │
│ │ Allow primary cred fallback                                     │ │
│ │ [ ] Allow members to use their primary password as a POSIX     │ │
│ │     login fallback                                              │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ WebAuthn attestation                                            │ │
│ │                                                                 │ │
│ │ Attestation CA list                                             │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Currently configured: fido_metadata.json (uploaded 2025-08) │ │ │
│ │ │ [Upload JSON file] [Download current] [Clear]               │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ Restricts passkey registration to certified authenticators.    │ │
│ │ JSON format follows FIDO metadata service.                     │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Search limits                                                   │ │
│ │                                                                 │ │
│ │ Maximum search results       ┌──────┐                           │ │
│ │                              │ 1000 │                           │ │
│ │                              └──────┘                           │ │
│ │ Max number of results returned for searches by members of this │ │
│ │ group.                                                          │ │
│ │ [Reset to default]                                              │ │
│ │                                                                 │ │
│ │ Maximum filter test         ┌──────┐                           │ │
│ │                             │ 1500 │                           │ │
│ │                             └──────┘                           │ │
│ │ Max number of entries tested against a search filter (partial  │ │
│ │ index path).                                                    │ │
│ │ [Reset to default]                                              │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│                                            [Discard] [Save policy]  │
└─────────────────────────────────────────────────────────────────────┘
```

## Tab content

### Enable / disable

When policy is not enabled, show the disabled state (State A). The "Enable account policy" button activates it (calls `POST /v1/group/{name}/_account_policy` per CLI's `group account-policy enable` command). After enabling, the group's `class` array gains `"account_policy"`.

When enabled, show "● Enabled" indicator at top. **NOTE:** kanidm CLI has NO `disable` command for account policy. Once enabled, the `class: account_policy` is permanent. The only way to "disable" is to reset every individual setting to its default (each field has a `reset-{name}` CLI command). The UI should NOT offer a "Disable policy" button — instead offer "Reset all policy settings to default" which loops through each field's reset endpoint.

### API data shape — critical notes

See `../api-reality.md`. Policy fields are top-level attrs on the group entry, NOT nested. Conventions:

- `class` contains `"account_policy"` if policy is enabled.
- Each policy attr (e.g., `authsession_expiry`) is a **string** (`"28800"`) even for numeric/boolean values.
- A policy attr being ABSENT from the entry means the field is at its kanidm-default — NOT that policy is disabled. The class array is the source of truth for enabled/disabled.
- Booleans: `"true"` / `"false"` strings.
- The `webauthn_attestation_ca_list` attr holds the full JSON blob as a string.

When reading: convert string numbers to int, string booleans to bool, missing fields to `null` (meaning "use kanidm default").

When writing: each field uses a generic attribute mutation against `/v1/group/{name}/_attr/{attr_name}`:

| UI field | Attribute name | Notes |
|---|---|---|
| Credential type minimum | `credential_type_minimum` | values: `any`, `mfa`, `passkey`, `attested_passkey` |
| Password minimum length | `auth_password_minimum_length` | **NOTE: `auth_` prefix** — not `password_minimum_length` |
| Session expiry | `authsession_expiry` | seconds as string |
| Privileged session expiry | `privilege_expiry` | seconds |
| WebAuthn attestation CA list | `webauthn_attestation_ca_list` | JSON blob as single string |
| Max search results | `limit_search_max_results` | integer |
| Max filter test | `limit_search_max_filter_test` | integer |
| Allow primary cred fallback | `allow_primary_cred_fallback` | `"true"` / `"false"` |

- Set: `PUT /v1/group/{name}/_attr/{attr}` body `["<value>"]`
- Reset to default: `DELETE /v1/group/{name}/_attr/{attr}` — removes the attribute entirely, kanidm uses its built-in default.
- Enable policy: `POST /v1/group/{name}/_attr/class` body `["account_policy"]` — adds the class.

### Authentication requirements section

**Credential type minimum** (radio group):
- Any (kanidm default)
- MFA (password + second factor — TOTP, security key, etc.)
- Passkey (unattested webauthn allowed)
- Attested passkey (only certified authenticators)

Each option has a one-line description below. Selected option in accent color.

**Password minimum length** (number input):
- Input accepting integers, with "characters" suffix
- Helper: "Minimum: 8. Default: 10."
- Reset to default link (sets to 10 OR removes the attribute — designer's call; "removing" is semantically a reset)

### Sessions section

**Session expiry** (seconds with human conversion):
- Number input + "seconds" suffix
- Right side: human-readable conversion (`≈ 8 hours`, `≈ 30 minutes`)
- Helper: "Max duration of a sign-in session."

**Privileged session expiry** (seconds):
- Same pattern as session expiry
- Helper: "Duration of an elevated/privileged session before re-auth."

**Allow primary cred fallback** (checkbox):
- "Allow members to use their primary password as a POSIX login fallback"
- (Note: POSIX is out of scope per project description, but this account-policy attribute exists. Render it but the description makes its meaning clear without requiring POSIX UI. Admin can still toggle it for use by external POSIX integrations.)

### WebAuthn attestation section

**Attestation CA list** (JSON file management):
- Current state: shows uploaded file name + upload date, OR "Not configured" subtle text
- Actions:
  - "Upload JSON file" — file picker, accepts `.json` only
  - "Download current" — only shown if configured
  - "Clear" — danger-secondary, removes the attestation list
- Helper: "Restricts passkey registration to certified authenticators. JSON format follows FIDO metadata service."

### Search limits section

**Maximum search results** (number input):
- Helper: "Max number of results returned for searches by members of this group."
- Reset to default

**Maximum filter test** (number input):
- Helper: "Max number of entries tested against a search filter (partial index path)."
- Reset to default

### Footer

- Discard — reverts unsaved changes
- Save policy — primary, disabled until any change
- On save: calls the appropriate `kanidm group account-policy {field}` REST equivalents (each field has its own endpoint per CLI mapping)

## States

- **Loading current policy:** skeleton.
- **Disabled (State A):** as described.
- **Enabled, with modifications unsaved:** Save button enabled, "Unsaved changes" indicator.
- **Saving:** Save spinner.
- **Saved:** toast "Policy updated."
- **Field-level error:** inline below the offending field (e.g., "Password minimum length must be at least 8.")
- **Server error:** toast.

## Sample data

Two scenarios:

**For `developers`:**
- Credential type minimum: MFA
- Password minimum length: 12
- Session expiry: 28800 (= 8 hours)
- Privileged session expiry: 1800 (= 30 minutes)
- Allow primary cred fallback: true (checkbox on)
- Attestation CA list: not configured
- Max search results: default (1000)
- Max filter test: default (1500)

**For `idm_admins`:**
- Credential type minimum: Passkey
- Password minimum length: 16
- Session expiry: 3600 (= 1 hour)
- Privileged session expiry: 900 (= 15 minutes)
- Allow primary cred fallback: false
- Attestation CA list: configured (uploaded `fido_metadata.json` on 2025-08-12)
- Max search results: 1000
- Max filter test: 1500

For State A: `vpn_users` (no policy configured).

## Edge cases

- **Field requires value but is empty:** save disabled with inline error.
- **Reducing an existing limit:** changes apply at next save. Existing sessions are not affected immediately (kanidm enforces at session creation).
- **Changing credential-type-minimum to a stricter level:** members who don't satisfy the new requirement may be blocked from future sign-ins until they update credentials. Show a warning callout when raising the level: "⚠ Members whose credentials don't meet the new minimum may be unable to sign in until they update their credentials."
- **Attestation upload — invalid JSON:** show inline error "Could not parse JSON. Make sure the file is valid FIDO metadata."
- **Disabling policy:** confirm dialog. After disable, member's effective policy falls back to kanidm defaults or another policy-enabled group they're in.

## Mockup elements to render

- Tab content with "Account policy" heading
- For mockup, use `developers`'s values (State B)
- All 5 sections: Authentication requirements, Sessions, WebAuthn attestation, Search limits, footer
- "● Enabled" indicator at top + Disable button
- Discard + Save policy buttons at bottom (Save disabled if no changes)
- Render the State A mockup (disabled) for `vpn_users` as a second variant
- Render the "Disable policy" confirm modal as a third variant
