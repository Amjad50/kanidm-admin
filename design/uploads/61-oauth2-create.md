# 61 — OAuth2 Apps: Create New Application

A wizard or single-form to create a new OAuth2 application. Per kanidm CLI: `create` (confidential) vs `create-public` (public/PKCE), then name + displayname + origin.

## Purpose

Create an OAuth2 application. Decide upfront whether it's confidential (has a client secret) or public (PKCE-only, e.g., SPAs and native apps). Capture the minimum required fields. After creation, navigate to the new app's detail for further configuration (scope maps, etc.).

## Layout

A multi-step wizard is appropriate here because the type choice changes downstream config. A single-page form with type-toggle at the top also works (simpler, fewer clicks). Designer's call.

### Wizard variant (recommended for Stripe/Cloudflare, more illustrative)

**Step 1 — Type:**

```
┌─────────────────────────────────────────────────────────────────────┐
│ OAuth2 Apps > Create application                                    │
│                                                                     │
│ Create OAuth2 application                                           │
│                                                                     │
│ Step 1 of 2: Choose a client type                                   │
│                                                                     │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ ⓘ Confidential client                                        │    │
│ │ Has a client secret. Used for server-side apps that can     │    │
│ │ safely keep credentials secret (Grafana, Nextcloud, Gitea). │    │
│ └──────────────────────────────────────────────────────────────┘    │
│                                                                     │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ ⓘ Public client                                              │    │
│ │ No client secret. PKCE required. Used for SPAs and native   │    │
│ │ apps that can't safely store secrets.                       │    │
│ └──────────────────────────────────────────────────────────────┘    │
│                                                                     │
│                                          [Cancel]   [Continue]      │
└─────────────────────────────────────────────────────────────────────┘
```

Two large radio-card options, prominent. Each has icon + heading + description.

**Step 2 — Details:**

```
┌─────────────────────────────────────────────────────────────────────┐
│ OAuth2 Apps > Create application                                    │
│                                                                     │
│ Create OAuth2 application                                           │
│                                                                     │
│ Step 2 of 2: Details                                                │
│ Confidential client                                                 │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ System name *                                                   │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ grafana                                                     │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ Used in URLs and tokens. Lowercase, no spaces. You can         │ │
│ │ rename later, but apps using this name must be reconfigured.   │ │
│ │                                                                 │ │
│ │ Display name *                                                  │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Grafana                                                     │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ Shown on consent screens and the apps listing.                  │ │
│ │                                                                 │ │
│ │ Origin URL *                                                    │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ https://grafana.example.com                                 │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ The application's primary URL. Becomes the landing URL and the │ │
│ │ default origin for redirects. Use HTTPS in production.         │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│                                  [Back] [Cancel] [Create application]│
└─────────────────────────────────────────────────────────────────────┘
```

After creation, navigate to `/oauth2/grafana` with toast "Application created: Grafana".

For confidential clients, the post-creation flow shows the client secret immediately (one-time-show pattern) on the detail page Secret card (see screen 64). The wizard can either:
- (A) Show the secret in a step 3 of the wizard ("Save your client secret"), then Continue → detail page
- (B) Navigate to detail page where the secret is revealed as the first card

Designer's call. Option B is cleaner; option A is more explicit. Either way, the secret is shown exactly once.

### Single-form variant (recommended for Linear)

All fields on one page with a type toggle at top:

```
┌─────────────────────────────────────────────────────────────────────┐
│ OAuth2 Apps > Create application                                    │
│                                                                     │
│ Create OAuth2 application                                           │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Client type *                                                   │ │
│ │ (•) Confidential — has a client secret (server-side apps)       │ │
│ │ ( ) Public — no secret, PKCE required (SPAs, native apps)       │ │
│ │                                                                 │ │
│ │ System name *                                                   │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ grafana                                                     │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │                                                                 │ │
│ │ Display name *                                                  │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Grafana                                                     │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │                                                                 │ │
│ │ Origin URL *                                                    │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ https://grafana.example.com                                 │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │                                                                 │ │
│ │              [Cancel]            [Create application]           │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Form fields (regardless of variant)

### Client type (radio choice)

- Confidential (default for new admin) — calls `POST /v1/oauth2/_basic`
- Public — calls `POST /v1/oauth2/_public`

### System name

- Label "System name *"
- Required
- Validation: lowercase letters, digits, underscore, hyphen (kanidm constraints)
- Placeholder: `grafana`
- Helper: "Used in URLs and tokens. Lowercase, no spaces. You can rename later, but apps using this name must be reconfigured."
- Inline validation

### Display name

- Label "Display name *"
- Required
- Placeholder: `Grafana`
- Helper: "Shown on consent screens and the apps listing."

### Origin URL

- Label "Origin URL *"
- Required
- Input type `url`
- Placeholder: `https://grafana.example.com`
- Helper: "The application's primary URL. Becomes the landing URL and the default origin for redirects. Use HTTPS in production."
- Inline validation: must be a parseable URL with scheme
- Warning if HTTPS not used: "⚠ Non-HTTPS URLs are not recommended for production." (Don't block save.)

## Footer (single-form variant)

- Cancel — navigate to `/oauth2`
- Create application — primary, disabled until all required fields valid
- On success: navigate to `/oauth2/grafana` with success toast. For confidential, the basic secret is revealed on the detail page (one-time-show).
- On 409: inline error on system name "An application with this name already exists."
- On 422: per-field error from server.

## States

- **Idle:** all fields empty (or pre-filled per wizard navigation).
- **Submitting:** Create button spinner; inputs read-only.
- **Field error:** inline.
- **Success:** navigate to detail.

## Sample data

For the mockup, use Grafana (new creation):
- Client type: Confidential
- System name: `grafana`
- Display name: `Grafana`
- Origin URL: `https://grafana.example.com`

For a second variant, use a public client:
- Client type: Public
- System name: `homelab-spa`
- Display name: `Homelab Dashboard (SPA)`
- Origin URL: `https://dash.example.com`

## Edge cases

- **System name reserved:** kanidm may reserve some names. Show server error inline.
- **Origin URL parsing failure:** require a valid URL format. Show inline error "Enter a full URL including https://".
- **Duplicate system name:** server returns 409.
- **Switching client type mid-form (single-form variant):** keep other field values; just update the API endpoint that will be called on submit.

## Mockup elements to render

- Choose either the wizard variant or single-form variant per design system; render that
- Breadcrumb "OAuth2 Apps > Create application"
- Title + step indicator if wizard
- Form with sample Grafana data
- Cancel + Create application buttons
- For wizard, render step 1 (type choice) AND step 2 (details) as two separate mockups
- Render the secret-reveal variant (post-creation, basic secret shown) — either as a wizard step 3 or as the destination detail page
