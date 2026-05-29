# 26 — People: RADIUS Tab

The RADIUS tab on the person detail page. Manages a person's RADIUS shared secret, which is used for WiFi/VPN authentication via kanidm's RADIUS integration.

## Purpose

Generate, view (once), regenerate, or delete a person's RADIUS shared secret. The secret is only shown the first time after generation; subsequently it's masked.

## Layout

Tab content inside the person detail page.

### State A — Not configured

```
┌────────────────────────────────────────────────────────────────┐
│ RADIUS                                                         │
│                                                                │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │  📡  RADIUS not configured                                  │ │
│ │                                                            │ │
│ │  Generate a RADIUS shared secret to allow this person      │ │
│ │  to authenticate via RADIUS (e.g., for WiFi or VPN).       │ │
│ │                                                            │ │
│ │  [ Generate RADIUS secret ]                                │ │
│ └────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

### State B — Configured (post-generation, secret revealed once)

```
┌────────────────────────────────────────────────────────────────┐
│ RADIUS                                                         │
│                                                                │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │  ✓ RADIUS secret generated                                 │ │
│ │                                                            │ │
│ │  ⚠ This is the only time the secret will be shown.         │ │
│ │  Copy it now and store it in your RADIUS server config.    │ │
│ │                                                            │ │
│ │  Shared secret                                             │ │
│ │  ┌───────────────────────────────────────────────────┐     │ │
│ │  │ xK8mP2qF9vN4jH7tR1yC6wA3eL5sB0gD              📋  │     │ │
│ │  └───────────────────────────────────────────────────┘     │ │
│ │                                                            │ │
│ │  [ I've saved the secret ]                                 │ │
│ └────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

### State C — Configured (secret hidden, default state on tab load if already exists)

```
┌────────────────────────────────────────────────────────────────┐
│ RADIUS                                                         │
│                                                                │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │  📡  RADIUS is configured                                   │ │
│ │                                                            │ │
│ │  A RADIUS shared secret is set for this person. The secret │ │
│ │  cannot be retrieved — you can only regenerate or delete   │ │
│ │  it.                                                       │ │
│ │                                                            │ │
│ │  [ Regenerate secret ]    [ Delete secret ]                │ │
│ └────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

## Tab content

### Heading

"RADIUS"

### Card content varies by state

**State A (no RADIUS secret yet):**
- Icon (Lucide `Antenna` or `Wifi`) + heading "RADIUS not configured"
- Body: "Generate a RADIUS shared secret to allow this person to authenticate via RADIUS (e.g., for WiFi or VPN)."
- Primary button: "Generate RADIUS secret"

Clicking the button calls `POST /v1/person/{id}/_radius` (kanidm generates the secret server-side). Requires privilege session.

**State B (just generated, one-time display):**

- Success header with checkmark "RADIUS secret generated"
- Strong warning: "⚠ This is the only time the secret will be shown. Copy it now and store it in your RADIUS server config."
- Secret display block:
  - Label "Shared secret"
  - Read-only input or styled monospace block showing the secret in plain text
  - Copy button (Lucide `Copy`) on the right side — on click, button briefly shows "Copied" with checkmark
- Acknowledgement button: "I've saved the secret" — clicking transitions to State C (the secret is no longer accessible from the UI)
- If the user leaves the tab / page without acknowledging, the next time they return, the secret is gone (State C). This is intentional — the secret is fetched once via `GET /v1/person/{id}/_radius` after generation, but after acknowledgement the UI doesn't request it again.

Wait — actually per the kanidm API, `GET /v1/person/{id}/_radius` returns the secret if it exists. So we could theoretically re-show it. But best practice in admin UIs is to treat secrets as one-time-show even when re-retrievable — it forces the admin to record it externally. Document this design decision: the UI deliberately doesn't re-display the secret after acknowledgement. To get it again, regenerate (which invalidates the old one).

**State C (configured, hidden):**

- Icon + heading "RADIUS is configured"
- Body: "A RADIUS shared secret is set for this person. The secret cannot be retrieved — you can only regenerate or delete it."
- Two action buttons:
  - "Regenerate secret" — secondary. Confirm modal: "Regenerating will create a new secret and invalidate the current one. Any RADIUS clients using the old secret will stop working until updated. Continue?" with Cancel / Regenerate. On confirm: calls `POST /v1/person/{id}/_radius` again, transitions to State B with the new secret.
  - "Delete secret" — danger button. Confirm modal: "Delete the RADIUS secret? RADIUS authentication will be disabled for this person until a new secret is generated." with Cancel / Delete. On confirm: calls `DELETE /v1/person/{id}/_radius`, transitions to State A.

## States

- **Loading:** small skeleton card while fetching status.
- **Idle (any of A/B/C):** as described.
- **Generating/regenerating:** button shows spinner.
- **Generation success:** transitions to State B.
- **Generation error:** toast "Could not generate RADIUS secret. Try again."
- **Deletion success:** toast "RADIUS secret deleted." + transitions to State A.
- **Deletion error:** toast.

## Sample data

For Alice Smith from `_sample-data.md`:
- Alice's RADIUS is configured (State C in normal display)

For Bob Jones:
- Not configured (State A)

The sample RADIUS secret to show in State B (post-generation mockup):
- `xK8mP2qF9vN4jH7tR1yC6wA3eL5sB0gD`

## Edge cases

- **Privilege session required:** all actions (Generate, Regenerate, Delete) require an active privilege session.
- **RADIUS not enabled on the server:** kanidm might require server-side RADIUS configuration. If `POST` returns a specific error indicating RADIUS isn't enabled, show a message: "RADIUS isn't enabled on this kanidm server. Ask the server administrator to configure RADIUS first." with a link to the kanidm docs page.
- **Concurrent regeneration:** if two admins regenerate at the same time, one of them sees the new secret; the other gets a stale view. Refresh on tab focus to mitigate.
- **Person doesn't belong to the RADIUS access group:** kanidm requires the person to be in a specific group (e.g., `idm_radius_servers` or similar) to actually use RADIUS. The UI does not check this — it's a separate config. Consider adding a notice in State A or B: "Make sure this person is a member of your configured RADIUS access group."

## Mockup elements to render

- Tab content with "RADIUS" heading
- Render State A (not configured) for Bob Jones
- Render State B (post-generation) for Alice with the sample secret `xK8mP2qF9vN4jH7tR1yC6wA3eL5sB0gD`, including the warning banner, monospace secret display, copy button, and "I've saved the secret" acknowledgement button
- Render State C (configured, hidden) for Alice (default state when revisiting), with Regenerate + Delete buttons
- Render the Regenerate confirm modal as a separate mockup
