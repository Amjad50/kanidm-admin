# 29 — People: Delete Confirm Modal

The confirmation modal shown when an admin clicks Delete on a person. Type-to-confirm pattern to prevent accidents.

## Purpose

Make deletion intentional. Show the consequences clearly. Require the admin to type the person's SPN to confirm. After deletion, the person goes to the recycle bin (kanidm soft-delete) — not permanent destruction. The UI should communicate this.

## Layout

A modal overlay using the design system's modal spec (medium size, ~520-560px wide).

```
   ┌──────────────────────────────────────────────────┐
   │  Delete person                            [×]    │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  ⚠ You're about to delete:                       │
   │                                                  │
   │  ┌──┐                                            │
   │  │AS│  Alice Smith                               │
   │  └──┘  alice.smith@idm.example.com               │
   │                                                  │
   │  What happens:                                   │
   │   ▸ Alice will be signed out from all devices    │
   │     immediately.                                 │
   │   ▸ Her account moves to the recycle bin and is  │
   │     recoverable for 7 days.                      │
   │   ▸ All group memberships and OAuth2 scope maps  │
   │     referencing her are kept until she's purged. │
   │   ▸ Active OAuth2 tokens she has signed are      │
   │     revoked.                                     │
   │                                                  │
   │  Type the SPN to confirm:                        │
   │  alice.smith@idm.example.com                     │
   │  ┌────────────────────────────────────────────┐  │
   │  │                                            │  │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]       [Delete person]      │
   └──────────────────────────────────────────────────┘
```

## Modal content

**Header:**
- Title: "Delete person"
- Close button (×) — equivalent to Cancel

**Body:**

*Lead-in:*
"⚠ You're about to delete:" with a danger icon (Lucide `AlertTriangle`).

*Identity card:*
A compact representation of the person being deleted:
- 40px avatar with initials
- Display name (e.g., "Alice Smith") in primary text
- SPN (`alice.smith@idm.example.com`) in monospace secondary

*"What happens" section:*
A bulleted list explaining consequences:
- ▸ "{Name} will be signed out from all devices immediately." (their sessions are destroyed)
- ▸ "Their account moves to the recycle bin and is recoverable for 7 days." (kanidm soft-delete)
- ▸ "All group memberships and OAuth2 scope maps referencing them are kept until they're purged." (kanidm reference semantics)
- ▸ "Active OAuth2 tokens they have signed are revoked." (if applicable)

The list should be specific to what kanidm actually does on delete. Avoid hand-waving.

*Type-to-confirm input:*
- Label: "Type the SPN to confirm:"
- Below the label, the exact SPN shown in monospace, copyable (with a small copy button to the right — accessibility hint: don't make the admin type 30+ characters manually, but DO make them deliberately confirm. Allowing copy is fine — the friction is in the deliberate paste, not in pure typing.)
- Input field below — must match the SPN exactly (case-sensitive) before the Delete button enables

**Footer:**
- Cancel — secondary
- Delete person — danger primary, disabled until input matches the SPN

## Confirm input behavior

- The input field uses the same monospace font as the SPN display.
- As the admin types, validate character-by-character. When the input exactly equals the SPN, the Delete button enables with a subtle visual cue (color shift from disabled to active).
- If the admin pastes (Cmd+V) the SPN, it enables instantly.
- No error states for mismatched input; just keep the Delete button disabled.

## States

- **Idle:** modal open, Delete button disabled, input empty.
- **Input matches:** Delete button enabled.
- **Deleting:** Delete button shows spinner + "Deleting…", Cancel still works.
- **Deletion success:** modal closes, toast "Person deleted: alice.smith@idm.example.com — recoverable from the recycle bin for 7 days." Then the parent page (person detail or list) handles navigation:
  - From detail: navigate back to `/people` list.
  - From list (bulk or row delete): the row is removed from the table.
- **Deletion error:** modal stays open, inline error at the top "Could not delete: {server message}". Cancel re-enabled.

## Sample data

Use Alice Smith:
- Display name: "Alice Smith"
- SPN: `alice.smith@idm.example.com`
- The required confirm text: `alice.smith@idm.example.com`

For the input mockup, show the partially-typed state: `alice.smit` (Delete still disabled).
For the enabled state, show: `alice.smith@idm.example.com` (Delete enabled).

## Edge cases

- **Privilege session expired:** clicking Delete (after confirm matches) opens the reauth modal (screen 08) BEFORE the actual delete API call. After reauth succeeds, the delete proceeds.
- **Deleting self:** if the admin is deleting their own account, add an extra-warning callout: "⚠ This is your account. You will be signed out and will lose access to kanidm." Optionally require a second confirm step ("Yes, sign me out and delete my account").
- **Deleting a member of `idm_admins`:** add an extra-warning callout: "⚠ This is a member of the idm_admins group. Make sure another admin can still administer kanidm."
- **Person was last-modified very recently:** no special handling, but a tiny "Last modified 2 hours ago" subtitle near the identity block could help the admin reconsider if they're deleting fresh data.
- **Bulk delete (from people list):** same modal pattern but the identity card becomes a small list of all selected SPNs ("Alice Smith, Bob Jones, +3 more"). The type-to-confirm uses the literal text `DELETE` (uppercase) instead of a specific SPN, with caption "Type DELETE to confirm deletion of 5 people." This is a slightly different pattern for bulk — fewer characters but clearly destructive.

## Tone

Serious without being alarmist. The bulleted "What happens" section is factual; don't add scare quotes. Deletion is recoverable for 7 days (kanidm soft-delete), so the danger is real but not catastrophic — communicate that.

## Mockup elements to render

- Modal overlay with backdrop blur
- Modal header "Delete person" + close button
- Body with warning icon + lead-in
- Identity card for Alice Smith
- "What happens" bulleted list
- Type-to-confirm section with SPN displayed in monospace + copy button
- Empty input field below
- Footer with Cancel + Delete person (Delete disabled)
- Render a second variant with the input filled with `alice.smith@idm.example.com` and the Delete button enabled in danger color
- Render the bulk-delete variant: identity card replaced with a list of 5 SPNs, type-DELETE pattern, "Delete 5 people" button
