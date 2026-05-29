# 81 — Sessions Page (Self)

The signed-in admin's own list of active sessions. Lets them destroy individual sessions (including their current one, with caution) or destroy all sessions.

## Purpose

Show the admin their own active sessions across devices and API tokens. Highlight which is the current session. Allow ending any session.

This is the self-managed equivalent of the per-person Sessions tab (screen 27). The data and UX are similar; differences are:
- The current session is highlighted with a "This session" badge
- Destroying the current session signs the admin out
- This page is accessed via the user menu → "My sessions" or directly at `/sessions`

## Layout

Inside the app shell:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Your sessions                                                       │
│                                                                     │
│ 3 active sessions                                  [Destroy others] │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Session                Issued       Expires      Purpose  Actions│ │
│ │─────────────────────────────────────────────────────────────────│ │
│ │ a4c2e8f1…  ★ This      09:22       17:22         RW priv  ✕    │ │
│ │ b8d3f9a2…              May 13      May 13 22:08  RW       ✕    │ │
│ │ c1e4a7b8…  📡 API      May 10      Never         RO       ✕    │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ [Sign out everywhere]                                               │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- No breadcrumb or `Self > Sessions`
- Title: "Your sessions"

## Controls row

- Subtitle: "{N} active sessions"
- Right: "Destroy others" — secondary button. Destroys all sessions EXCEPT the current one. Opens confirm: "Destroy {N-1} other sessions? You'll remain signed in on this device."

## Sessions table

Same columns as screen 27 (per-person sessions), with one addition:

1. **Session ID** — monospace + copy. Includes context badge:
   - "★ This" (or "Current") for the current session
   - "📡 API" for API tokens (purpose=read-only AND expires=never AND no human-issued context)
   - Otherwise plain
2. **Issued** — relative / absolute
3. **Expires** — "Never" or relative / absolute
4. **Purpose** — RW (read-write) / RO (read-only) / RW priv (privileged read-write)
5. **Actions** — "✕" destroy button

Destroying a session that is NOT the current one: small confirm "Destroy session a4c2e8f1…?" with Cancel / Destroy.

Destroying the CURRENT session: stronger confirm:

```
   ┌──────────────────────────────────────────────────┐
   │  Sign out this session?                  [×]     │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  This is your current session on this device.    │
   │  Continuing will sign you out immediately.       │
   │                                                  │
   │  Your other sessions on other devices and any    │
   │  API tokens will remain active.                  │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]       [Sign me out]        │
   └──────────────────────────────────────────────────┘
```

After confirm: API call destroys the session, browser is redirected to login screen 01.

## Sign out everywhere

Bottom of page (or in a kebab-style menu): "Sign out everywhere" — destructive primary. Confirm:

```
   ┌──────────────────────────────────────────────────┐
   │  Sign out everywhere?                      [×]   │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  ⚠ This destroys all 3 active sessions including │
   │  this one. You'll be signed out everywhere and   │
   │  any API tokens you have will stop working.      │
   │                                                  │
   │  Use this if you suspect any device is           │
   │  compromised.                                    │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │           [Cancel]    [Sign me out everywhere]   │
   └──────────────────────────────────────────────────┘
```

After confirm: destroy all sessions in a loop. Redirect to login.

## States

- **Loading:** skeleton table.
- **One active session (just the current one):** "Destroy others" is hidden or disabled. Single row in table. "Sign out everywhere" still available.
- **No sessions (impossible while logged in):** N/A.
- **Destroying:** row state, then row removed (or, for current session, full sign-out flow).
- **Error destroying a session:** toast.

## Sample data

For the admin's sessions (use Alice's sample as a stand-in if needed):
- Session `a4c2e8f1-…` — ★ This — issued 09:22 — expires 17:22 — RW privileged
- Session `b8d3f9a2-…` — issued May 13 14:08 — expires May 13 22:08 — RW
- Session `c1e4a7b8-…` — 📡 API token (CI) — issued May 10 — expires Never — RO

## Edge cases

- **Destroying current session via "Destroy others" mistake:** "Destroy others" should be wired carefully — only destroy non-current sessions. UI never accidentally destroys current.
- **Privileged session vs ordinary read-write session:** distinguish in the table with "RW priv" badge.
- **API tokens with revealing labels:** kanidm doesn't currently store custom labels for sessions — context is inferred from the purpose/expiry pattern. Don't fabricate labels.
- **Browser back after sign-out:** UI restored from cache may briefly show stale state; force a refresh on auth-failure response.

## Mockup elements to render

- Page title "Your sessions"
- Subtitle "3 active sessions" + "Destroy others" button
- Sessions table with 3 rows: current (★), regular, API token
- "Sign out everywhere" button at bottom
- Render the "Sign out this session?" confirm modal (when admin attempts to destroy the current session)
- Render the "Sign out everywhere?" confirm modal
