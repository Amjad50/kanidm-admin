# 67 — OAuth2 Apps: Crypto Tab (Signing Keys)

The Crypto tab on the OAuth2 detail page. Manages cryptographic signing keys for the OAuth2 client's tokens.

## Purpose

View existing signing keys, schedule key rotation (now or future), revoke a specific key. Maps to kanidm CLI's `rotate-cryptographic-keys` and `revoke-cryptographic-key`.

## Layout

Tab content inside the OAuth2 detail page:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Cryptographic keys                                                  │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │  Signing keys are used to sign tokens issued for this app.      │ │
│ │  Older keys remain valid for existing tokens until they expire  │ │
│ │  or are revoked. Rotation creates a new key; new tokens use the │ │
│ │  newest key.                                                    │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ Keys                                                                │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Key ID         │ Created     │ Status     │ Actions             │ │
│ │────────────────┼─────────────┼────────────┼─────────────────────│ │
│ │ key-7f3a2c1d   │ 2026-01-12  │ ● Active   │ Revoke              │ │
│ │ key-2b8e5d4a   │ 2025-08-04  │ ◐ Rotated  │ Revoke              │ │
│ │ key-9c1f3e7b   │ 2025-02-19  │ ⊘ Revoked  │ —                   │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ Rotate keys                                                         │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Schedule a key rotation. The new key becomes active at the      │ │
│ │ specified time; the current key is marked rotated but remains   │ │
│ │ valid for existing tokens.                                      │ │
│ │                                                                 │ │
│ │ Rotate at:                                                      │ │
│ │ (•) Now                                                         │ │
│ │ ( ) Specific date/time                                          │ │
│ │     ┌───────────────────┐ ┌─────────────┐                       │ │
│ │     │ 2026-06-01        │ │ 00:00 UTC   │                       │ │
│ │     └───────────────────┘ └─────────────┘                       │ │
│ │                                                                 │ │
│ │ [ Schedule rotation ]                                           │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## API data shape — critical notes

See `../api-reality.md`. The entry's `key_internal_data` is an array of **pre-formatted strings**, one per key:

```
"57850d1d41fd: valid jws_es256 0"
```

Format: `{key_id}: {status} {algorithm} {rotation_counter}`.

- `status` observed: `valid`. Other values likely exist (`revoked`, `expired`, `retired`) but were NOT observed in the surveyed instance. The brief's "Active / Rotated / Revoked" naming was speculative — verify against actual kanidm source before finalizing display strings.
- `algorithm` observed: `jws_es256` (ECDSA signing) and `jwe_a128gcm` (AES-GCM encryption). A typical OAuth2 client has TWO keys: one for signing tokens, one for encrypting nested JWEs.
- The numeric counter at the end is internal (probably rotation generation); display it as a small "(rev N)" subscript or omit.

There is no separate "created date" — kanidm doesn't expose key creation timestamps. The brief's "Created" column should be removed or replaced with "Algorithm / Status" columns.

The CLI commands `rotate-cryptographic-keys` and `revoke-cryptographic-key` make changes via PATCH to the entry. Refetch after mutation.

## Tab content

### Description

A short paragraph at the top:
"Signing keys are used to sign tokens issued for this app. Older keys remain valid for existing tokens until they expire or are revoked. Rotation creates a new key; new tokens use the newest key."

### Keys table

Columns:
- **Key ID** — monospace, e.g., `key-7f3a2c1d`. Copyable (small copy button next to ID).
- **Created** — date/time, relative or absolute. Hover for full timestamp.
- **Status** — semantic indicator:
  - ● Active (success color) — currently the signing key
  - ◐ Rotated (warning color or neutral) — previous active; still validates old tokens
  - ⊘ Revoked (danger color) — manually invalidated
- **Actions** — "Revoke" button (only for non-revoked keys); revoked rows show "—"

### Revoke action

Clicking Revoke opens a confirm modal:

```
   ┌──────────────────────────────────────────────────┐
   │  Revoke signing key                       [×]    │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  ⚠ Revoke key-2b8e5d4a?                          │
   │                                                  │
   │  All tokens signed by this key will be invalid   │
   │  immediately. Users with active sessions using   │
   │  this key will need to sign in again.            │
   │                                                  │
   │  Only revoke a key if you suspect compromise. If │
   │  you just want to retire an old key, let it      │
   │  expire naturally via rotation.                  │
   │                                                  │
   │  Type the key ID to confirm:                     │
   │  key-2b8e5d4a                                    │
   │  ┌────────────────────────────────────────────┐  │
   │  │                                            │  │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]       [Revoke key]         │
   └──────────────────────────────────────────────────┘
```

Type-to-confirm pattern (key ID, copy-and-paste friendly). On confirm: calls revoke endpoint. Toast on success.

### Rotate keys section

Below the keys table:

**Header:** "Rotate keys"

**Description:** "Schedule a key rotation. The new key becomes active at the specified time; the current key is marked rotated but remains valid for existing tokens."

**Form:**
- Radio: Now (default) / Specific date/time
- If "Specific" selected: date + time pickers (same pattern as validity tab)
- Primary button: "Schedule rotation"

On submit: calls `kanidm system oauth2 rotate-cryptographic-keys NAME ROTATE-AT` REST equivalent (likely a PATCH).

Success: toast "Key rotation scheduled. New key activates at {time}." Table refreshes.

## States

- **Loading keys:** skeleton table.
- **No keys (impossible — kanidm always has at least the active key):** show only the active key. If somehow empty, show a recovery banner: "No signing keys configured. → Schedule a rotation now."
- **Rotating:** Schedule button spinner.
- **Revoking:** confirm modal, then row shows "Revoking…" briefly.

## Sample data

For `grafana` from `_sample-data.md`:
- `key-7f3a2c1d` — created 2026-01-12 — ● Active
- `key-2b8e5d4a` — created 2025-08-04 — ◐ Rotated
- `key-9c1f3e7b` — created 2025-02-19 — ⊘ Revoked

For the rotate form:
- Default state: "Now" radio selected
- Variant: "Specific date/time" with `2026-06-01 00:00 UTC`

## Edge cases

- **Revoking the active key:** server may reject or auto-rotate. If allowed, show prominent warning: "⚠ Revoking the active key will immediately invalidate all tokens for this application. A new active key will be generated. Users must sign in again."
- **Scheduling rotation in the past:** validate client-side; inline error.
- **Multiple rotated keys:** allowed. They all remain valid for existing tokens. Old enough rotated keys may be auto-pruned by kanidm; UI just shows what the API returns.
- **Privilege required:** revoke / rotate require privilege session.

## Mockup elements to render

- Tab content with "Cryptographic keys" heading
- Description paragraph
- Keys table with all 3 sample keys (Active / Rotated / Revoked statuses)
- Rotate keys section with "Now" radio selected, Schedule button
- Render a second variant: rotate form with Specific date selected and date/time populated
- Render the Revoke confirm modal with type-to-confirm
- Render an Active-key revoke variant with the extra-warning callout
