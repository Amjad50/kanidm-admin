# 64 — OAuth2 Apps: Basic Secret (View / Reset)

The basic secret management view for confidential OAuth2 clients. One-time-show pattern on reset, regenerate confirm.

## Purpose

Let the admin retrieve the current basic auth secret (kanidm's API allows this — `GET /v1/oauth2/{name}/_basic_secret` returns `{"secret": "..."}`), and reset (regenerate) the secret with a confirm. After reset, the new secret is fetched via the same GET endpoint.

For **public clients**, this screen shows an explainer instead: public clients don't have a basic secret.

## API endpoints

- **Read:** `GET /v1/oauth2/{name}/_basic_secret` returns `{"secret": "<value>"}` (or null for public clients).
- **Regenerate:** `PATCH /v1/oauth2/{name}` with body `{"attrs": {"oauth2_rs_basic_secret": []}}` (clearing the attr triggers regeneration server-side). Then GET above to fetch the new value.

There is NOT a dedicated "rotate-secret" endpoint that returns the new secret in the response — the rotation is two-step: clear-then-fetch.

## Layout

This is accessed via the "View secret" button on the detail identity card (screen 62), not as a tab. The UI can render it as a modal or as an inline reveal — designer's call. Modal is recommended for the security framing.

### Modal variant (recommended)

```
   ┌──────────────────────────────────────────────────┐
   │  Grafana — Basic secret                  [×]     │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  Current secret                                  │
   │  ┌──────────────────────────────────────────┐    │
   │  │ ••••••••••••••••••••••••••••  [👁] [📋]  │    │
   │  └──────────────────────────────────────────┘    │
   │                                                  │
   │  Reveal to copy. Treat this as sensitive — it    │
   │  authorizes any client using your client_id to   │
   │  request tokens.                                 │
   │                                                  │
   │  ──────────                                      │
   │                                                  │
   │  Regenerate secret                               │
   │  Creates a new secret and invalidates the        │
   │  current one. Any client using the old secret    │
   │  will stop working until updated.                │
   │                                                  │
   │  [ Regenerate ]                                  │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Close]                             │
   └──────────────────────────────────────────────────┘
```

### Inline variant (alternative)

A card on the OAuth2 detail page, shown only when "View secret" is clicked. Same content, in-page instead of modal.

## Content

**Header:** "{App display name} — Basic secret" + close (×).

### Current secret block

- Label "Current secret"
- Reveal-toggle masked input/display
  - Masked default: `••••••••••••••••••••••••••••`
  - Eye icon: click reveals the secret in monospace
  - Copy icon: click copies the full secret. On click: "Copied" feedback for 1.5s.
- Below the block: "Reveal to copy. Treat this as sensitive — it authorizes any client using your client_id to request tokens."

### Regenerate section

- Heading "Regenerate secret"
- Body: "Creates a new secret and invalidates the current one. Any client using the old secret will stop working until updated."
- Primary button: "Regenerate"
- Clicking opens a confirm sub-modal:

```
   ┌──────────────────────────────────────────────────┐
   │  Regenerate basic secret?                  [×]   │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  ⚠ Confirm regeneration                          │
   │                                                  │
   │  The current secret will be invalidated          │
   │  immediately. Any application using the old      │
   │  secret will stop working until updated.         │
   │                                                  │
   │  Affected: any deployed Grafana instance using   │
   │  the current client_id and secret.               │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]   [Regenerate secret]      │
   └──────────────────────────────────────────────────┘
```

After confirming, the API call is made. The modal returns to a "post-regeneration" state showing the new secret:

```
   ┌──────────────────────────────────────────────────┐
   │  Grafana — Basic secret                  [×]     │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  ✓ Secret regenerated                            │
   │                                                  │
   │  ⚠ This is the only time the new secret is shown │
   │  in full. Copy it now and update your Grafana    │
   │  configuration.                                  │
   │                                                  │
   │  New secret                                      │
   │  ┌──────────────────────────────────────────┐    │
   │  │ kFn3pV9wQ2bT8nX1aL5cR4jH7sM6dY...  [📋]   │    │
   │  └──────────────────────────────────────────┘    │
   │                                                  │
   │  [ I've saved the new secret ]                   │
   │                                                  │
   └──────────────────────────────────────────────────┘
```

After "I've saved" is clicked, return to the regular view (masked, reveal button still works to get the secret again since kanidm's API allows retrieval — but the explicit acknowledgement step reinforces security culture).

## Public client variant

For a public client, the modal shows an explainer:

```
   ┌──────────────────────────────────────────────────┐
   │  Homelab SPA — Basic secret              [×]     │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  Public clients don't have a basic secret.       │
   │                                                  │
   │  This client uses PKCE for the authorization     │
   │  code flow. Public clients can't safely store    │
   │  secrets, so authentication relies on PKCE       │
   │  proof challenges instead.                       │
   │                                                  │
   │  Learn more: https://kanidm.github.io/kanidm/    │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Close]                             │
   └──────────────────────────────────────────────────┘
```

## States

- **Loading current secret:** the reveal toggle is disabled until the secret is fetched.
- **Idle (masked):** as described.
- **Revealed:** secret visible.
- **Regenerating:** Regenerate button spinner + "Regenerating…".
- **Post-regeneration:** success state with new secret.
- **Error:** toast.

## Sample data

For Grafana:
- Current secret (revealed for mockup): `kFn3pV9wQ2bT8nX1aL5cR4jH7sM6dY8fZ2gH4tQ5xA9eC1nB3vK7p` (50-character random-ish string)
- For masked state: 50 bullet characters

For the public-client variant: Homelab SPA.

## Edge cases

- **Privilege session required:** opening this modal triggers reauth modal if privilege session expired.
- **Concurrent regeneration:** if two admins regenerate at the same time, the second one wins. Show toast on the loser's side: "The secret was changed by another admin. Reload to see the current secret."
- **Secret with special characters:** render in monospace; copy preserves the exact value.
- **Server returns no secret (corrupt state):** show "Could not retrieve the current secret. Try regenerating." with a Regenerate button.

## Mockup elements to render

- Modal with header "Grafana — Basic secret"
- Current secret block with mask + eye toggle + copy
- "Reveal to copy" helper text
- Regenerate section with heading + body + Regenerate button
- Footer with Close
- Render a second variant: revealed state showing the sample secret
- Render the regenerate confirm sub-modal
- Render the post-regeneration variant: success header, warning callout, new secret visible, "I've saved the new secret" acknowledgement button
- Render the public-client variant for Homelab SPA
