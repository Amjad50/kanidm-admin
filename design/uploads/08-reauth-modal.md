# 08 — Re-authentication Modal (Privilege Escalation)

A modal that appears when an admin attempts a privileged action (mutating data, viewing a secret, etc.) but their privilege session has expired or never existed. The user re-authenticates without losing their current page context.

## Purpose

Confirm the admin is still who they claim to be, before allowing a sensitive operation. Avoid forcing a full sign-out / sign-in round-trip. Preserve context (the page they were on, the action they were about to take) so they can resume after re-authentication.

## When this appears

- User clicks a destructive action (Delete person, Revoke key, etc.) and privilege session is expired
- User clicks "Reveal secret" on a one-time-show widget
- User attempts to generate a reset link / regenerate RADIUS / regenerate OAuth2 secret
- User clicks "Re-authenticate" in the user menu (proactive privilege grant)

The UI checks privilege session state from the auth token's claims (`purpose: 'privilege_capable'` vs. `'readwrite'`). When `privilege_capable` is needed but not active, this modal opens.

## Layout

Standard modal (per the design system's modal spec).

- Width: ~480px (small modal)
- Backdrop: per design system
- Modal contents:

```
   ┌────────────────────────────────────┐
   │  Re-authenticate required     [×]  │
   ├────────────────────────────────────┤
   │                                    │
   │  This action requires elevated     │
   │  privileges. Please re-confirm     │
   │  your identity to continue.        │
   │                                    │
   │  ┌──┐                              │
   │  │AD│ System Administrator         │
   │  └──┘ admin@idm.example.com        │
   │                                    │
   │  Password                          │
   │  ┌──────────────────────────────┐  │
   │  │ ••••••••                  👁 │  │
   │  └──────────────────────────────┘  │
   │                                    │
   │  ──── or ────                      │
   │                                    │
   │  [ 🔑 Use passkey ]                │
   │                                    │
   ├────────────────────────────────────┤
   │              [Cancel] [Authenticate]│
   └────────────────────────────────────┘
```

## Modal content

**Header:**
- Title: "Re-authenticate required"
- Close button (×) top-right — equivalent to Cancel

**Body:**

*Context line:* "This action requires elevated privileges. Please re-confirm your identity to continue."

*Identity reminder:*
- 32px avatar, current user's display name, SPN
- Reassures the user they're confirming the right account

*Authentication options:*
The modal supports multiple mechanisms. Show the user's primary mechanism by default; offer alternatives below an "or" divider.

For a user with password+TOTP:
- Password input first
- After password verifies, modal updates in-place to show TOTP input

For a user with a passkey:
- A primary action "Use passkey" button that invokes `navigator.credentials.get()`
- A divider "or"
- A password fallback (if account has both registered)

The modal does NOT re-prompt for username — the current session identifies the user.

**Footer:**
- Right-aligned: secondary "Cancel" button + primary "Authenticate" button
- Authenticate button is disabled until input is valid (password is non-empty, etc.)

## States

- **Idle:** as described.
- **Authenticating:** Authenticate button shows spinner + "Verifying…" label. Inputs go read-only.
- **Error — wrong password / TOTP:** inline error below input. User retries within the modal.
- **Success:** modal closes; the original action that triggered the modal is resumed automatically. A subtle toast confirms: "Privileged session active for the next 15 minutes." (Duration is per the account policy.)
- **Cancelled:** modal closes; the original action is aborted. The user remains on the page, no destructive action happens.

## Sample data

- Current user (admin doing the action): display name "System Administrator", SPN `admin@idm.example.com`, avatar initials "SA"
- Password input pre-focused
- Original action: e.g., "Generate reset link for alice.smith"

## Edge cases

- **Token expired entirely:** the modal can't refresh privilege if the session itself is dead. Detect this case and redirect to login screen 01, preserving an intent (e.g., return URL) so the user lands back on the same page after signing in.
- **Multiple mechanisms configured:** show the most-used (per local storage hint, or fall back to passkey > MFA > password order).
- **Anonymous user (impossible here, since admin UI denies anonymous):** not applicable.
- **User has only TOTP mechanism (no password):** show TOTP input directly.
- **Privilege escalation declined repeatedly:** after 3 failed attempts within the modal, close modal, show toast: "Could not verify identity. Try again." and require a fresh open.

## Tone

Direct, brief. The user is in the middle of something — don't lecture about security. Just confirm and move on.

## Mockup elements to render

- Modal with backdrop overlay
- Modal header: "Re-authenticate required" + close button
- Body context line
- Identity reminder (avatar + name + SPN for the `admin` account)
- Password input with reveal toggle
- "or" divider
- "Use passkey" secondary button with key icon
- Footer: "Cancel" + "Authenticate" buttons
- Show a variant: TOTP step (after password verified, the modal transitions to showing 6-digit input)
