# 69 — OAuth2 Apps: Advanced Tab

The Advanced tab on the OAuth2 detail page. Configures refresh token expiry and the device flow toggle.

## Purpose

Configure less-frequently-used OAuth2 settings. Per kanidm CLI: `set-refresh-token-expiry` and `device-flow-enable` / `device-flow-disable` (feature-gated `dev-oauth2-device-flow`).

## Layout

Tab content inside the OAuth2 detail page:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Advanced                                                            │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Refresh token expiry                                            │ │
│ │                                                                 │ │
│ │ ┌──────────────┐ seconds   ≈ 30 days                            │ │
│ │ │ 2592000      │                                                 │ │
│ │ └──────────────┘                                                 │ │
│ │ How long a refresh token remains valid. Leave blank to use the  │ │
│ │ server default. Typical: 7-30 days.                             │ │
│ │ [Reset to default]                                              │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Device authorization flow                                       │ │
│ │                                                                 │ │
│ │ Device flow                                       [○ Disabled]  │ │
│ │ Enables the OAuth2 device authorization grant for this client.  │ │
│ │ Used by TVs, IoT devices, and other input-constrained devices.  │ │
│ │                                                                 │ │
│ │ ⚠ Device flow is feature-gated on the server. Enabling here     │ │
│ │   requires the kanidm server to be built with the               │ │
│ │   dev-oauth2-device-flow feature.                               │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│                                            [Discard] [Save changes] │
└─────────────────────────────────────────────────────────────────────┘
```

## Tab content

### Refresh token expiry card

**Field:**
- Number input (integer, seconds)
- Suffix: "seconds"
- Right-side: human-readable conversion ("≈ 30 days") next to the field
- Helper: "How long a refresh token remains valid. Leave blank to use the server default. Typical: 7-30 days."
- Reset to default link — sets to empty (kanidm uses its default)

If the field is empty: human-readable shows "Using server default".

### Device flow card

**Toggle row:**
- Label "Device flow"
- Description: "Enables the OAuth2 device authorization grant for this client. Used by TVs, IoT devices, and other input-constrained devices."
- Toggle switch right-aligned

**Warning callout** below the toggle (always shown):
"⚠ Device flow is feature-gated on the server. Enabling here requires the kanidm server to be built with the dev-oauth2-device-flow feature."

If the server reports the feature is not available (e.g., kanidm returns an error on the flag set), the toggle is disabled with tooltip "Device flow is not enabled on this kanidm server."

### Footer

- Discard — revert
- Save changes — primary, disabled until any modification

## States

- **Loading:** skeleton.
- **Idle:** current values shown.
- **Modified:** Save enabled.
- **Saving:** spinner.
- **Error:** inline or toast.
- **Feature not supported:** device flow toggle disabled.

## Sample data

For `grafana`:
- Refresh token expiry: `2592000` (30 days)
- Device flow: Disabled

For `vaultwarden`:
- Refresh token expiry: empty (server default)
- Device flow: Disabled

## Edge cases

- **Refresh token expiry of 0:** kanidm may reject as invalid. Show inline error: "Expiry must be a positive number."
- **Very large value (e.g., billions of seconds):** allowed up to kanidm's max. Show human-readable conversion regardless.
- **Server doesn't support device flow:** toggle disabled with tooltip.

## Mockup elements to render

- Tab content with "Advanced" heading
- Refresh token expiry card with `2592000` filled and "≈ 30 days" conversion
- Device flow card with toggle off (○) + warning callout
- Footer with Discard + Save changes
- Render a second variant: refresh token field empty showing "Using server default" + Reset to default link visible
- Render a variant where device flow is not supported on the server: toggle greyed out, tooltip visible
