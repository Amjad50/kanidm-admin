# 22 — People: Detail View

The detail page for a single person. Tabbed layout covering overview, credentials, SSH keys, RADIUS, sessions, and validity.

## Purpose

Show everything administered about a person in one place. Group related operations into tabs so the page never feels overwhelming, but the most-used info (overview) is the default tab.

## Layout

Inside the app shell. Main content area:

```
┌─────────────────────────────────────────────────────────────────────┐
│ People > alice.smith@idm.example.com                                │
│                                                                     │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ ⓐ Alice Smith                  Active                        │    │
│ │   alice.smith@idm.example.com                                │    │
│ │   ✉ alice.smith@example.com                                  │    │
│ │                                                              │    │
│ │   [Edit] [Generate reset link] [⋯ more]                      │    │
│ └──────────────────────────────────────────────────────────────┘    │
│                                                                     │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ Overview │ Credentials │ SSH Keys │ RADIUS │ Sessions │ Validity │ │
│ ├──────────────────────────────────────────────────────────────┤    │
│ │                                                              │    │
│ │  (Tab content here, e.g., Overview)                          │    │
│ │                                                              │    │
│ └──────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- Breadcrumb: `People > alice.smith@idm.example.com` (last segment is the SPN, not the displayname — exact match the URL slug)

## Identity card (top of page)

A prominent card showing the core identity:
- Large avatar (~64px, initials or uploaded image)
- Display name (large, prominent): "Alice Smith"
- SPN (monospace, secondary): "alice.smith@idm.example.com"
- Primary email (subdued, with mail icon): "alice.smith@example.com"
- Status badge: "Active" (per validity state)
- Right side or below: action buttons:
  - **Edit** — primary or secondary depending on design; opens `23-people-edit.md` flow
  - **Generate reset link** — secondary, opens `24-people-credentials.md` modal/flow
  - **⋯ more** — kebab dropdown with: Destroy all sessions, Delete (red), Copy UUID

## Tabs

Horizontal tab bar below the identity card. Tab labels:

1. **Overview** (default) — summary view, see below
2. **Credentials** — links to screen 24's flow inline (status + reset link generator)
3. **SSH Keys** — screen 25 inline
4. **RADIUS** — screen 26 inline
5. **Sessions** — screen 27 inline
6. **Validity** — screen 28 inline

Tab state preserved in URL: `/people/alice.smith?tab=credentials`. Browser back/forward navigates between tabs.

Each tab is a separate brief; this file describes the **Overview tab**.

## Overview tab content

```
┌────────────────────────────────────────────────────────────────┐
│ Identity                                                       │
│                                                                │
│ UUID         7c3a8b4e-2f1d-4c5e-9a8b-1f2e3d4c5b6a   [copy]     │
│ Username     alice.smith                                       │
│ SPN          alice.smith@idm.example.com           [copy]      │
│ Display name Alice Smith                                       │
│ Legal name   Alice Marion Smith                                │
│                                                                │
│ Emails                                                         │
│   ★ alice.smith@example.com  (primary)                         │
│     alice@example.com                                          │
│                                                                │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ Group memberships                                              │
│                                                                │
│   ▣ idm_admins                                                 │
│   ▣ developers                                                 │
│   ▣ vpn_users                                                  │
│                                                                │
│   [+ Add to a group]                                           │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ Validity                                                       │
│                                                                │
│ Valid from   any time                                          │
│ Expires      never                                             │
│                                                                │
│ → Edit on Validity tab                                         │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ Credentials summary                                            │
│                                                                │
│ Primary       Password + TOTP                                  │
│ Passkeys      2 registered                                     │
│ SSH keys      3 registered                                     │
│ RADIUS        configured                                       │
│                                                                │
│ → Manage on Credentials tab                                    │
└────────────────────────────────────────────────────────────────┘
```

### Identity section

Key-value list of read-only attributes:
- UUID (monospace, copy button)
- Username (the `name` attribute)
- SPN (monospace, copy button)
- Display name
- Legal name (if set; "—" if not)
- Emails: a small list with star indicator on the primary. If no emails, "No emails set" in subdued tone.

### Group memberships section

Direct memberships (not transitive) — from the `directmemberof` attr on the person entry. Each membership is a small pill / chip showing the group name, clickable to navigate to that group's detail page. Right edge of each chip has a "×" to remove (with privilege check / reauth gate).

Below the chips: "+ Add to a group" button — opens a small picker (search-as-you-type group selector). Group add/remove are MUTATIONS ON THE GROUP, not on the person:
- Add: `POST /v1/group/{group_name}/_attr/member` body `["<person_spn>"]`
- Remove: `DELETE /v1/group/{group_name}/_attr/member` body `["<person_spn>"]`

After the mutation, refetch the person entry to update the displayed memberships.

If no memberships: "Not a member of any group" + the add button.

### Validity section

Brief read-only summary:
- Valid from: "any time" if not set, or a formatted date
- Expires: "never" if not set, or a formatted date with relative time ("expires in 3 months" or "expired 2 days ago" in danger color)

Link to Validity tab for edits.

### Credentials summary section

Brief read-only summary:
- Primary credential type ("Password + TOTP", "Password only", "Passkey only", "None set")
- Passkeys count
- SSH keys count
- RADIUS status

Link to Credentials tab for management.

## States

- **Loading:** skeleton identity card + skeleton tabs + skeleton cards. Tab labels visible.
- **Person not found (URL with bad ID):** show a 404-style empty state: "Person not found" + "Back to People" button.
- **Read-only viewer (no edit privileges):** action buttons hidden or disabled. Tabs still navigable but their content is read-only.
- **Action requires privilege session:** clicking Edit, Generate reset link, or any other action opens the reauth modal (screen 08) if privilege session is expired.

## Sample data

Use exactly Alice Smith's data from `_sample-data.md`:
- UUID: `7c3a8b4e-2f1d-4c5e-9a8b-1f2e3d4c5b6a`
- Username: `alice.smith`
- SPN: `alice.smith@idm.example.com`
- Display name: Alice Smith
- Legal name: Alice Marion Smith
- Primary email: `alice.smith@example.com`
- Secondary email: `alice@example.com`
- Groups: `idm_admins`, `developers`, `vpn_users`
- Validity: from "any time", expires "never"
- Credentials summary: Password + TOTP, 2 passkeys, 3 SSH keys, RADIUS configured

## Edge cases

- **No emails set:** Emails section shows "No emails set" in subdued tone with an inline "+ Add email" button.
- **No displayname set:** show SPN as the large name in the identity card. The `displayname` attr can be absent in kanidm entries.
- **No legalname set:** show "—" or omit the field. Optional in kanidm.
- **Person who is the current signed-in user:** add a small banner at top of the page "This is your account. → View Self page instead." (links to screen 80). Editing your own account from here is allowed but the Self page may be more appropriate.
- **Recently expired account:** highlight the Validity section with a `--danger` left border. Status badge becomes "Expired". Detected from `account_expire` attr being in the past.
- **Account with `idm_admins` membership being modified:** show a warning toast on save: "You modified an admin-group account. Confirm this was intentional." (Not a confirm dialog — just informational.)
- **Person who is `entry-managed-by` for another entity:** show a small notice "This person manages N groups / N OAuth2 apps." with links.
- **Class is `service_account` not `person`:** the URL might match a service-account, not a person. kanidm uses the same `account` superclass for both. The UI should detect via `class` array — if `"person"` is NOT present but `"service_account"` is, redirect or show an error: "This is a service account, not a person."

## Mockup elements to render

- Breadcrumb
- Identity card with Alice Smith's data (avatar, display name, SPN, primary email, "Active" badge, action buttons)
- Tab bar with all 6 tabs, "Overview" active
- Overview content with all 4 sections (Identity, Group memberships, Validity summary, Credentials summary) using Alice's sample data
- All copy buttons visible
- Group membership chips for idm_admins, developers, vpn_users with × on each
- Render a second variant: Dave Locked (expired account) showing "Expired" badge in danger color, Validity section highlighted, status indicator on identity card
