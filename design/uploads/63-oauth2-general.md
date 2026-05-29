# 63 — OAuth2 Apps: General Tab

The General tab on the OAuth2 detail page. Edit basic settings: system name (with rename warning), display name, landing URL, supplementary redirect URLs, and all security toggles.

## Purpose

Single place to edit non-secret, non-relationship-based OAuth2 client configuration. Maps to most of kanidm's `system oauth2 set-*` and `enable-*` / `disable-*` CLI commands.

## Layout

Tab content inside OAuth2 detail page:

```
┌─────────────────────────────────────────────────────────────────────┐
│ General                                                             │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Basic settings                                                  │ │
│ │                                                                 │ │
│ │ Display name                                                    │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Grafana                                                     │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ Shown on consent screens and the apps listing.                  │ │
│ │                                                                 │ │
│ │ System name                                                     │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ grafana                                                     │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ ⚠ Renaming changes the client_id. Any application using the    │ │
│ │   current name must be reconfigured.                            │ │
│ │                                                                 │ │
│ │ Landing URL                                                     │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ https://grafana.example.com                                 │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ Primary URL of the application; default redirect from the apps │ │
│ │ portal.                                                         │ │
│ │                                                                 │ │
│ │ Supplementary redirect URLs                                     │ │
│ │ ┌────────────────────────────────────────────────────────────┐  │ │
│ │ │ https://grafana.example.com/login/generic_oauth   ✕        │  │ │
│ │ └────────────────────────────────────────────────────────────┘  │ │
│ │ [+ Add URL]                                                     │ │
│ │ Additional redirect URIs (e.g., mobile app deep links, OAuth   │ │
│ │ callback paths beyond the landing URL).                        │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Security toggles                                                │ │
│ │                                                                 │ │
│ │ PKCE                                              [● Enabled]   │ │
│ │ Proof Key for Code Exchange. Required for public clients;       │ │
│ │ recommended for all. Disabling reduces security.                │ │
│ │                                                                 │ │
│ │ Strict redirect URL                                [● Enabled]   │ │
│ │ Requires exact URI match. When disabled, only origin is matched │ │
│ │ (path is ignored).                                              │ │
│ │                                                                 │ │
│ │ Localhost redirects (public only)              [○ Disabled]     │ │
│ │ Allow public clients to redirect to http://localhost:*. Use     │ │
│ │ only for development.                                           │ │
│ │                                                                 │ │
│ │ Consent prompt                                    [● Enabled]   │ │
│ │ Show the user a consent screen on first authorization. Disable │ │
│ │ to auto-approve (use carefully).                                │ │
│ │                                                                 │ │
│ │ Prefer short username                            [○ Disabled]   │ │
│ │ Use `name` instead of `spn` for the preferred_username claim.   │ │
│ │                                                                 │ │
│ │ Legacy crypto (HS256)                          [○ Disabled]     │ │
│ │ Enables HS256 token signing for old clients that don't support │ │
│ │ RS256. Reduces security. Don't enable unless required.          │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│                                            [Discard] [Save changes] │
└─────────────────────────────────────────────────────────────────────┘
```

## Tab content sections

### Basic settings card

**Display name** (text input)
- Helper: "Shown on consent screens and the apps listing."

**System name** (text input, monospace)
- Lowercase, dash, underscore allowed (kanidm constraints)
- Rename warning callout appears when the value differs from the saved original: "⚠ Renaming changes the client_id. Any application using the current name must be reconfigured."

**Landing URL** (URL input)
- Helper: "Primary URL of the application; default redirect from the apps portal."
- Warning if non-HTTPS in production-like origin: "⚠ Non-HTTPS URLs are not recommended for production."

**Supplementary redirect URLs** (list editor)
- Each row: URL display + "×" remove button
- "+ Add URL" button at bottom — opens an inline input to type a new URL
- Helper: "Additional redirect URIs (e.g., mobile app deep links, OAuth callback paths beyond the landing URL)."
- Each URL validated as URL on add. Allows `app://`, `https://`, `http://localhost:*` (if localhost toggle is on)

### Security toggles card

Six toggle rows. Each row:
- Toggle label (left)
- One-line description (subdued, below label)
- Toggle switch (right) — filled = on, outline = off
- Toggle states map to kanidm flags:

| UI Toggle | Backing attr | UI ON → attr value | UI OFF → attr value | Default |
|---|---|---|---|---|
| **PKCE** | `oauth2_allow_insecure_client_disable_pkce` | `[]` (cleared) | `["true"]` | PKCE ON (attr absent) |
| **Strict redirect URL** | `oauth2_strict_redirect_uri` | `["true"]` | `["false"]` | ON (secure default) |
| **Localhost redirects** | `oauth2_allow_localhost_redirect` | `["true"]` | `["false"]` | OFF |
| **Consent prompt** | `oauth2_consent_prompt_enable` | `["true"]` | `["false"]` | ON |
| **Prefer short username** | `oauth2_prefer_short_username` | `["true"]` | `["false"]` | OFF (use SPN) |
| **Legacy crypto (HS256)** | `oauth2_jwt_legacy_crypto_enable` | `["true"]` | `["false"]` | OFF |

**⚠ PKCE is INVERTED in the attribute name:** `oauth2_allow_insecure_client_disable_pkce` — `true` means "allow the client to disable PKCE", i.e., PKCE is OFF. The UI normalizer/write helper must flip this. The user-facing label remains "PKCE" (toggle ON = secure).

Some toggles warrant a confirm modal when changing to the riskier state:
- Disabling PKCE: confirm "Disabling PKCE reduces security. Continue?"
- Disabling strict redirect URL: confirm
- Disabling consent prompt: confirm
- Enabling legacy crypto: confirm "Enabling HS256 reduces security. Only enable if your client doesn't support RS256."

### Footer

- Discard — revert unsaved changes
- Save changes — primary, disabled until any field modified. On click:
  - Calls the appropriate kanidm endpoints. Since these map to different attributes and endpoints, the UI can batch them into a single `PATCH /v1/oauth2/{name}` call where possible, or sequentially call each toggle endpoint. Implementation detail.
  - On success: toast "Settings saved." Stay on tab.
  - On error: toast or inline.

## States

- **Idle:** as described with current values.
- **Modified:** Save button enabled, "Unsaved changes" indicator (a small dot or text near footer).
- **Saving:** Save button spinner.
- **Confirmation modal in progress for risky toggle:** that toggle stays in its current state until confirmation; reverts on Cancel.
- **Error:** per-field or toast.

## Sample data

For `grafana` from `_sample-data.md`:
- Display name: Grafana
- System name: grafana
- Landing URL: `https://grafana.example.com`
- Supplementary redirect URLs: `https://grafana.example.com/login/generic_oauth`
- Toggles:
  - PKCE: Enabled
  - Strict redirect URL: Enabled
  - Localhost redirects: Disabled
  - Consent prompt: Enabled
  - Prefer short username: Disabled
  - Legacy crypto (HS256): Disabled

## Edge cases

- **Public client editing localhost-redirects:** the toggle is meaningful. For confidential client, the toggle row is greyed out with tooltip "Localhost redirects apply only to public clients."
- **Public client editing PKCE:** PKCE is enforced for public clients regardless of toggle. The toggle row can be shown but disabled with tooltip "PKCE is always required for public clients."
- **Empty landing URL:** required; cannot save without it.
- **Adding a URL that's already in the list:** show inline error "This URL is already added."
- **System name rename collision:** server 409, inline error.

## Mockup elements to render

- Tab content with "General" heading
- Basic settings card with all sample Grafana data
- Supplementary redirect URLs list with the sample URL + Add button
- Security toggles card with all 6 toggles in Grafana's configuration (PKCE on, strict redirect on, localhost off, consent on, prefer short off, legacy crypto off)
- Footer with Discard + Save changes (disabled)
- Render the rename warning state as a second variant: system name changed
- Render the legacy-crypto-enable confirm modal as a third variant
