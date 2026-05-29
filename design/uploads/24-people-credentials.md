# 24 — People: Credentials Tab (Status + Reset Link)

The Credentials tab on the person detail page. Shows the read-only credential status from kanidm AND lets the admin generate an intent-token reset link to share with the person.

## Purpose

Display what credentials the person has (without revealing secret material), and provide the admin with a copy-paste reset URL the user can open to set/update their own credentials via kanidm's existing `/ui/reset` self-service flow.

This screen does NOT let the admin directly change another person's password / TOTP / passkey. Per kanidm's design, all credential changes go through the user's self-service flow, kicked off by an admin-generated intent token.

## Layout

Tab content inside the person detail page (screen 22's shell).

```
┌────────────────────────────────────────────────────────────────────┐
│ Credentials                                                        │
│                                                                    │
│ Current credentials                                                │
│ ┌────────────────────────────────────────────────────────────────┐ │
│ │ Primary       Password + TOTP                                  │ │
│ │ Passkeys      2 registered                                     │ │
│ │   ‣ MacBook Pro Touch ID                                       │ │
│ │   ‣ YubiKey 5C NFC                                             │ │
│ │ Attested      None                                             │ │
│ │ Backup codes  6 remaining (of 8 generated)                     │ │
│ │ SSH keys      3 (manage on SSH Keys tab)                       │ │
│ │ RADIUS        Configured (manage on RADIUS tab)                │ │
│ └────────────────────────────────────────────────────────────────┘ │
│                                                                    │
│ Reset credentials                                                  │
│ ┌────────────────────────────────────────────────────────────────┐ │
│ │ Generate a one-time reset link to share with Alice. They can   │ │
│ │ use it to change their password, add a passkey, or rotate      │ │
│ │ their TOTP without your further involvement.                   │ │
│ │                                                                │ │
│ │ Link expires in   ┌──────────────┐                             │ │
│ │                   │ 1 hour    ▾  │                             │ │
│ │                   └──────────────┘                             │ │
│ │                                                                │ │
│ │ [ Generate reset link ]                                        │ │
│ └────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

After generating, the bottom card transforms to show the link:

```
│ Reset credentials                                                  │
│ ┌────────────────────────────────────────────────────────────────┐ │
│ │ ✓ Reset link generated. Share it with Alice.                   │ │
│ │                                                                │ │
│ │ This link will expire on 2026-05-14 17:22 UTC (in 1 hour).     │ │
│ │                                                                │ │
│ │  ┌──────────────────────────────────────────────────────────┐  │ │
│ │  │ https://idm.example.com/ui/reset?token=••••••••••••••… │ │
│ │  │                                              [👁] [📋]   │  │ │
│ │  └──────────────────────────────────────────────────────────┘  │ │
│ │                                                                │ │
│ │  ┌─────┐                                                       │ │
│ │  │ QR  │   Scan to open on a phone                             │ │
│ │  └─────┘                                                       │ │
│ │                                                                │ │
│ │  [Generate new link]                                           │ │
│ └────────────────────────────────────────────────────────────────┘ │
```

## Tab content sections

### Current credentials card

A key-value table showing the credential status. **Data sources are split:** detailed credential data comes from `GET /v1/person/{id}/_credential/_status` (REST), but the CLI's `person credential status` command returns only `password: set/not set`, a `totp` label list, and `backup_code: disabled/...` in TEXT format (no JSON output). The UI should call the REST endpoint directly. For passkey/attested-passkey labels, the UI reads from the person entry's `passkeys` attr (a string array of labels — kanidm doesn't expose registration date or detail).

All fields are read-only.

Fields:
- **Primary** — describes the primary credential: "None set" / "Password only" / "Password + TOTP" / "Password + security key" / "Passkey only". Inferred from the `_credential/_status` payload's `primary` field (or the person entry's `primary_credential: ["primary"]` sentinel — set vs absent).
- **Passkeys** — count + list of labels (from the person entry's `passkeys` attr array). Empty: "None registered."
- **Attested passkeys** — same pattern. Empty: "None."
- **Backup codes** — kanidm exposes `disabled` or a remaining count via the credential status endpoint. Empty: "None generated."
- **SSH keys** — count from `_ssh_pubkeys` endpoint + link to SSH Keys tab
- **RADIUS** — "Configured" / "Not configured" based on whether the person entry's `radius_secret: ["hidden"]` sentinel is present + link to RADIUS tab

No "Show" or "Reveal" buttons — secret material is never displayed. Only metadata.

### Reset credentials card

Two states: pre-generation and post-generation.

**Pre-generation state:**

- Heading "Reset credentials"
- Description: "Generate a one-time reset link to share with {DisplayName}. They can use it to change their password, add a passkey, or rotate their TOTP without your further involvement."
- TTL picker:
  - Label "Link expires in"
  - Dropdown with options: 15 minutes, 1 hour (default), 4 hours, 24 hours, 7 days. Custom option opens an inline number+unit picker.
  - Helper text below picker: "Shorter is more secure. Longer is more forgiving if the person can't get to it immediately."
- Primary button: "Generate reset link"
- Clicking calls `GET /v1/person/{id}/_credential/_update_intent/{ttl_seconds}` (privilege session required — opens reauth modal if needed).

**Post-generation state:**

- Success header: "✓ Reset link generated. Share it with {DisplayName}."
- Expiry info: "This link will expire on {absolute datetime} (in {relative time})."
- URL display block:
  - Reveal-toggle masked URL: `https://idm.example.com/ui/reset?token=••••••••••…` initially masked
  - Eye icon (Lucide `Eye` / `EyeOff`) toggles visibility (revealing the token portion only — `https://idm.example.com/ui/reset?token=` portion is always visible)
  - Copy icon (Lucide `Copy`) copies the full URL to clipboard. On click: button briefly changes to "Copied" with a checkmark.
- QR code: a small QR (~120px) encoding the full URL. Caption "Scan to open on a phone."
- Secondary button: "Generate new link" — re-runs the flow (invalidates the previous token by generating a new one with a fresh ttl).

Once generated, the URL is **shown once**. If the admin reloads the page or navigates away, the URL is gone (kanidm doesn't store it for later retrieval — the token itself isn't stored, only the intent record). Pre-generation state returns. Document this clearly.

## States

- **Loading credential status:** skeleton lines in the current credentials card.
- **Pre-generation:** as described.
- **Generating:** Generate button → spinner + "Generating…", TTL picker read-only.
- **Post-generation:** as described.
- **Token expired (post-generation):** if the admin sits on this page past the expiry, the expiry text turns red: "This link expired on …". The reveal/copy/QR can still be used (the admin might want to debug) but a warning appears: "⚠ This link has expired. Generate a new one." with a "Generate new link" button.
- **Error generating:** toast "Could not generate reset link. Try again."

## Sample data

For Alice Smith from `_sample-data.md`:
- Credential summary:
  - Primary: Password + TOTP
  - Passkeys: 2 registered — MacBook Pro Touch ID, YubiKey 5C NFC
  - Attested: None
  - Backup codes: 6 remaining (of 8 generated)
  - SSH keys: 3
  - RADIUS: Configured
- Post-generation:
  - URL: `https://idm.example.com/ui/reset?token=xkdzk-fr2p7-fb5wd-5e2hf`
  - Expiry: 2026-05-14 17:22 UTC (in 1 hour)
  - QR code rendering of the full URL

**Token format note:** kanidm intent tokens are short, hyphen-separated codes (e.g., `xkdzk-fr2p7-fb5wd-5e2hf` — 5×5 chars). NOT JWT-style. Designed to be readable / typeable by humans. The mask-and-reveal pattern is overkill for such short tokens — consider just showing them in monospace with a copy button, no mask toggle.

## Out of scope (clarification)

The kanidm API exposes a full **interactive credential-update flow** (state machine across 20+ endpoints) that lets an authorized session edit the primary password, TOTP, passkeys, etc. directly. This admin UI does NOT use that flow — it only generates intent links that the end user opens in the existing kanidm `/ui/reset` view.

Endpoints related to the interactive flow (for reference, not used here):
- `GET /v1/person/{id}/_credential/_update` — begin admin-initiated session
- `POST /v1/credential/_exchange_intent` — user redeems an intent token
- `POST /v1/credential/_update` — state machine step (Password, TotpGenerate, PasskeyInit, etc.)
- `POST /v1/credential/_status` — get current state
- `POST /v1/credential/_commit` — commit staged changes

For Jane Doe (new, no credentials):
- Primary: None set
- Passkeys: None registered
- All other fields: empty or "None" / "Not configured"
- A status banner at top of the tab: "Jane Doe has no credentials yet. Generate a reset link to let them set one up."

## Edge cases

- **No credentials at all:** banner at top + Reset card is the primary action.
- **Privilege session expired:** clicking Generate opens reauth modal (screen 08) before proceeding.
- **Re-generating an active link:** the new generation creates a new intent token; the old one becomes invalid. Show a small note when re-generating: "Generating a new link will invalidate the previous one."
- **Person is the current admin (self):** the Credentials tab still works for their own account, but suggest using the regular self-service flow at `/ui/` instead. Show a small notice: "→ You can also reset your own credentials directly from your self page."
- **Token send by email:** kanidm has a `POST /v1/person/{id}/_credential/_update_intent_send` endpoint that emails the link. This brief includes only the manual copy/QR flow. Adding the email-send option can be a small secondary button "Send by email" below the Generate button — opens a tiny inline form (email picker if multiple emails configured, TTL picker, Send button). This is optional for the v1 mockup. If included, note that it requires kanidm SMTP setup (configurable on the server) and surface a clear error if SMTP isn't configured.

## Mockup elements to render

- Tab content inside person detail shell (Alice Smith identity card visible at top, Credentials tab active)
- "Current credentials" card with all fields populated using Alice's sample data
- "Reset credentials" card in pre-generation state: description text, TTL dropdown (1 hour selected), Generate button
- Render a second variant in post-generation state: success header, expiry info, URL with reveal toggle and copy button, QR code, "Generate new link" button
- Render a third variant for Jane Doe (no credentials): top banner + pre-generation card emphasized
