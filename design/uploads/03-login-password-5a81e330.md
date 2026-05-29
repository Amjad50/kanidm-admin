# 03 — Login: Password Entry

The user enters their password as part of a password-based authentication mechanism.

## Purpose

Collect the user's password securely. Submit to kanidm. On success, either complete authentication (if mechanism is `password`) or proceed to a second factor (if mechanism is `passwordmfa`, `passwordbackupcode`, or `passwordsecuritykey`).

## Layout

Same centered card as previous login screens (~440px wide).

```
              ┌────────────────────────┐
              │       Kanidm           │
              │   idm.example.com      │
              ├────────────────────────┤
              │  Enter your password   │
              │                        │
              │  ┌──────┐ Alice Smith  │
              │  │ AS   │ alice.smith@…│
              │  └──────┘              │
              │                        │
              │  Password              │
              │  ┌──────────────────┐  │
              │  │ ••••••••      👁 │  │
              │  └──────────────────┘  │
              │                        │
              │  [   Continue   ]      │
              │                        │
              │  ← Choose another      │
              │     method             │
              └────────────────────────┘
```

## Card content

**Header:** wordmark + domain "idm.example.com".

**Title:** "Enter your password"

**Identity row:** small avatar (32px) + display name on top, SPN below (e.g., "Alice Smith" / "alice.smith@idm.example.com"). Helps reassure the user they're signing in as the right person, especially after the username and mechanism-choice steps.

**Password input:**
- Label "Password"
- Type `password` (masked)
- Autocomplete: `current-password`
- Autofocus on page load
- Right-side reveal toggle: eye icon (Lucide `Eye` / `EyeOff`) — click to toggle masked/unmasked. Tooltip "Show password" / "Hide password".
- Caps Lock indicator: if Caps Lock is detected while the input is focused, show a small warning icon + tooltip on the right side of the input: "Caps Lock is on".
- Submit on Enter

**Primary action:** full-width button "Continue".

**Secondary link:** "← Choose another method" — text link below the button, returns to mechanism-choice screen (02).

## States

- **Idle:** as described.
- **Loading:** button shows spinner + label "Verifying…". Input is read-only.
- **Error — wrong password:** input border in `--danger`, inline error below: "Incorrect password. Try again." Input is cleared and refocused. Button returns to idle.
- **Error — account locked:** navigate to denied screen (07) with reason "Account is temporarily locked. Try again in 5 minutes." (kanidm returns specific lockout duration where possible.)
- **Error — server error:** toast "Could not verify password. Try again." Button returns to idle.
- **Success — auth complete (mechanism was `password`):** brief success indicator (e.g., checkmark animation), then redirect to dashboard.
- **Success — proceed to MFA:** redirect to TOTP (04), backup code (05), or security key entry screen depending on mechanism.

## Sample data

- Avatar + name + SPN: `alice.smith@idm.example.com`, display name "Alice Smith"
- Password placeholder: shown as `••••••••` masked dots
- After wrong attempt, inline error in `--danger` color

## Edge cases

- **Browser password manager:** the input accepts autofill. After autofill, "Continue" still requires a click (or Enter).
- **Empty submit:** button is disabled until at least 1 character is entered.
- **Long passwords:** input does not truncate visually; allow it to scroll if needed.
- **Account requires password change:** kanidm returns a specific response code; UI shows a notice "Your password must be changed before continuing" and redirects to the reset flow (out of admin UI scope — kanidm's existing `/ui/reset` route handles this).
- **Failed attempts approach softlock:** after several failures, show inline warning below input: "Multiple failed attempts. Your account will be locked after 2 more tries."

## Tone

Direct. No reassurances about security ("we use bcrypt!") — those don't belong here. Don't apologize for asking for a password.

## Mockup elements to render

- Centered card with wordmark + domain
- "Enter your password" title
- Identity row (avatar "AS" + "Alice Smith" + SPN)
- Password input with masked dots + eye-toggle button inside
- Full-width "Continue" primary button
- "← Choose another method" text link
- Show an example error state in a separate render: input in danger color, "Incorrect password. Try again." below
