# 01 — Login: Username Entry

The first step of the kanidm authentication flow. The user enters their username; kanidm responds with available authentication mechanisms.

## Purpose

Identify the user so kanidm can return the appropriate mechanism list. Provide a clean, focused entry point with no distractions. Optionally allow specifying a different kanidm instance URL if the UI is configured to support multiple instances.

## Layout

Centered card, ~440px wide, on a full-viewport background. Background is the canvas color from the design system. Optional subtle visual interest (a soft gradient blob, a faint dot pattern, a very subdued geometric mark) — the design system dictates whether to include it.

```
                                   
              ┌────────────────┐
              │     Kanidm     │
              ├────────────────┤
              │                │
              │  Sign in       │
              │                │
              │  ┌──────────┐  │
              │  │ Username │  │
              │  └──────────┘  │
              │                │
              │  [ Continue ]  │
              │                │
              │  ── or ──      │
              │                │
              │  [ Passkey ]   │
              │                │
              └────────────────┘
```

## Card content

**Header:**
- "Kanidm" wordmark at top (or stylized "K" + wordmark)
- 4px small status text: "idm.example.com" (the domain being signed into) in a subdued color

**Title:**
- "Sign in" in the design system's page-title size

**Form:**
- Single text input, label "Username"
- Placeholder: `you@idm.example.com` (illustrating SPN format) OR just `username` (illustrating short name) — designer's call
- Helper text below input: "Enter your username or SPN (e.g., alice.smith or alice.smith@idm.example.com)"
- Autocomplete: `username`
- Autofocus on page load
- The input border state follows the design system

**Primary action:**
- Full-width button "Continue", primary color
- Submits the form. On submit, kanidm returns the list of available auth mechanisms; the UI navigates to `screens/02-login-choose-mech.md` (or skips to the appropriate input page if only one mechanism is offered).

**Divider:**
- Horizontal line with the word "or" centered on it

**Secondary action:**
- Full-width button "Sign in with passkey", secondary style
- Icon: Lucide `Key` 16px on left
- Initiates the unauthenticated passkey flow (relevant when the user has a passkey and doesn't want to type their name — passkey credential reveals which user is signing in)

## States

- **Idle:** as described above. Username field focused.
- **Loading (after Continue):** the button shows a spinner inside, button label changes to "Signing in…", input is read-only, passkey button is disabled.
- **Error — user not found:** inline error below the input: "Account not found" in `--danger` text. Input border in `--danger`. The error clears as soon as the user types.
- **Error — server unreachable:** a toast appears top-right: "Could not reach kanidm server. Check your connection and try again." Button returns to idle state.
- **Error — denied (e.g., account disabled):** input field shakes briefly (shake-animation 200ms, two oscillations), then error message below: "This account cannot sign in. Contact an administrator." Form remains in idle for retry.

## Footer (below the card)

- "Forgot your password?" text link. Clicking opens kanidm's existing self-service `/ui/reset` flow (the user would need a separately-shared reset URL — admin generates these). The link copy should be honest: clicking shows a small popover: "Ask an administrator to generate a reset link for you."
- A small "About Kanidm" link to https://kanidm.github.io/kanidm/stable/ in subdued text.

## Sample data

- The domain shown in the header: `idm.example.com`
- The username placeholder example: `alice.smith` or `alice.smith@idm.example.com`
- The "currently signing in to" status: `idm.example.com`

## Edge cases

- **Instance URL configuration:** if the SPA was deployed with a single configured kanidm URL, no instance picker is shown — the URL is fixed. (Multi-instance support is out of scope.)
- **Anonymous authentication enabled:** kanidm exposes an `anonymous` mechanism. The UI does NOT surface this on the username screen (the admin UI is not for anonymous use). If the user manages to authenticate anonymously via API, they're shown a "Anonymous accounts cannot administer kanidm" denied screen.
- **Browser autofill:** the username input should accept browser-saved usernames. If browser autofills a username + password, the password is ignored on this screen (password is on the next screen).
- **WebAuthn-only account:** kanidm returns only `passkey` as the available mechanism. UI navigates directly to the passkey screen, skipping mechanism choice.

## Tone

Neutral, calm, professional. Don't be playful — this is the security gate of an identity system. Avoid copy like "Welcome back, friend!" Avoid mascot illustrations.

## Mockup elements to render

For a generated mockup, render:
- The centered card on the canvas background
- The "Kanidm" wordmark + domain "idm.example.com"
- "Sign in" title
- Username input with placeholder "alice.smith"
- "Continue" primary button (full-width)
- "or" divider
- "Sign in with passkey" secondary button (full-width, with key icon)
- Bottom: "Ask an administrator for a reset link" text link in subdued tone
- Document the focus state visually (input has accent ring)
