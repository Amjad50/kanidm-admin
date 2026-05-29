# 07 — Login: Denied / Failure Screen

A terminal screen shown when kanidm definitively denies the authentication attempt. The user cannot proceed without help.

## Purpose

Tell the user clearly what went wrong, give them a path forward (contact admin, retry from scratch), without revealing information that helps attackers (e.g., don't say "account doesn't exist" — say "could not sign in").

## Layout

Same centered card pattern as other login screens (~440px wide).

```
              ┌────────────────────────┐
              │       Kanidm           │
              │   idm.example.com      │
              ├────────────────────────┤
              │       ⚠️               │
              │   (warning icon)       │
              │                        │
              │  Sign-in denied        │
              │                        │
              │  Your account is       │
              │  temporarily locked    │
              │  after multiple failed │
              │  sign-in attempts.     │
              │                        │
              │  Try again in:         │
              │  4 min 32 sec          │
              │                        │
              │  ──────────            │
              │                        │
              │  [ Try a different     │
              │    account ]           │
              │                        │
              │  Contact an admin if   │
              │  you can't sign in.    │
              └────────────────────────┘
```

## Card content

**Header:** wordmark + domain.

**Visual indicator:**
Center-aligned `--warning` or `--danger` icon (Lucide `AlertTriangle` or `ShieldX`), ~48px, in the appropriate semantic color.

**Title:** "Sign-in denied"

**Reason text:**
A clear, plain-language description of why the attempt was denied. The kanidm server returns reasons; the UI maps them to user-facing messages. Common cases:

| Server reason | User-facing message |
|---|---|
| Account locked (brute force) | "Your account is temporarily locked after multiple failed sign-in attempts." |
| Account disabled / expired | "This account is not active. Contact an administrator." |
| Account valid-from in the future | "This account is not yet active. It becomes available on {date}." |
| No authentication mechanisms available | "No sign-in method is configured for this account. Contact an administrator." |
| Anonymous attempted | "Anonymous sign-in is not permitted here." |
| Too many in-flight auth sessions | "Too many sign-in attempts. Wait a moment and try again." |
| Generic denied (fallback) | "Could not sign in. Contact an administrator if this continues." |

**Countdown (only if applicable — softlock):**
If the denial is a temporary lockout with a known expiry, show a countdown timer below the reason:
- Label: "Try again in:"
- Countdown: "4 min 32 sec" (updates every second)
- When the countdown reaches zero, the page auto-redirects to screen 01 (username entry)

**Primary action:** "Try a different account" button — returns to screen 01.

**Helper text below the button:** "Contact an admin if you can't sign in." with the configured admin email or contact info if available (kanidm doesn't currently surface a contact endpoint; this can be a static value configured at deployment time, or omitted).

## States

- **Denied — temporary (with countdown):** as above with timer.
- **Denied — permanent (no countdown):** same layout, no timer, primary button is "Sign in as a different user".
- **Denied — future-dated account:** message includes the activation date (e.g., "available on 2026-06-01"). No retry button — the date is in the future. Button label: "Back to sign-in".

## Sample data

For the countdown variant:
- Reason: "Your account is temporarily locked after multiple failed sign-in attempts."
- Countdown: "4 min 32 sec"

For a permanent denial:
- Reason: "This account is not active. Contact an administrator."

For a future-dated account:
- Reason: "This account is not yet active. It becomes available on 2026-06-01."

## Edge cases

- **Reason text length:** if the reason is unusually long, the card grows vertically. No truncation.
- **Server returns no reason:** use the generic fallback "Could not sign in. Contact an administrator if this continues."
- **No admin contact configured:** omit the helper line; just show the button.
- **JavaScript disabled (countdown won't tick):** show the static text "Try again in approximately 5 minutes" without a live timer.

## Tone

Neutral, not blaming, not apologetic. Don't say "we" or "sorry" — be matter-of-fact. The denial is a fact; the user needs a path forward.

Do NOT reveal:
- Whether the account exists or not
- How many failed attempts there have been
- Whether the failure was on password vs. TOTP vs. something else (kanidm itself is intentionally vague)

## Mockup elements to render

- Centered card pattern
- Warning/danger icon (large, centered)
- "Sign-in denied" title
- Reason text (use the locked-out example: "Your account is temporarily locked after multiple failed sign-in attempts.")
- Countdown: "Try again in: 4 min 32 sec"
- "Try a different account" primary button
- "Contact an admin if you can't sign in." helper text
- Render a second variant: permanent denial, no countdown, message "This account is not active. Contact an administrator."
