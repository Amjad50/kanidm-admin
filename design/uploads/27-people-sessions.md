# 27 — People: Sessions Tab

The Sessions tab on the person detail page. Lists active session tokens for this person and lets the admin destroy individual sessions.

## Purpose

Show all active sessions for the person (which devices / API clients are currently signed in). Allow the admin to destroy any single session or destroy all sessions at once (e.g., when a device is lost or after an incident).

## Layout

Tab content inside the person detail page:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Sessions                                                            │
│                                                                     │
│ 3 active sessions                                  [Destroy all]    │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Session                Issued       Expires      Purpose Actions│ │
│ │─────────────────────────────────────────────────────────────────│ │
│ │ a4c2e8f1…              09:22       17:22         read-write  ✕ │ │
│ │ This device · Privileged                                        │ │
│ │                                                                 │ │
│ │ b8d3f9a2…              May 13      May 13 22:08  read-write  ✕ │ │
│ │ Another device                                                  │ │
│ │                                                                 │ │
│ │ c1e4a7b8…              May 10      Never         read-only   ✕ │ │
│ │ API token (CI)                                                  │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Tab content

### Heading row

- "Sessions"
- Right side: count subtitle "{N} active sessions" + "Destroy all" danger-ghost button

### Sessions table

Columns:

1. **Session ID** (30% width)
   - First line: monospace truncated UUID (`a4c2e8f1...`) with copy icon (Lucide `Copy`) — full UUID copied on click
   - Second line, subdued: context hints — "This device" if it's the admin's own session being viewed (only relevant for the Self page; for other people, just hide this), or a short purpose label like "API token (CI)" if known. Privilege state: "Privileged" if the session has privilege_capable, otherwise omit.

2. **Issued** (15% width)
   - Date the session began. Show relative ("2 hours ago") for sessions within the last 24h, absolute ("May 13 14:08") for older. On hover, show full ISO timestamp tooltip.

3. **Expires** (15% width)
   - "Never" for sessions with no expiry (API tokens), otherwise relative or absolute date. **API note:** kanidm returns session state as one of:
     - `"revoked"` — already-revoked (hide by default, show via toggle for history)
     - `"expires at <RFC3339>"` — has an expiry timestamp
     - `"never_expires"` (or similar — verify in proto) — for API tokens with no expiry

4. **Purpose** (15% width)
   - Badge showing the auth token purpose. **API note:** the CLI emits these as space-separated lowercase: `read write`, `read only`, `privilege_capable` (per kanidm proto: `ReadWrite`, `ReadOnly`, `PrivilegeCapable`).
     - `read only` — neutral / info
     - `read write` — neutral primary
     - `privilege_capable` — accent (highlight that this session has privilege)

5. **Actions** (10% width)
   - "✕" or trash icon button to destroy this individual session
   - On click: small confirm dialog "Destroy session a4c2e8f1…? The signed-in client will be signed out immediately."

### Destroy all action

Primary destructive action at the top right of the header row. On click: confirm modal:

```
   ┌────────────────────────────────────┐
   │  Destroy all sessions?       [×]   │
   ├────────────────────────────────────┤
   │                                    │
   │  This will sign out Alice from     │
   │  every device and invalidate all   │
   │  API tokens. They will need to     │
   │  sign in again.                    │
   │                                    │
   │  3 sessions will be destroyed.     │
   │                                    │
   │              [Cancel] [Destroy all]│
   └────────────────────────────────────┘
```

On confirm, the UI loops `DELETE /v1/account/{id}/_user_auth_token/{sid}` for each session. Show a progress indicator during. Toast on completion: "All sessions destroyed."

## States

- **Loading:** skeleton table.
- **No active sessions:** empty state in the table area: "No active sessions." with a small note: "{Person} is not currently signed in."
- **Destroying one session:** that row shows a "Destroying…" state with spinner, then disappears.
- **Destroying all:** the action button shows spinner + "Destroying…", table rows fade out as each completes.
- **Error destroying:** toast per failure; the row remains visible.

## Sample data

Use Alice Smith's sessions from `_sample-data.md`:

| Session ID | Issued | Expires | Purpose | Context |
|---|---|---|---|---|
| `a4c2e8f1-...` | 2026-05-14 09:22 | 2026-05-14 17:22 | read-write (privileged) | This device |
| `b8d3f9a2-...` | 2026-05-13 14:08 | 2026-05-13 22:08 | read-write | Another device |
| `c1e4a7b8-...` | 2026-05-10 11:15 | never | read-only | API token (CI) |

## Edge cases

- **Destroying the admin's own session:** if the admin destroys a session that's their own current session, they're immediately signed out — show a confirm dialog with extra emphasis: "This is your current session. Destroying it will sign you out." with explicit "Sign me out" button. (For the Sessions tab on the admin's own profile, accessed via `/self`, this is more common.)
- **Session ID truncation:** show first 8 characters + ellipsis. Tooltip on hover shows full UUID. Copy gets the full UUID.
- **Very many sessions (e.g., API token bot):** if there are 20+ sessions, paginate the table. Keep "Destroy all" available at the top.
- **Session destroyed externally during view:** if the user is on the page and another admin destroys a session, the next refresh shows it gone. No special animation needed.
- **API tokens vs interactive sessions:** kanidm doesn't strictly distinguish in the API, but admin can infer from `purpose: read-only` and `expires: never` patterns. Use the context hint "API token" when these patterns match.

## Tone

Direct. Sessions tab is for security-conscious operations (offboarding, incident response, "I forgot my laptop in a cab"). The destroy actions should feel confident, not whimsical.

## Mockup elements to render

- Tab content with "Sessions" heading
- "3 active sessions" subtitle + "Destroy all" button top-right
- Table with all 3 sample sessions for Alice
- Privileged indicator visible on session 1
- "API token (CI)" context on session 3
- "Never" expiry on session 3, formatted dates on the others
- Render the "Destroy all" confirm modal as a separate mockup
- Render an empty-sessions state for a person who isn't signed in anywhere
