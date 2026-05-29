# 93 — Destructive Confirmation (Cross-Cutting Pattern)

The universal pattern for confirming destructive or irreversible actions: delete, revoke, regenerate, purge.

## Purpose

Prevent accidental destructive actions. Provide clear consequences upfront. Use friction proportional to the irreversibility of the action.

## Confirmation tiers

Different destructive actions warrant different levels of confirmation. Match the tier to the consequence.

### Tier 1 — Simple confirm

For reversible or low-impact actions: removing a single email from a list, removing a member from a group, dismissing a toast, etc.

**Appearance:**
- Small popover or inline confirm: "Remove this email? Cancel | Remove"
- OR a one-click action with an undo toast: action happens immediately, toast appears: "Email removed. [Undo]" — undo button reverses the action within ~5 seconds

**Use:**
- Remove email from a person's mail list (with undo toast)
- Remove member from a group (small confirm OR undo toast)
- Remove a single SSH key
- Remove a scope map / claim map row

### Tier 2 — Confirm modal

For actions that are recoverable but inconvenient to undo: deleting individual entities that go to the recycle bin, revoking sessions, etc.

**Appearance:**
- Standard modal (per design system modal spec, ~480-560px wide)
- Title: "Delete person" / "Destroy session" / "Revoke key"
- Body: 1-2 sentences about consequences
- Footer: Cancel + danger primary action ("Delete", "Destroy", "Revoke")

**Use:**
- Destroying a single session
- Revoking a non-active signing key
- Removing scope map (some impact but recoverable)
- Disabling account policy

### Tier 3 — Type-to-confirm modal

For destructive actions that affect entire entities or are bulk: deleting a person, deleting a group, deleting an OAuth2 application, revoking the active signing key, purging members.

**Appearance:**
- Standard modal with extended content
- Title + warning icon
- Identity card showing what's being destroyed
- Bulleted "What happens" section explaining consequences
- Type-to-confirm input field requiring the user to type the entity's exact identifier
- Footer: Cancel + danger primary, primary disabled until input matches

**Examples (each has its own screen brief with specifics):**
- Delete person — screen 29
- Delete group — screen 46
- Delete OAuth2 application — screen 6A
- Revoke active signing key — within screen 67
- Purge all members of a group — within screen 44
- Bulk delete (N entities) — type `DELETE` instead of a specific name

**Common elements:**

```
   ┌──────────────────────────────────────────────────┐
   │  {Action} {entity-type}                   [×]    │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  ⚠ You're about to {action}:                     │
   │                                                  │
   │  [Identity card for the entity being affected]   │
   │                                                  │
   │  What happens:                                   │
   │   ▸ {Consequence 1}                              │
   │   ▸ {Consequence 2}                              │
   │   ▸ {Consequence 3}                              │
   │   ▸ {Recovery info, if any}                      │
   │                                                  │
   │  Type {what to type} to confirm:                 │
   │  {literal-confirm-text}                          │
   │  ┌────────────────────────────────────────────┐  │
   │  │                                            │  │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]       [{Action}]           │
   └──────────────────────────────────────────────────┘
```

### Tier 4 — Type-to-confirm with extra confirmation

For catastrophic, non-recoverable actions: deleting your own admin account, signing out everywhere when no other admins exist, etc.

**Appearance:**
- Same as Tier 3 but with additional warning callouts
- Sometimes a checkbox that says "I understand this is non-recoverable" must be ticked
- Action button stays in danger color throughout

This tier is rare in this admin UI since kanidm uses soft-delete (recycle bin) for most things. Reserved for sign-out-everywhere when it's the user's last session, or analogous.

## What to type

For type-to-confirm:

- **Single entity:** type the entity's exact identifier (SPN, group name, OAuth2 system name, key ID). This is the most natural and meaningful to type.
- **Bulk:** type literal `DELETE` (uppercase) — simpler and less error-prone than typing many identifiers.

The label clearly indicates what to type:
- "Type the SPN to confirm: alice.smith@idm.example.com"
- "Type the group name to confirm: developers"
- "Type the application name to confirm: grafana"
- "Type DELETE to confirm deletion of 5 items"

The text to type is shown immediately above the input, in monospace, with a copy button (allowing copy-paste is fine — the friction is the deliberate act of copying and pasting, not pure typing).

## Validation behavior

- Input is compared exactly to the target (case-sensitive)
- Match: primary action button enables (e.g., color transitions from disabled to active danger)
- Mismatch: button stays disabled, no error shown until the user clicks (which they can't, since it's disabled)
- Paste support: paste of the exact value enables instantly
- Empty input: button disabled

## Privilege session integration

Before any destructive action proceeds:
1. UI checks if privilege session is active
2. If not, the reauth modal (screen 08) appears BEFORE the confirm modal opens
3. After reauth succeeds, the confirm modal opens normally
4. After confirm input matches and the user clicks the action button, the destructive API call fires

This means: type-to-confirm + privilege session = double-friction for the most dangerous actions. Privilege session ensures identity; type-to-confirm ensures intent.

## Visual treatment

- The danger primary button always uses the design system's `--danger` color
- Modal header may have a tinted background (`--danger-soft`) for highest tier
- Icons use Lucide `AlertTriangle` (warning) or `AlertOctagon` (severe)
- The "What happens" list uses small chevron / arrow markers to make it scannable

## Sample data references

- All deletion examples use samples from `_sample-data.md`: Alice Smith / `alice.smith@idm.example.com`, `developers` group, `grafana` OAuth2 app

## Mockup elements to render

Render 4 distinct destructive confirmation variants:

1. **Tier 1 — undo toast pattern** — A toast in top-right: "Email removed. [Undo]" — shows immediately after the action
2. **Tier 2 — standard confirm modal** — Modal "Destroy session" with brief body and Cancel/Destroy buttons
3. **Tier 3 — type-to-confirm** — Modal "Delete person" with identity card for Alice Smith, "What happens" list, type-to-confirm with `alice.smith@idm.example.com` as the target, input empty, Delete button disabled
4. **Tier 3 enabled state** — Same modal but with input filled and Delete button enabled in danger color
