# 6A — OAuth2 Apps: Delete Confirm Modal

Confirmation modal for deleting an OAuth2 application. Type-to-confirm pattern.

## Purpose

Make OAuth2 client deletion intentional. Show consequences (existing tokens invalidated, scope maps cleaned up, signed-in users affected). Require typed confirmation.

## Layout

Modal overlay (~520-560px wide).

```
   ┌──────────────────────────────────────────────────┐
   │  Delete OAuth2 application                [×]    │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  ⚠ You're about to delete:                       │
   │                                                  │
   │  [Grafana icon]   Grafana                        │
   │                   grafana                        │
   │                   Confidential                   │
   │                                                  │
   │  What happens:                                   │
   │   ▸ All issued tokens are immediately invalid.   │
   │     Users currently signed in to Grafana via     │
   │     kanidm will be signed out.                   │
   │   ▸ The 3 scope maps and signing keys for this   │
   │     application are removed.                     │
   │   ▸ The application moves to the recycle bin     │
   │     and is recoverable for 7 days.               │
   │   ▸ Any external system using the client_id      │
   │     "grafana" will stop authenticating until     │
   │     reconfigured.                                │
   │                                                  │
   │  Type the application name to confirm:           │
   │  grafana                                         │
   │  ┌────────────────────────────────────────────┐  │
   │  │                                            │  │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]    [Delete application]    │
   └──────────────────────────────────────────────────┘
```

## Modal content

**Header:** "Delete OAuth2 application" + close.

**Body:**

*Lead-in:* "⚠ You're about to delete:" with danger icon.

*Identity card:*
- App image (48px) — uploaded image or placeholder
- Display name (primary): "Grafana"
- System name (monospace, subdued): "grafana"
- Type subtitle: "Confidential" or "Public"

*"What happens" section:*
- ▸ "All issued tokens are immediately invalid. Users currently signed in to {DisplayName} via kanidm will be signed out."
- ▸ "The {N} scope maps and signing keys for this application are removed." (count adapts; omit if 0)
- ▸ "The application moves to the recycle bin and is recoverable for 7 days."
- ▸ "Any external system using the client_id `{system_name}` will stop authenticating until reconfigured."

*Type-to-confirm:*
- "Type the application name to confirm:"
- System name in monospace + copy button
- Input field; matches enable Delete button

**Footer:** Cancel + Delete application (danger primary, disabled until match).

## States

- **Idle:** Delete disabled.
- **Match:** Delete enabled.
- **Deleting:** spinner.
- **Success:** modal closes, toast "Application deleted: Grafana — recoverable from the recycle bin for 7 days." Navigate to `/oauth2` or remove row.
- **Error:** inline at top.

## Sample data

For Grafana:
- Display name: Grafana
- System name: grafana
- Type: Confidential
- 3 scope maps (standard + 1 supplementary count adaptable: e.g., "4 scope maps")
- Required confirm text: `grafana`

## Edge cases

- **Privilege required:** triggers reauth modal on Delete click.
- **Bulk delete (from list):** list of system names + type `DELETE` to confirm.
- **App with active tokens:** kanidm handles invalidation; UI doesn't need to do anything special. The consequence line covers it.

## Mockup elements to render

- Modal with backdrop
- Header "Delete OAuth2 application" + close
- Body with warning icon + lead-in
- Identity card for Grafana
- "What happens" bulleted list
- Type-to-confirm with `grafana` in monospace + copy
- Empty input + Delete button (disabled)
- Render a second variant: input filled `grafana` and Delete enabled
- Render bulk variant: list of 3 OAuth2 apps + type DELETE pattern
