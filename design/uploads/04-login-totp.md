# 04 — Login: TOTP Entry

The user enters a 6-digit time-based one-time password from their authenticator app, as the second factor for the `passwordmfa` mechanism.

## Purpose

Collect a 6-digit TOTP. Verify it with kanidm. Complete authentication on success.

## Layout

Same centered card pattern (~440px wide).

```
              ┌────────────────────────────┐
              │       Kanidm               │
              │   idm.example.com          │
              ├────────────────────────────┤
              │  Enter the 6-digit code    │
              │                            │
              │  ┌──┐                      │
              │  │AS│ Alice Smith          │
              │  └──┘ alice.smith@…        │
              │                            │
              │  ┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐        │
              │  │7││3││9││2││4││1│        │
              │  └─┘└─┘└─┘└─┘└─┘└─┘        │
              │                            │
              │  Code refreshes every 30s  │
              │                            │
              │  [    Verify    ]          │
              │                            │
              │  Use backup code instead   │
              └────────────────────────────┘
```

## Card content

**Header:** wordmark + domain.

**Title:** "Enter the 6-digit code" — direct, no jargon.

**Identity row:** same as screen 03 (avatar + display name + SPN).

**TOTP input:**
A 6-digit input. Two visual styles are acceptable:

- **(Preferred) Six separate digit boxes:** 6 boxes side-by-side, each ~48px wide, ~56px tall, monospace font, large character size (~28px). Auto-advance on entry: typing a digit moves focus to the next box. Backspace moves focus back. Paste of a 6-digit code distributes across boxes. Visually clear and tactile.
- **(Alternative) Single input field:** one wider input, `inputmode="numeric"`, `pattern="[0-9]{6}"`, `maxlength="6"`, large monospace font, letter-spacing wide for readability. Used if the design system favors simplicity over visual emphasis.

Designer picks based on the design system's density. Linear-variant likely uses single input; Cloudflare/Stripe likely use six boxes.

**Helper text:**
"Code refreshes every 30 seconds" — informational, in subdued tone.

A small progress indicator showing the current TOTP window's remaining time can be added (a 16px circular progress ring next to the helper text, counting down 30 → 0 seconds). This is a nice-to-have, not required.

**Primary action:** "Verify" button. Disabled until 6 digits are entered. Auto-submits when 6 digits are entered (designer's call — typical pattern is auto-submit, with the button as fallback).

**Secondary link:**
- "Use backup code instead" — navigates to backup code screen (05) if `passwordbackupcode` is also available for the account
- "← Choose another method" — same as previous screens

## States

- **Idle:** as described. First digit box focused.
- **Entering:** as user types, digits appear in boxes. After 6 digits, button enables (and may auto-submit).
- **Verifying:** boxes go read-only, button shows spinner + "Verifying…" label.
- **Error — wrong code:** boxes shake briefly (200ms, two oscillations), boxes clear and refocus to first. Inline error: "Incorrect code. Try again." Show this immediately after the failed verification.
- **Error — code expired:** "The code has expired. Enter the next code from your authenticator." (rare — happens when user typed slowly past the 30s window)
- **Success:** brief checkmark, redirect to dashboard.

## Sample data

- Identity: Alice Smith / `alice.smith@idm.example.com`
- Sample 6-digit code shown in a mockup (idle state): empty boxes
- Sample 6-digit code shown in a mockup (filled state): `739241`

## Edge cases

- **Clock drift:** kanidm allows a small time window. If the code is rejected due to drift, the error is the same as wrong-code — no special handling needed at this layer.
- **Paste support:** the input accepts a pasted 6-digit code and distributes it correctly across boxes.
- **Mobile autofill:** modern browsers can autofill TOTP from SMS or password managers. The 6-digit input pattern supports this.
- **No backup code configured:** the "Use backup code instead" link is hidden.

## Tone

Calm. Matter-of-fact. The user is in the middle of signing in — don't add friction with extra copy.

## Mockup elements to render

- Centered card with wordmark + domain
- "Enter the 6-digit code" title
- Identity row (Alice Smith)
- Six digit boxes (mockup can show either empty state or "739241" filled)
- Helper text "Code refreshes every 30 seconds"
- "Verify" primary button (full-width)
- "Use backup code instead" text link
- Optional: small 16px countdown ring next to helper text
- Show an error state mockup separately: boxes empty, inline error "Incorrect code. Try again."
