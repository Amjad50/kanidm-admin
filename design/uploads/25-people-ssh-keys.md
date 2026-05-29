# 25 — People: SSH Keys Tab

The SSH Keys tab on the person detail page. Lists registered SSH public keys (with labels and fingerprints), allows adding new keys, deleting existing ones.

## Purpose

Manage the SSH public keys associated with a person. Keys are used by kanidm-integrated Linux/Unix systems to authorize SSH access. Each key has a unique label (per-account constraint).

## Layout

Tab content inside the person detail page:

```
┌─────────────────────────────────────────────────────────────────────┐
│ SSH Keys                                                            │
│                                                                     │
│ Public keys                                                         │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Label              Fingerprint              Added       Actions │ │
│ │─────────────────────────────────────────────────────────────────│ │
│ │ laptop_ed25519     SHA256:4FZJYr…           2025-11-03  ✕      │ │
│ │ workstation_rsa    SHA256:bxQ8mF…           2025-06-21  ✕      │ │
│ │ yubikey_5c         SHA256:Vk2P9N…           2025-03-15  ✕      │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ Add a key                                                           │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Label *                                                         │ │
│ │ ┌─────────────────────────────────────────────────────────┐     │ │
│ │ │ work_laptop                                             │     │ │
│ │ └─────────────────────────────────────────────────────────┘     │ │
│ │ A short identifier for this key. Unique per person.             │ │
│ │                                                                 │ │
│ │ Public key *                                                    │ │
│ │ ┌─────────────────────────────────────────────────────────┐     │ │
│ │ │ ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIK7…                │     │ │
│ │ │                                                         │     │ │
│ │ │                                                         │     │ │
│ │ └─────────────────────────────────────────────────────────┘     │ │
│ │ Paste a single OpenSSH-format public key (ssh-ed25519,          │ │
│ │ ssh-rsa, ecdsa-sha2-*).                                         │ │
│ │                                                                 │ │
│ │ Detected: ssh-ed25519 (256 bits)                                │ │
│ │                                                                 │ │
│ │ [Cancel]   [Add key]                                            │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Tab content sections

### Public keys list

A table or card-list of registered keys. Source: `GET /v1/person/{id}/_ssh_pubkeys` — returns each key as the string `{label}: {ssh-key-line}\n`. The UI parses by splitting on the first `: ` to get `[label, key_line]`, then extracts the algorithm (`ssh-ed25519`, `ssh-rsa`, etc.) and base64 portion from the key line. No structured shape; no timestamps; no fingerprints from the API.

Columns:
- **Label** — the unique identifier the user gave it (`laptop_ed25519`, etc.). Monospace.
- **Fingerprint** — SHA256 fingerprint, monospace, truncated with copy button. Click reveals full fingerprint. **API note:** kanidm does NOT return fingerprints; the UI must compute them client-side from the raw key data (parse the SSH key line's base64 portion, SHA256-hash the decoded bytes, base64-encode the digest, format as `SHA256:xxx`).
- **Added** — date the key was registered. **API note:** kanidm does NOT track this. The API returns only `{label}: {full-ssh-key-line}` per key with NO timestamps. Either omit this column entirely OR derive a "first seen by this UI" timestamp client-side (low value). Recommendation: omit the column.
- **Actions** — "×" delete button (Lucide `Trash2`)

Each row is hoverable; hover highlights the row. Delete opens a small confirm dialog: "Delete SSH key `laptop_ed25519`? This won't sign anyone out, but the key will no longer authorize new connections." with Cancel / Delete buttons.

If no keys: empty state in the table area: "No SSH keys registered." with the Add form below (always visible).

### Add a key form

Always visible below the list (not collapsed by default; this is one of the most common operations).

**Label field:**
- Required
- Validation: non-empty, unique within this person's keys, kanidm-acceptable character set (lowercase letters, digits, underscore, hyphen typically — match the server's constraint)
- Placeholder: `work_laptop`
- Helper: "A short identifier for this key. Unique per person."
- Inline error if duplicate: "This label is already used. Try a different one."

**Public key field:**
- Required, large textarea (4-6 rows)
- Monospace font
- Placeholder: `ssh-ed25519 AAAAC3NzaC1lZDI1NTE5...` (truncated to fit)
- Helper: "Paste a single OpenSSH-format public key (ssh-ed25519, ssh-rsa, ecdsa-sha2-*)."
- **Client-side parse on blur or as-you-type (debounced):** detect the key type and bit length, show below the textarea: "Detected: ssh-ed25519 (256 bits)" or "Detected: ssh-rsa (3072 bits)" — green checkmark
- Invalid key: "⚠ Could not parse this as an SSH public key. Make sure you copied a single line beginning with `ssh-`, `ecdsa-`, or `sk-`."
- Strip leading/trailing whitespace, accept the OpenSSH format with optional comment at end

**Footer:**
- Cancel — clears the form
- Add key — primary, disabled until both fields valid. On click:
  - Calls `POST /v1/person/{id}/_ssh_pubkeys` with `{label, key}`
  - On success: clears form, shows toast "SSH key added: work_laptop", refreshes list above
  - On error: inline error per server response (e.g., 409 duplicate label, 422 invalid key)

## States

- **Loading keys:** skeleton rows in the list.
- **Empty:** "No SSH keys registered." in the list area; add form below.
- **Adding:** Add button → spinner + "Adding…"; inputs read-only.
- **Adding success:** form clears, toast appears, list refreshes.
- **Adding error:** inline error or toast.
- **Deleting:** small confirm dialog, then row briefly shows a "Removing…" state, then disappears.

## Sample data

Use Alice Smith's SSH keys from `_sample-data.md`:
- `laptop_ed25519` — SHA256:4FZJYr… — 2025-11-03 — ssh-ed25519
- `workstation_rsa` — SHA256:bxQ8mF… — 2025-06-21 — ssh-rsa
- `yubikey_5c` — SHA256:Vk2P9N… — 2025-03-15 — ecdsa-sha2-nistp256

For the add form, show a partially-filled form with:
- Label: `work_laptop` (typed)
- Public key: a sample `ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIK7...` line
- Detected indicator: "ssh-ed25519 (256 bits)"

For Jane Doe (new person), show empty state: "No SSH keys registered." with the add form below.

## Edge cases

- **Pasted key with surrounding whitespace:** trim before sending.
- **Pasted key with newlines (full pubkey file content):** if the textarea receives multiple lines, take only the first non-empty line.
- **Pasted key with optional comment at end:** kanidm accepts the comment; preserve or strip per server policy.
- **Multi-line ssh key (e.g., armored PEM):** reject in the parser — "This looks like a private key or certificate. Paste the single-line public key only."
- **Long label (e.g., 100 chars):** allow up to kanidm's limit; truncate visually in the table with full label on hover.
- **Deleting the only key:** allowed; the list becomes empty.
- **Privilege session required:** add/delete actions require an active privilege session; reauth modal appears if expired.

## Tone

Functional. The SSH keys tab is for power users (this is the audience who knows what `ssh-ed25519` means). Minimal hand-holding except for the inline parse-validation feedback, which is helpful for everyone.

## Mockup elements to render

- Tab content with "SSH Keys" heading
- Public keys table with all 3 sample keys
- Add a key form with sample partial input (label `work_laptop` + sample ssh-ed25519 key) and detected indicator
- Render an empty state variant for Jane Doe: empty table + add form
- Render an error state on the public key textarea: invalid key, danger border, error message "Could not parse this as an SSH public key…"
