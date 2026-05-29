# 06 — Login: Passkey / WebAuthn Prompt

The user authenticates with a passkey (WebAuthn credential). The browser shows its native authenticator UI on top; the kanidm page shows a waiting state with a cancel option.

## Purpose

Trigger the browser's WebAuthn `navigator.credentials.get()` call, show a clear "waiting for authenticator" state, and handle success / failure / cancellation.

This screen is used by two flows:
1. As the second factor for `passwordsecuritykey` (after password entry)
2. As the single factor for `passkey` (no password needed)

## Layout

Same centered card pattern (~440px wide).

```
              ┌────────────────────────┐
              │       Kanidm           │
              │   idm.example.com      │
              ├────────────────────────┤
              │  Use your passkey      │
              │                        │
              │  ┌──┐                  │
              │  │AS│ Alice Smith      │
              │  └──┘ alice.smith@…    │
              │                        │
              │                        │
              │       🔑               │
              │   (animated icon)      │
              │                        │
              │   Waiting for your     │
              │   authenticator…       │
              │                        │
              │  Tap or click your     │
              │  security key, or use  │
              │  Touch ID / Windows    │
              │  Hello.                │
              │                        │
              │  [    Cancel    ]      │
              │                        │
              │  ← Choose another      │
              │     method             │
              └────────────────────────┘
```

## Card content

**Header:** wordmark + domain.

**Title:** "Use your passkey"

**Identity row:** avatar + name + SPN.

**Visual indicator (large):**
Center-aligned, ~80px tall area. An animated icon representing "waiting for authenticator":
- A key icon (Lucide `Key`) with a subtle pulse animation (scale 0.95 → 1.05 over 1.5s ease-in-out, infinite), OR
- A device icon (Lucide `Smartphone` or `Fingerprint`) with a similar pulse, OR
- A custom rendering — designer's call based on the design system

The Stripe variant might use a gradient pulse and feel more dramatic. Linear should be minimal. Cloudflare in-between.

**Status text:** "Waiting for your authenticator…"

**Help copy:** "Tap or click your security key, or use Touch ID / Windows Hello." Helpful context for novice users; advanced users skip past it.

**Primary action:** "Cancel" — secondary-style button (this is the cancel path; there's no "Continue" because the browser's native prompt is the primary action). Cancel returns to the mechanism-choice screen (02).

**Secondary link:** "← Choose another method".

## States

- **Waiting (default):** as described above. The browser's native WebAuthn prompt is overlaid by the browser itself.
- **Verifying (after user completes auth on their device):** status text changes to "Verifying…", spinner replaces the animated icon briefly.
- **Success:** brief checkmark, redirect to dashboard.
- **Error — user cancelled in browser prompt:** status text changes to "Authentication cancelled. Try again or choose another method." A retry button appears, replacing the cancel button: "Try again". Clicking it re-invokes `navigator.credentials.get()`.
- **Error — no matching credential:** "Your authenticator doesn't have a credential for this account. Try a different authenticator or choose another method."
- **Error — browser doesn't support WebAuthn:** rare on modern browsers. Show denied screen (07) with reason "Your browser doesn't support passkeys."
- **Error — origin/RP mismatch:** developer error; shows a generic "Could not authenticate. Try again." toast.

## Sample data

- Identity: Alice Smith / `alice.smith@idm.example.com`
- Status text variants:
  - "Waiting for your authenticator…"
  - "Verifying…"
  - "Authentication cancelled. Try again or choose another method."

## Edge cases

- **Unauthenticated passkey flow (from screen 01):** When user clicks "Sign in with passkey" on the username screen without entering a username, this screen still appears but without the identity row at top (since the user hasn't been identified yet). The status text becomes: "Select your account on your authenticator." The browser native prompt will let the user pick from their resident credentials.
- **Conditional UI (passkey autofill):** if supported, the username field on screen 01 can show passkey suggestions inline. That's a separate enhancement — this screen handles only the explicit "click button to use passkey" path.
- **Multiple registered passkeys:** the browser's native UI handles the choice between them.
- **Long wait:** if the user takes >2 minutes, kanidm's session may time out. Show error "Your sign-in attempt has timed out. Start again." with a "Restart" button that returns to screen 01.

## Tone

Patient, clear, reassuring. WebAuthn is unfamiliar to many users; the help copy gives them confidence they're doing the right thing.

## Mockup elements to render

- Centered card pattern
- "Use your passkey" title
- Identity row (Alice Smith)
- Large animated icon area (a key/fingerprint icon, can be static in the mockup)
- "Waiting for your authenticator…" status text
- Help copy "Tap or click your security key, or use Touch ID / Windows Hello."
- "Cancel" secondary button
- "← Choose another method" text link
- Also render a success state (checkmark + "Signed in") and an error state (retry button + cancellation message)
