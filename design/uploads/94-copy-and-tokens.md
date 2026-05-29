# 94 — Copy and Tokens (Cross-Cutting Pattern)

The universal pattern for handling values that are copy-paste-relevant (UUIDs, SPNs, fingerprints) and for one-time-show secrets (basic secrets, RADIUS shared secrets, reset URLs).

## Purpose

Make it trivially easy to copy any identifier the admin will use in another tool. Make it deliberately careful to handle sensitive secrets that should only be shown once.

## Pattern 1 — Copyable identifiers (always visible)

For non-sensitive identifiers that the admin needs to copy frequently: UUIDs, SPNs, system names, public key fingerprints, key IDs.

**Appearance:**
- Value displayed inline, monospace font, in normal text color
- A small copy button (Lucide `Copy` icon, 14-16px) immediately to the right of the value
- Copy button has a tooltip "Copy {label}" — e.g., "Copy UUID"
- On click:
  - Value is copied to clipboard
  - Copy icon briefly changes to a checkmark (Lucide `Check`) for ~1.5s
  - Optional very brief tooltip "Copied" near the cursor
  - No toast — copy is a frequent action and toasts would be noisy

**Examples in the UI:**
- UUID display in identity sections: `7c3a8b4e-2f1d-4c5e-9a8b-1f2e3d4c5b6a [📋]`
- SPN display: `alice.smith@idm.example.com [📋]`
- SSH fingerprint: `SHA256:4FZJYr... [📋]`
- OAuth2 key ID: `key-7f3a2c1d [📋]`

For longer values (e.g., UUID), the displayed value may be truncated with `…` in lists; the full value is copied. Tooltip on hover shows the full value.

## Pattern 2 — Token / secret display (one-time-show)

For values that are sensitive AND deliberately shown only at creation time: OAuth2 basic secret on rotation, RADIUS shared secret on generation, intent-token reset URL on creation, generated backup codes.

**Appearance (initial reveal state, right after generation):**

```
┌──────────────────────────────────────────────────────────┐
│ ⚠ This is the only time the secret will be shown.       │
│   Copy it now and store it securely.                    │
│                                                          │
│ ┌──────────────────────────────────────────────────┐    │
│ │ xK8mP2qF9vN4jH7tR1yC6wA3eL5sB0gD             [📋] │   │
│ └──────────────────────────────────────────────────┘    │
│                                                          │
│ [I've saved the secret]                                  │
└──────────────────────────────────────────────────────────┘
```

- Strong warning callout at top with `--warning` semantic styling
- Value displayed in monospace, larger than usual (so the user can confirm what they're copying)
- Copy button prominently visible
- Acknowledgement button "I've saved the secret" — confirms the admin has stored the value and moves the UI to the post-acknowledgement state where the value is no longer visible

**Appearance (post-acknowledgement OR re-entry to the page):**

```
┌──────────────────────────────────────────────────────────┐
│ Status: Configured                                       │
│                                                          │
│ The secret cannot be retrieved. To get a new value, use  │
│ the "Regenerate" action — this will invalidate the       │
│ current secret.                                          │
│                                                          │
│ [Regenerate]                                             │
└──────────────────────────────────────────────────────────┘
```

For values where kanidm's API DOES allow re-retrieval (like OAuth2 basic secret via `_basic_secret`), use a mask+reveal pattern instead of a hidden state. See Pattern 3.

## Pattern 3 — Mask with reveal (re-retrievable secrets)

For sensitive values that the API allows retrieving anytime (e.g., OAuth2 basic secret, intent-token URL while still in current generation flow).

**Appearance:**

```
┌──────────────────────────────────────────────────────────┐
│ Current secret                                           │
│ ┌──────────────────────────────────────────────────┐    │
│ │ ••••••••••••••••••••••••••••••••••••••  [👁] [📋] │   │
│ └──────────────────────────────────────────────────┘    │
│                                                          │
│ Reveal to copy. Treat this as sensitive.                │
└──────────────────────────────────────────────────────────┘
```

- Value displayed as masked dots by default
- Eye icon (Lucide `Eye` / `EyeOff`) toggles reveal
- Copy icon always available (copies the actual value even when masked — no need to reveal to copy)
- Helper text below: "Reveal to copy. Treat this as sensitive."

When revealed:
- Dots replaced with actual value
- Eye icon changes to `EyeOff` indicating "click to mask"

## Pattern 4 — QR code

For values that should also be scannable from a phone (e.g., the intent-token reset URL).

**Appearance:**
- A square QR code (~120-160px) encoding the full value
- Caption below: "Scan to open on a phone." (for URLs) or "Scan with your authenticator app." (for TOTP secrets)
- Aligned with the rest of the secret display block

For the OAuth2 image upload screen (not a token use case), QR codes are not needed.

## Pattern 5 — Copy with type indicator

For values that span multiple lines (e.g., SSH public keys, JSON blobs).

**Appearance:**
- Block-style display, monospace, in a `--code-bg` container
- Copy button top-right of the block
- Optional "Format" / "Pretty-print" toggle for JSON
- Optional download button for very long values (e.g., a JSON CA list)

## Sample data references

- UUID copy example: `7c3a8b4e-2f1d-4c5e-9a8b-1f2e3d4c5b6a` (Alice Smith)
- SPN copy example: `alice.smith@idm.example.com`
- OAuth2 basic secret example (one-time-show on regeneration): `kFn3pV9wQ2bT8nX1aL5cR4jH7sM6dY8fZ2gH4tQ5xA9eC1nB3vK7p`
- RADIUS shared secret example: `xK8mP2qF9vN4jH7tR1yC6wA3eL5sB0gD`
- Reset URL example: `https://idm.example.com/ui/reset?token=eyJhbGciOiJFUzI1NiIs...`
- SSH key fingerprint: `SHA256:4FZJYr...`

## Accessibility

- All copy buttons must have an aria-label like `aria-label="Copy UUID"`
- The "Copied" feedback should also be announced to screen readers via aria-live region
- Mask/reveal toggle buttons must indicate state via aria-pressed and label

## Design system variations

- **Linear:** copy button is a small ghost icon, minimal padding, blends with text
- **Cloudflare:** copy button is slightly more visible, possibly with a small ring on hover
- **Stripe:** copy button is a small secondary button with subtle background, more prominent

## Mockup elements to render

Render 5 distinct copy-and-token variants:

1. **Inline UUID with copy** — a key-value row "UUID  7c3a8b4e-...  [📋]"
2. **One-time-show secret (post-generation)** — a card with warning callout, monospace secret display, copy button, "I've saved" acknowledgement
3. **Mask-with-reveal** — current OAuth2 secret in masked state with eye toggle + copy
4. **QR code variant** — a reset URL display with monospace URL + QR code below + caption
5. **Multi-line code block** — an SSH public key in a `--code-bg` container with top-right copy button
