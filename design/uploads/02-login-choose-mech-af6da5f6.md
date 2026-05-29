# 02 — Login: Choose Authentication Mechanism

After the username step, if kanidm returns more than one available authentication mechanism, the user picks how they want to authenticate.

## Purpose

Let the user choose between available auth mechanisms (e.g., they have a passkey registered AND a password+TOTP combo). Skipped automatically when only one mechanism is available.

## Available mechanisms (from kanidm)

Per the kanidm proto definition (`proto/src/v1/auth.rs`), these are the possible mechanisms:

- **`password`** — Password only (no MFA)
- **`passwordmfa`** — Password + TOTP
- **`passwordbackupcode`** — Password + backup code (recovery)
- **`passwordsecuritykey`** — Password + security key (FIDO2 second factor)
- **`passkey`** — Passkey (WebAuthn, single-factor authenticator)
- **`anonymous`** — Anonymous (the admin UI does not surface this option even if returned by the server)

Each user account has only a subset of these enabled. The UI shows only what kanidm returns.

## Layout

Same centered card pattern as `01-login-username.md` (~440px wide).

```
              ┌────────────────────────┐
              │       Kanidm           │
              │   idm.example.com      │
              ├────────────────────────┤
              │  Choose how to sign in │
              │                        │
              │  Signed in as:         │
              │  alice.smith@…         │
              │                        │
              │  ┌──────────────────┐  │
              │  │ 🔑 Passkey       │  │
              │  ├──────────────────┤  │
              │  │ 🔒 Password + TOTP│ │
              │  ├──────────────────┤  │
              │  │ 🔒 Password + ⋯  │  │
              │  ├──────────────────┤  │
              │  │ 🛡️ Backup code   │  │
              │  └──────────────────┘  │
              │                        │
              │  ← Use different user  │
              └────────────────────────┘
```

## Card content

**Header:** same wordmark and domain as login-username.

**Identity reminder:**
A small block at the top of the content area showing who the user just claimed to be:
- Avatar (24px) + display name + SPN (e.g., "Alice Smith · alice.smith@idm.example.com")
- A back-link below: "← Sign in as someone else" (returns to step 01)

**Title:** "Choose how to sign in"

**Mechanism list:**
A vertical stack of mechanism choice buttons. Each is a wide button (full-width within the card), aligned left with icon + label + (optional) description. Buttons are listed in priority order (passkey first because it's the most secure single-factor option, then MFA combos, then password-only last).

For each mechanism:

| Mechanism | Label | Icon (Lucide) | Description |
|---|---|---|---|
| `passkey` | Passkey | `Key` | Use a registered authenticator (Touch ID, YubiKey, etc.) |
| `passwordmfa` | Password and TOTP | `LockKeyhole` | Your password plus a one-time code from your authenticator app |
| `passwordsecuritykey` | Password and security key | `KeySquare` | Your password plus a tap on your security key |
| `passwordbackupcode` | Backup code | `LifeBuoy` | Use a recovery backup code |
| `password` | Password only | `Lock` | Sign in with just your password |

Each mechanism button has:
- 56px height
- Icon (20-24px) on the left in a square / circular container
- Two lines of text: bold label + subdued description
- Right side: ChevronRight icon (16px)
- Hover state: full-row hover background
- Click: submits the chosen mechanism to `POST /v1/auth` with `step: { Begin: <mech> }`, then navigates to the appropriate cred-entry screen (03, 04, 05, or 06)

## States

- **Idle:** as described.
- **Loading (after choice):** chosen row shows a spinner replacing the chevron, other rows go disabled.
- **Error:** toast notification "Could not start authentication" with a "Retry" action.

## Sample data

Identity reminder block at top:
- Avatar with "AS" initials
- Display name "Alice Smith"
- SPN "alice.smith@idm.example.com"

Mechanism list (Alice's example): `passkey`, `passwordmfa`, `passwordbackupcode`. Three rows shown.

## Edge cases

- **Single mechanism:** UI auto-navigates and skips this screen entirely.
- **No mechanisms:** very rare (account has no credentials). Shows a denied screen (`07`) with reason "No authentication mechanisms available for this account".
- **Anonymous filtered out:** if kanidm returns `anonymous` alongside other mechanisms, we hide it. If `anonymous` is the only mechanism, show denied screen (admin UI doesn't permit anonymous).
- **Mechanism unavailable mid-flow:** if the user picks a mechanism and the server then says "actually that's not allowed right now" (rare), show a toast and refresh the list.

## Tone

Calm, helpful. Descriptions should make the user feel confident they're picking the right thing. Avoid jargon like "WebAuthn" or "FIDO2" in the user-facing labels — the descriptions use natural language ("a registered authenticator", "a one-time code").

## Mockup elements to render

- Centered card, same shell as screen 01
- Header: "Kanidm" wordmark + "idm.example.com" subtitle
- Identity reminder: avatar "AS" + "Alice Smith" + "alice.smith@idm.example.com" + "← Sign in as someone else"
- Title "Choose how to sign in"
- Three mechanism rows: Passkey, Password and TOTP, Backup code
- Each row with icon + bold label + description + chevron
- Bottom: same footer as login-username
