# 80 — Self Page

The signed-in admin's own profile page. Mostly read-only; supports re-authentication and links to session management.

## Purpose

Show the current admin what kanidm knows about them: SPN, UUID, mails, groups, session state. Provide a quick link to re-authenticate (privilege session) and to view all their sessions.

This is NOT a fully editable profile page — for editing their own attributes, the admin can navigate to their own entry under `/people/{themselves}` (which uses the same edit screen 23). Some users might prefer the Self page for "view-only" needs.

## Layout

Inside the app shell. Accessed via the user menu → "View profile" or directly at `/self`.

```
┌─────────────────────────────────────────────────────────────────────┐
│ Your profile                                                        │
│                                                                     │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ [AD]   System Administrator                                  │    │
│ │        admin@idm.example.com                                 │    │
│ │        ✉ admin@example.com                                   │    │
│ │                                                              │    │
│ │   [Re-authenticate] [Edit profile] [⋯]                       │    │
│ └──────────────────────────────────────────────────────────────┘    │
│                                                                     │
│ ┌────────────────────────────────────────────────────────────────┐  │
│ │ Identity                                                       │  │
│ │                                                                │  │
│ │ UUID         00000000-0000-0000-0000-000000000000   [copy]     │  │
│ │ Username     admin                                             │  │
│ │ SPN          admin@idm.example.com                  [copy]     │  │
│ │ Display name System Administrator                              │  │
│ │ Emails       ★ admin@example.com (primary)                     │  │
│ │                                                                │  │
│ └────────────────────────────────────────────────────────────────┘  │
│                                                                     │
│ ┌────────────────────────────────────────────────────────────────┐  │
│ │ Group memberships                                              │  │
│ │                                                                │  │
│ │  ▣ idm_admins                                                  │  │
│ │  ▣ system_admins                                               │  │
│ │                                                                │  │
│ └────────────────────────────────────────────────────────────────┘  │
│                                                                     │
│ ┌────────────────────────────────────────────────────────────────┐  │
│ │ Current session                                                │  │
│ │                                                                │  │
│ │ Signed in       38 minutes ago                                 │  │
│ │ Expires         in 6h 22m                                      │  │
│ │ Purpose         Read-write                                     │  │
│ │ Privileged      ● Active for 22 minutes more                   │  │
│ │                                                                │  │
│ │ → View all sessions                                            │  │
│ └────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- No breadcrumb (root-level page) OR breadcrumb `Self > Profile` per design
- Title: "Your profile"

## Identity card (top)

Same pattern as person detail (screen 22):
- Avatar (64px) — initials "SA" for "System Administrator"
- Display name: "System Administrator"
- SPN: `admin@idm.example.com` (monospace)
- Primary email
- Action buttons:
  - **Re-authenticate** — primary or accent; opens reauth modal (screen 08). Useful when admin wants to refresh privilege session before doing a batch of sensitive operations.
  - **Edit profile** — opens person edit (screen 23) at `/people/admin/edit`
  - ⋯ kebab: Identity verification flow (kanidm's `self identify-user` flow), Copy UUID, Sign out

## Identity section

Key-value list:
- UUID (copy)
- Username
- SPN (copy)
- Display name
- Emails (with primary indicator)

Read-only; the "Edit profile" button leads to the editable view.

## Group memberships section

Same pattern as person detail:
- List of group chips (clickable, navigate to group detail)
- For the admin user: typically `idm_admins`, `system_admins`

No "+ Add to group" button here — adding self to a new group should be done explicitly via the People > Edit flow, not casually from the profile page.

## Current session section

Read-only summary of the active session:
- Signed in: relative time
- Expires: relative time
- Purpose: read-write / read-only
- Privileged: ● Active for {N} minutes more / ○ Not active. If not active, show a "Re-authenticate" link inline.

Link at bottom: "→ View all sessions" → navigates to screen 81.

## States

- **Loading:** skeleton.
- **Read-only viewer (no edit privileges):** "Edit profile" disabled with tooltip.
- **Identity verification flow (from kebab):** opens a separate flow — the kanidm `self identify-user` exchange. This is a 2-party flow where two users mutually verify each other's identities by exchanging codes. The UI for this is a small modal:

```
   ┌──────────────────────────────────────────────────┐
   │  Verify another person's identity         [×]    │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  Use this when you need to confirm someone's     │
   │  identity in real-time (e.g., over a phone call).│
   │  Both of you should be in this UI at the same    │
   │  time.                                           │
   │                                                  │
   │  Step 1 — Get the other person's identity        │
   │  Have them give you their username or UUID.      │
   │                                                  │
   │  ┌────────────────────────────────────────────┐  │
   │  │ alice.smith@idm.example.com                │  │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   │  [ Start verification ]                          │
   │                                                  │
   └──────────────────────────────────────────────────┘
```

Then the flow asks for a TOTP code Alice reads aloud, similar to kanidm's existing identity-verification UI. This whole sub-flow is somewhat out-of-band for the core admin UI; mention it briefly in this brief but don't deeply spec — it's a stretch feature.

## Sample data

For `admin`:
- Display name: System Administrator
- SPN: `admin@idm.example.com`
- UUID: `00000000-0000-0000-0000-000000000000`
- Username: `admin`
- Primary email: `admin@example.com`
- Groups: `idm_admins`, `system_admins`
- Session: signed in 38 min ago, expires in 6h 22m, read-write, privileged active for 22 more minutes

## Edge cases

- **Admin's own account has no displayname:** show SPN as the large name.
- **Admin's session expired during view:** redirect to login with a "Your session expired" banner.
- **Admin is also being deleted from another tab:** show banner "Your account is being deleted. You will be signed out shortly."

## Mockup elements to render

- Page title "Your profile"
- Identity card for admin with avatar, display name, SPN, primary email, action buttons
- Identity section with all key-values
- Group memberships chips (idm_admins, system_admins)
- Current session card with all fields and privileged indicator active
- Link "→ View all sessions"
- Render a second variant with privileged session NOT active: dot is outline, inline "Re-authenticate" link visible
